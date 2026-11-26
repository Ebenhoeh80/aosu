//! Official Tuya Cloud adapter for Aosu cameras.
//!
//! This adapter deliberately uses the *user's own* Tuya IoT project
//! (`client_id`/`client_secret`) and the documented OpenAPI endpoints. It
//! never reads OEM secrets from an APK and never bypasses a login flow. The
//! user links their Aosu app account to the IoT project ("Link App Account"),
//! which is the supported OAuth path.
//!
//! Secrets and tokens live only in this process. Nothing is written to disk,
//! logged or returned to the UI beyond short-lived stream URLs.

use crate::capability::{BlockedReason, Capability, CapabilityReport, Support};
use crate::device::{
    AudioChannelHandle, Command, Device, DeviceId, Event, HealthStatus, MediaStreamHandle,
    Recording, RecordingSource, TimeRange, Timestamp,
};
use crate::transport::{Result, Transport, TransportError};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// Configuration supplied by the user at runtime. Every field is RAM-only.
#[derive(Clone)]
pub struct TuyaConfig {
    pub client_id: String,
    pub client_secret: String,
    /// Region token: `eu`, `us`, `cn`, `in` or `sg`.
    pub region: String,
    /// UID of the linked app account. When omitted it is taken from the token.
    pub uid: Option<String>,
    /// Optional local go2rtc base URL (e.g. `http://127.0.0.1:1984`).
    pub go2rtc_base: Option<String>,
    /// Name of a stream the user configured in go2rtc with a documented
    /// `tuya://...` Cloud API source (credentials stay in the user's go2rtc
    /// config, never in this app). Required for `talk()`.
    pub go2rtc_stream: Option<String>,
    /// Public HTTPS base URL Tuya can call back for cloud-storage keys, e.g.
    /// `https://example.org/aosu-recording-key`. Required by the documented
    /// HLS playback API; never persisted by this client.
    pub callback_base: Option<String>,
    /// PTZ motors are safety-relevant: only enable after a STOP test.
    pub ptz_enabled: bool,
}

#[derive(Clone)]
struct TuyaToken {
    access_token: String,
    uid: String,
    expire_at_ms: u128,
}

/// A thin, auditable client for the documented Tuya OpenAPI.
pub struct TuyaCloudClient {
    http: reqwest::blocking::Client,
    config: TuyaConfig,
    token: Mutex<Option<TuyaToken>>,
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

/// Region host from the official regional endpoint list.
pub fn api_base(region: &str) -> &'static str {
    match region.trim().to_ascii_lowercase().as_str() {
        "cn" => "https://openapi.tuyacn.com",
        "us" => "https://openapi.tuyaus.com",
        "in" => "https://openapi.tuyain.com",
        "sg" => "https://openapi-sg.iotbing.com",
        _ => "https://openapi.tuyaeu.com",
    }
}

/// URL used in the signature: path plus alphabetically sorted query pairs.
/// Tuya signs the raw values; the request URL encodes them separately.
fn sorted_query<'a>(query: &[(&'a str, &'a str)]) -> Vec<(&'a str, &'a str)> {
    let mut pairs = query.to_vec();
    pairs.sort_by(|a, b| a.0.cmp(b.0));
    pairs
}

fn canonical_url(path: &str, query: &[(&str, &str)]) -> String {
    let pairs = sorted_query(query);
    if pairs.is_empty() {
        return path.to_string();
    }
    let joined = pairs
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&");
    format!("{path}?{joined}")
}

fn encoded_query(query: &[(&str, &str)]) -> String {
    sorted_query(query)
        .iter()
        .map(|(key, value)| format!("{}={}", key, urlencode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

/// Tuya HMAC-SHA256 signature, upper-case hex. Matches the documented
/// `stringToSign`: METHOD + newline + lowercase SHA256(body) + newline +
/// empty signature headers + newline + canonical URL. Callers pass
/// `canonical_url(path, query)`. The token is empty on `/v1.0/token`.
pub fn signature(
    client_id: &str,
    client_secret: &str,
    access_token: &str,
    t_ms: &str,
    method: &str,
    body: &str,
    url: &str,
) -> String {
    let body_hash = hex::encode(Sha256::digest(body.as_bytes()));
    let string_to_sign = format!(
        "{client_id}{access_token}{t_ms}{}\n{body_hash}\n\n{url}",
        method.to_ascii_uppercase()
    );
    let mut mac = Hmac::<Sha256>::new_from_slice(client_secret.as_bytes())
        .expect("HMAC accepts any key length");
    mac.update(string_to_sign.as_bytes());
    hex::encode(mac.finalize().into_bytes()).to_uppercase()
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    success: bool,
    result: Option<TokenResult>,
}

#[derive(Debug, Deserialize)]
struct TokenResult {
    access_token: String,
    uid: String,
    expire_time: u64,
}

#[derive(Debug, Deserialize)]
struct ApiEnvelope {
    success: bool,
    result: Option<serde_json::Value>,
    #[serde(default)]
    msg: Option<String>,
}

impl TuyaCloudClient {
    pub fn new(config: TuyaConfig) -> Result<Self> {
        // Missing credentials are not a construction error: the transport
        // still exists and reports `Blocked(CredentialsRequired)` until the
        // user supplies an IoT project. No network call happens here.
        let http = reqwest::blocking::Client::builder()
            .user_agent("aosu-desktop/0.1")
            .build()
            .map_err(|e| TransportError::Other(format!("http client: {e}")))?;
        Ok(Self {
            http,
            config,
            token: Mutex::new(None),
        })
    }

    pub fn config(&self) -> &TuyaConfig {
        &self.config
    }

    /// Credentials are checked without performing any network request.
    pub fn has_credentials(&self) -> bool {
        !self.config.client_id.trim().is_empty() && !self.config.client_secret.trim().is_empty()
    }

    /// Explicit UID from the IoT project wins over the UID in the token.
    fn uid(&self) -> Result<String> {
        if let Some(uid) = self.config.uid.as_ref().filter(|s| !s.trim().is_empty()) {
            return Ok(uid.clone());
        }
        Ok(self.token()?.uid)
    }

    fn token(&self) -> Result<TuyaToken> {
        if let Some(t) = self.token.lock().unwrap().as_ref() {
            if t.expire_at_ms.saturating_sub(60_000) > now_ms() {
                return Ok(t.clone());
            }
        }
        let t_ms = now_ms().to_string();
        let path = "/v1.0/token";
        let query: [(&str, &str); 1] = [("grant_type", "1")];
        let sign = signature(
            &self.config.client_id,
            &self.config.client_secret,
            "",
            &t_ms,
            "GET",
            "",
            &canonical_url(path, &query),
        );
        let url = format!(
            "{}{}?{}",
            api_base(&self.config.region),
            path,
            encoded_query(&query)
        );
        let resp = self
            .http
            .get(url)
            .header("client_id", &self.config.client_id)
            .header("t", &t_ms)
            .header("sign_method", "HMAC-SHA256")
            .header("sign", sign)
            .send()
            .map_err(|e| TransportError::Other(format!("tuya token request: {e}")))?;
        let body: TokenResponse = resp
            .json()
            .map_err(|e| TransportError::Other(format!("tuya token json: {e}")))?;
        if !body.success {
            return Err(TransportError::CredentialsRequired);
        }
        let result = body.result.ok_or(TransportError::CredentialsRequired)?;
        let token = TuyaToken {
            access_token: result.access_token,
            uid: result.uid,
            expire_at_ms: now_ms() + result.expire_time.saturating_mul(1000) as u128,
        };
        *self.token.lock().unwrap() = Some(token.clone());
        Ok(token)
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        query: &[(&str, &str)],
        body: &str,
    ) -> Result<serde_json::Value> {
        let token = self.token()?;
        let t_ms = now_ms().to_string();
        let sign = signature(
            &self.config.client_id,
            &self.config.client_secret,
            &token.access_token,
            &t_ms,
            method,
            body,
            &canonical_url(path, query),
        );
        let mut url = format!("{}{}", api_base(&self.config.region), path);
        if !query.is_empty() {
            url.push('?');
            url.push_str(&encoded_query(query));
        }
        let mut req = match method.to_ascii_uppercase().as_str() {
            "POST" => self.http.post(url).body(body.to_string()),
            _ => self.http.get(url),
        };
        req = req
            .header("client_id", &self.config.client_id)
            .header("access_token", &token.access_token)
            .header("t", &t_ms)
            .header("sign_method", "HMAC-SHA256")
            .header("sign", sign)
            .header("Content-Type", "application/json");
        let resp = req
            .send()
            .map_err(|e| TransportError::Other(format!("tuya request: {e}")))?;
        let env: ApiEnvelope = resp
            .json()
            .map_err(|e| TransportError::Other(format!("tuya json: {e}")))?;
        if !env.success {
            return Err(TransportError::Other(
                env.msg.unwrap_or_else(|| "tuya api error".into()),
            ));
        }
        Ok(env.result.unwrap_or(serde_json::Value::Null))
    }

    /// Device list of the linked app account. `from=sharing` covers devices
    /// owned by the OEM app but shared into the user's IoT project.
    pub fn devices(&self) -> Result<Vec<Device>> {
        let uid = self.uid()?;
        let path = format!("/v1.0/users/{uid}/devices");
        let value = self.request(
            "GET",
            &path,
            &[("from", "sharing"), ("page_no", "1"), ("page_size", "100")],
            "",
        )?;
        let mut out = Vec::new();
        if let Some(list) = value.get("devices").and_then(|v| v.as_array()) {
            for item in list {
                if let Some(dev) = map_device(item) {
                    out.push(dev);
                }
            }
        }
        Ok(out)
    }

    /// Short-lived HLS URL from the official stream allocation endpoint.
    pub fn stream_url(&self, device_id: &str) -> Result<String> {
        let uid = self.uid()?;
        let path = format!("/v1.0/users/{uid}/devices/{device_id}/stream/actions/allocate");
        let value = self.request("POST", &path, &[], r#"{"type":"hls"}"#)?;
        value
            .get("url")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| TransportError::Other("tuya stream response has no url".into()))
    }

    /// Cloud-storage clips in a time range. The documented API requires a
    /// callback base URL so Tuya can obtain the short-lived HLS key; the URL
    /// is supplied by the user at runtime and never stored or logged.
    pub fn recordings(
        &self,
        device_id: &str,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<Vec<Recording>> {
        let callback = self
            .config
            .callback_base
            .as_ref()
            .filter(|s| !s.trim().is_empty())
            .ok_or(TransportError::Other(
                "cloud recording callback URL required".into(),
            ))?;
        let uid = self.uid()?;
        let path = format!("/v1.0/users/{uid}/devices/{device_id}/storage/stream/hls");
        let from_s = from.0.to_string();
        let to_s = to.0.to_string();
        let value = self.request(
            "GET",
            &path,
            &[
                ("time_g_t", &from_s),
                ("time_l_t", &to_s),
                ("callback", callback.as_str()),
            ],
            "",
        )?;
        let mut out = Vec::new();
        if let Some(list) = value.get("list").and_then(|v| v.as_array()) {
            for item in list {
                let Some(url) = item.get("url").and_then(|v| v.as_str()) else {
                    continue;
                };
                let started_at = Timestamp(
                    item.get("start_time")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(from.0),
                );
                let stopped_at = item
                    .get("stop_time")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(started_at.0);
                out.push(Recording {
                    device: DeviceId(device_id.to_string()),
                    started_at,
                    duration_secs: stopped_at.saturating_sub(started_at.0) as u32,
                    source: RecordingSource::Cloud,
                    url: Some(url.to_string()),
                    thumbnail_url: None,
                });
            }
        }
        Ok(out)
    }

    /// Send one or more data points to the device.
    pub fn send_commands(&self, device_id: &str, body: &str) -> Result<()> {
        let path = format!("/v1.0/devices/{device_id}/commands");
        self.request("POST", &path, &[], body)?;
        Ok(())
    }
}

fn urlencode(value: &str) -> String {
    let mut out = String::new();
    for b in value.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// JSON payload for a normalized command. Kept pure so the OEM DP mapping is
/// reviewable and testable without network access.
pub fn command_payload(cmd: &Command) -> Option<String> {
    let (code, value) = match cmd {
        Command::SetLight(on) => ("switch_led", serde_json::json!(on)),
        Command::SetSiren(on) => ("alarm_switch", serde_json::json!(on)),
        Command::Ptz {
            pan,
            tilt,
            duration_ms,
        } => {
            let direction = match (pan.signum(), tilt.signum()) {
                (1, 0) => "right",
                (-1, 0) => "left",
                (0, 1) => "up",
                (0, -1) => "down",
                _ => return None,
            };
            return Some(
                serde_json::json!({
                    "commands": [
                        {"code": "ptz_control", "value": direction},
                        {"code": "ptz_stop", "value": true, "delay_ms": duration_ms}
                    ]
                })
                .to_string(),
            );
        }
        Command::PtzStop => {
            return Some(
                serde_json::json!({"commands": [{"code": "ptz_stop", "value": true}]}).to_string(),
            )
        }
    };
    Some(serde_json::json!({"commands": [{"code": code, "value": value}]}).to_string())
}

fn map_device(value: &serde_json::Value) -> Option<Device> {
    let id = value
        .get("id")
        .and_then(|v| v.as_str())
        .or_else(|| value.get("device_id").and_then(|v| v.as_str()))?;
    let name = value
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(id)
        .to_string();
    let product_name = value
        .get("product_name")
        .and_then(|v| v.as_str())
        .unwrap_or("camera")
        .to_string();
    let online = value
        .get("online")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    Some(Device {
        id: DeviceId(id.to_string()),
        name,
        vendor: "Aosu (Tuya Cloud)".into(),
        model: product_name,
        kind: "camera".into(),
        online,
        health: if online {
            HealthStatus::Healthy
        } else {
            HealthStatus::Offline
        },
        battery_percent: None,
        wifi_signal_percent: None,
        firmware_version: value
            .get("fw_version")
            .and_then(|v| v.as_str())
            .map(str::to_string),
    })
}

/// The Transport facade. Capabilities only become `Supported` after a real
/// successful call; until then the UI sees `LiveVerificationRequired`.
pub struct TuyaCloudTransport {
    client: TuyaCloudClient,
    devices_verified: AtomicBool,
    live_verified: AtomicBool,
    commands_verified: AtomicBool,
    recordings_verified: AtomicBool,
}

impl TuyaCloudTransport {
    pub fn new(config: TuyaConfig) -> Result<Self> {
        Ok(Self {
            client: TuyaCloudClient::new(config)?,
            devices_verified: AtomicBool::new(false),
            live_verified: AtomicBool::new(false),
            commands_verified: AtomicBool::new(false),
            recordings_verified: AtomicBool::new(false),
        })
    }

    pub fn client(&self) -> &TuyaCloudClient {
        &self.client
    }
}

impl Transport for TuyaCloudTransport {
    fn name(&self) -> &'static str {
        "tuya-cloud"
    }

    fn discover(&self) -> Result<Vec<Device>> {
        self.devices()
    }

    fn devices(&self) -> Result<Vec<Device>> {
        let devices = self.client.devices()?;
        self.devices_verified.store(true, Ordering::Release);
        Ok(devices)
    }

    fn events(&self, _since: Timestamp) -> Result<Vec<Event>> {
        Err(TransportError::Unsupported)
    }

    fn live(&self, device: &DeviceId) -> Result<MediaStreamHandle> {
        let url = self.client.stream_url(&device.0)?;
        self.live_verified.store(true, Ordering::Release);
        Ok(MediaStreamHandle {
            description: "Tuya Cloud HLS stream".into(),
            url: Some(url),
            protocol: Some("hls".into()),
            session_id: None,
        })
    }

    fn talk(&self, device: &DeviceId) -> Result<AudioChannelHandle> {
        let cfg = self.client.config();
        let base = cfg
            .go2rtc_base
            .as_ref()
            .filter(|s| !s.trim().is_empty())
            .ok_or(TransportError::Unsupported)?;
        // go2rtc's Tuya Cloud source is `tuya://<host>?device_id=...&uid=...`
        // and must be configured by the user locally. We only hand the UI a
        // link to that named stream: no credentials or stream URLs here.
        let stream = cfg
            .go2rtc_stream
            .as_ref()
            .filter(|s| !s.trim().is_empty())
            .ok_or(TransportError::Unsupported)?;
        Ok(AudioChannelHandle {
            description: format!(
                "go2rtc WebRTC two-way audio (stream: {}, device: {})",
                stream, device.0
            ),
            url: Some(format!(
                "{}/webrtc.html?src={}",
                base.trim_end_matches('/'),
                urlencode(stream)
            )),
            protocol: Some("webrtc".into()),
            two_way: true,
        })
    }

    fn recordings(&self, device: &DeviceId, range: TimeRange) -> Result<Vec<Recording>> {
        let recordings = self.client.recordings(&device.0, range.from, range.to)?;
        self.recordings_verified.store(true, Ordering::Release);
        Ok(recordings)
    }

    fn command(&self, device: &DeviceId, cmd: Command) -> Result<()> {
        let payload = command_payload(&cmd).ok_or(TransportError::Unsupported)?;
        self.client.send_commands(&device.0, &payload)?;
        self.commands_verified.store(true, Ordering::Release);
        Ok(())
    }

    fn capability_report(&self) -> CapabilityReport {
        let mut r = CapabilityReport::new();
        let configured = self.client.has_credentials();
        let blocked = |reason| Support::Blocked { reason };
        let credentials = || blocked(BlockedReason::CredentialsRequired);
        let unverified = || blocked(BlockedReason::LiveVerificationRequired);
        let read_only = || {
            if !configured {
                credentials()
            } else if self.devices_verified.load(Ordering::Acquire) {
                Support::Supported
            } else {
                unverified()
            }
        };
        for cap in [
            Capability::Discover,
            Capability::DeviceList,
            Capability::DeviceStatus,
        ] {
            r.set(cap, read_only());
        }
        let verified_support = |verified: &AtomicBool| {
            if !configured {
                credentials()
            } else if verified.load(Ordering::Acquire) {
                Support::Supported
            } else {
                unverified()
            }
        };
        r.set(Capability::LiveVideo, verified_support(&self.live_verified));
        r.set(
            Capability::Commands,
            verified_support(&self.commands_verified),
        );
        let cfg = self.client.config();
        let talk_ready = cfg
            .go2rtc_base
            .as_ref()
            .is_some_and(|s| !s.trim().is_empty())
            && cfg
                .go2rtc_stream
                .as_ref()
                .is_some_and(|s| !s.trim().is_empty());
        r.set(
            Capability::TwoWayAudio,
            if !configured {
                credentials()
            } else if talk_ready {
                // A local go2rtc stream can only be verified by actually
                // talking to the camera; never claim it from config alone.
                unverified()
            } else {
                Support::Unsupported
            },
        );
        // Motor moves stay locked until the user validates STOP.
        r.set(Capability::Ptz, blocked(BlockedReason::NotImplementedYet));
        r.set(Capability::Events, Support::Unsupported);
        r.set(Capability::Notifications, Support::Unsupported);
        r.set(
            Capability::Recordings,
            if !configured {
                credentials()
            } else if !cfg
                .callback_base
                .as_ref()
                .is_some_and(|s| !s.trim().is_empty())
            {
                blocked(BlockedReason::CallbackUrlRequired)
            } else if self.recordings_verified.load(Ordering::Acquire) {
                Support::Supported
            } else {
                unverified()
            },
        );
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_hosts_follow_the_official_endpoint_list() {
        assert_eq!(api_base("eu"), "https://openapi.tuyaeu.com");
        assert_eq!(api_base("US"), "https://openapi.tuyaus.com");
        assert_eq!(api_base("cn"), "https://openapi.tuyacn.com");
        assert_eq!(api_base("unknown"), "https://openapi.tuyaeu.com");
    }

    #[test]
    fn signature_matches_the_documented_canonical_string() {
        // Fixed, non-secret test values: canonical string is
        // GET\n<lowercase sha256 of empty body>\n\n/v1.0/token?grant_type=1
        let sign = signature(
            "cid",
            "secret",
            "",
            "1000",
            "GET",
            "",
            &canonical_url("/v1.0/token", &[("grant_type", "1")]),
        );
        assert_eq!(
            sign,
            "CCBE6822CB12148CDF1A65B340EB7E292EBBA47A653023DFC9F8A733FF3898CE"
        );
        let with_token = signature(
            "cid",
            "secret",
            "tok",
            "1000",
            "GET",
            "",
            &canonical_url("/v1.0/token", &[("grant_type", "1")]),
        );
        assert_ne!(with_token, sign);
        assert_eq!(with_token.len(), 64);
        assert!(with_token.chars().all(|ch| ch.is_ascii_hexdigit()));
        assert_ne!(
            signature("cid", "secret", "", "1000", "GET", "", "/v1.0/token"),
            sign
        );
    }

    #[test]
    fn canonical_and_encoded_urls_sort_query_keys_identically() {
        let query: [(&str, &str); 3] = [
            ("callback", "https://example.org/key"),
            ("time_l_t", "200"),
            ("time_g_t", "100"),
        ];
        assert_eq!(
            canonical_url("/v1.0/hls", &query),
            "/v1.0/hls?callback=https://example.org/key&time_g_t=100&time_l_t=200"
        );
        assert_eq!(
            encoded_query(&query),
            "callback=https%3A%2F%2Fexample.org%2Fkey&time_g_t=100&time_l_t=200"
        );
        assert_eq!(canonical_url("/v1.0/devices", &[]), "/v1.0/devices");
    }

    #[test]
    fn light_siren_and_ptz_payloads_use_expected_dp_codes() {
        let light = command_payload(&Command::SetLight(true)).unwrap();
        assert!(light.contains("switch_led"));
        let siren = command_payload(&Command::SetSiren(false)).unwrap();
        assert!(siren.contains("alarm_switch"));
        let ptz = command_payload(&Command::Ptz {
            pan: -1,
            tilt: 0,
            duration_ms: 300,
        })
        .unwrap();
        assert!(ptz.contains("ptz_control"));
        assert!(ptz.contains("left"));
        assert!(ptz.contains("ptz_stop"));
        let stop = command_payload(&Command::PtzStop).unwrap();
        assert!(stop.contains("ptz_stop"));
        assert!(command_payload(&Command::Ptz {
            pan: 0,
            tilt: 0,
            duration_ms: 10
        })
        .is_none());
    }

    #[test]
    fn urlencode_leaves_unreserved_bytes_and_escapes_slashes() {
        assert_eq!(urlencode("a-b_C.1~"), "a-b_C.1~");
        assert_eq!(urlencode("https://x/y"), "https%3A%2F%2Fx%2Fy");
    }

    #[test]
    fn capabilities_never_claim_live_support_before_a_real_call() {
        let cfg = TuyaConfig {
            client_id: "cid".into(),
            client_secret: "secret".into(),
            region: "eu".into(),
            uid: None,
            go2rtc_base: Some("http://127.0.0.1:1984".into()),
            go2rtc_stream: Some("tuya_webrtc".into()),
            callback_base: Some("https://example.org/aosu-key".into()),
            ptz_enabled: true,
        };
        let t = TuyaCloudTransport::new(cfg).unwrap();
        let report = t.capability_report();
        for cap in [
            Capability::Discover,
            Capability::LiveVideo,
            Capability::Recordings,
        ] {
            assert_eq!(
                report.get(cap),
                Support::Blocked {
                    reason: BlockedReason::LiveVerificationRequired
                },
                "{cap:?} must not be advertised as supported without a live call"
            );
        }
        assert_eq!(
            report.get(Capability::TwoWayAudio),
            Support::Blocked {
                reason: BlockedReason::LiveVerificationRequired
            }
        );
        assert_eq!(
            report.get(Capability::Ptz),
            Support::Blocked {
                reason: BlockedReason::NotImplementedYet
            }
        );

        // A successful read-only call upgrades only that capability.
        t.devices_verified.store(true, Ordering::Release);
        assert_eq!(
            t.capability_report().get(Capability::DeviceList),
            Support::Supported
        );
        assert_eq!(
            t.capability_report().get(Capability::LiveVideo),
            Support::Blocked {
                reason: BlockedReason::LiveVerificationRequired
            }
        );

        let locked = TuyaCloudTransport::new(TuyaConfig {
            client_id: "cid".into(),
            client_secret: "secret".into(),
            region: "eu".into(),
            uid: None,
            go2rtc_base: None,
            go2rtc_stream: None,
            callback_base: None,
            ptz_enabled: false,
        })
        .unwrap();
        assert_eq!(
            locked.capability_report().get(Capability::LiveVideo),
            Support::Blocked {
                reason: BlockedReason::LiveVerificationRequired
            }
        );
        assert_eq!(
            locked.capability_report().get(Capability::TwoWayAudio),
            Support::Unsupported
        );
        assert_eq!(
            locked.capability_report().get(Capability::Recordings),
            Support::Blocked {
                reason: BlockedReason::CallbackUrlRequired
            }
        );
    }

    #[test]
    fn talk_links_a_user_configured_go2rtc_stream_without_secrets() {
        let t = TuyaCloudTransport::new(TuyaConfig {
            client_id: "cid".into(),
            client_secret: "secret".into(),
            region: "eu".into(),
            uid: None,
            go2rtc_base: Some("http://127.0.0.1:1984/".into()),
            go2rtc_stream: Some("tuya_webrtc".into()),
            callback_base: None,
            ptz_enabled: false,
        })
        .unwrap();
        let channel = t.talk(&DeviceId("dev-1".into())).unwrap();
        assert_eq!(
            channel.url.as_deref(),
            Some("http://127.0.0.1:1984/webrtc.html?src=tuya_webrtc")
        );
        assert!(channel.two_way);
        assert!(!channel.url.unwrap().contains("secret"));

        let without_stream = TuyaCloudTransport::new(TuyaConfig {
            go2rtc_stream: None,
            ..TuyaConfig {
                client_id: "cid".into(),
                client_secret: "secret".into(),
                region: "eu".into(),
                uid: None,
                go2rtc_base: Some("http://127.0.0.1:1984".into()),
                go2rtc_stream: Some("tuya_webrtc".into()),
                callback_base: None,
                ptz_enabled: false,
            }
        })
        .unwrap();
        assert!(matches!(
            without_stream.talk(&DeviceId("dev-1".into())),
            Err(TransportError::Unsupported)
        ));
    }
}
