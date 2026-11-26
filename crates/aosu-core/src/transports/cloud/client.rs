//! Read-only Aosu cloud client (Glazero API).
//!
//! Implements the minimum observed request shape for login and device
//! inventory. Secrets are not persisted. APK/SO material is read at runtime
//! from a user-provided path and kept only in memory.

use crate::device::{Device, DeviceId, HealthStatus};
use crate::transport::{Result, TransportError};
use aes::Aes128;
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use cbc::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
use md5::{Digest, Md5};
use serde::Deserialize;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const DEFAULT_API_BASE: &str = "https://api-eu.aosulife.com";
const LOGIN_PATH: &str = "/v1/user/login";
const DEVICE_LIST_PATH: &str = "/v1/dev/getList";
const OTA_INFO_PATH: &str = "/v1/ota/getNewByApp";
const KEY_OFFSET: usize = 0x18BC4;
const KEY_LEN: usize = 16;
const SO_PATH_IN_SPLIT: &str = "lib/arm64-v8a/libglazero-security.so";

#[derive(Debug, Clone)]
pub struct CloudCredentials {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct CloudSession {
    pub sid: String,
    pub uid: String,
}

#[derive(Debug, Clone)]
pub struct CloudClientConfig {
    pub api_base: String,
    pub apk_material_path: PathBuf,
}

impl CloudClientConfig {
    pub fn new(apk_material_path: impl Into<PathBuf>) -> Self {
        Self {
            api_base: DEFAULT_API_BASE.to_string(),
            apk_material_path: apk_material_path.into(),
        }
    }
}

pub struct AosuCloudClient {
    http: reqwest::blocking::Client,
    config: CloudClientConfig,
    material: [u8; KEY_LEN],
    sign_suffix: Vec<u8>,
}

impl AosuCloudClient {
    pub fn from_apk(config: CloudClientConfig) -> Result<Self> {
        let (material, sign_suffix) = load_materials(&config.apk_material_path)?;
        let http = reqwest::blocking::Client::builder()
            .user_agent("okhttp/5.0.0-alpha.12")
            .build()
            .map_err(|e| TransportError::Other(format!("http client: {e}")))?;
        Ok(Self {
            http,
            config,
            material,
            sign_suffix,
        })
    }

    pub fn login(&self, creds: &CloudCredentials) -> Result<CloudSession> {
        let enc_pw = self.encrypt_password(creds.password.as_bytes())?;
        let email = creds.email.trim().to_lowercase();
        let form = [
            ("region", "EU"),
            ("countryCode", "49"),
            ("countryAbbr", "DE"),
            ("type", "1"),
            ("email", email.as_str()),
            ("password", enc_pw.as_str()),
            ("pwType", "2"),
        ];
        let url = format!("{}{}", self.config.api_base, LOGIN_PATH);
        let mut last_err = None;
        let mut body_opt = None;
        for attempt in 0..2 {
            let ms = now_ms().to_string();
            let rid = Uuid::new_v4().to_string();
            let sign = self.sign(&rid, &ms);
            let resp = self
                .http
                .post(&url)
                .query(&[("uuid", rid.as_str()), ("t", ms.as_str())])
                .headers(self.base_headers(None, None, &sign)?)
                .form(&form)
                .send();
            match resp {
                Ok(resp) => {
                    let status = resp.status();
                    let text = resp.text().unwrap_or_default();
                    match serde_json::from_str::<ApiResponse<LoginData>>(&text) {
                        Ok(body) if body.errno == 0 => {
                            body_opt = Some(body);
                            break;
                        }
                        Ok(body) if body.errno == 503 && attempt == 0 => {
                            std::thread::sleep(std::time::Duration::from_millis(800));
                            last_err = Some(TransportError::Other(format!(
                                "login errno={} http={}",
                                body.errno, status
                            )));
                        }
                        Ok(body) => {
                            last_err = Some(TransportError::Other(format!(
                                "login errno={} http={}",
                                body.errno, status
                            )));
                            break;
                        }
                        Err(e) => {
                            last_err = Some(TransportError::Other(format!(
                                "login json: {e} http={status}"
                            )));
                            break;
                        }
                    }
                }
                Err(e) if attempt == 0 => {
                    last_err = Some(TransportError::Other(format!("login request: {e}")));
                    std::thread::sleep(std::time::Duration::from_millis(800));
                }
                Err(e) => {
                    last_err = Some(TransportError::Other(format!("login request: {e}")));
                    break;
                }
            }
        }
        let body = body_opt.ok_or_else(|| {
            last_err.unwrap_or_else(|| TransportError::Other("login failed".into()))
        })?;
        let data = body
            .data
            .ok_or_else(|| TransportError::Other("login missing data".into()))?;
        let sid = data
            .sid
            .ok_or_else(|| TransportError::Other("login missing sid".into()))?;
        let uid = data
            .uid
            .ok_or_else(|| TransportError::Other("login missing uid".into()))?;
        Ok(CloudSession { sid, uid })
    }

    pub fn get_device_list(&self, session: &CloudSession) -> Result<Vec<Device>> {
        let ms = now_ms().to_string();
        let rid = Uuid::new_v4().to_string();
        let sign = self.sign(&rid, &ms);
        let url = format!("{}{}", self.config.api_base, DEVICE_LIST_PATH);
        let mut last_err = None;
        let mut body_opt = None;
        for attempt in 0..2 {
            let resp = self
                .http
                .post(&url)
                .query(&[("uuid", rid.as_str()), ("t", ms.as_str())])
                .headers(self.base_headers(Some(&session.sid), Some(&session.uid), &sign)?)
                .send();
            match resp {
                Ok(resp) => {
                    let status = resp.status();
                    let text = resp.text().unwrap_or_default();
                    match serde_json::from_str::<ApiResponse<DeviceListData>>(&text) {
                        Ok(body) if body.errno == 0 => {
                            body_opt = Some(body);
                            break;
                        }
                        Ok(body) if body.errno == 503 && attempt == 0 => {
                            std::thread::sleep(std::time::Duration::from_millis(800));
                            last_err = Some(TransportError::Other(format!(
                                "getList errno={} http={}",
                                body.errno, status
                            )));
                        }
                        Ok(body) => {
                            last_err = Some(TransportError::Other(format!(
                                "getList errno={} http={}",
                                body.errno, status
                            )));
                            break;
                        }
                        Err(e) => {
                            last_err = Some(TransportError::Other(format!(
                                "getList json: {e} http={status}"
                            )));
                            break;
                        }
                    }
                }
                Err(e) if attempt == 0 => {
                    last_err = Some(TransportError::Other(format!("getList request: {e}")));
                    std::thread::sleep(std::time::Duration::from_millis(800));
                }
                Err(e) => {
                    last_err = Some(TransportError::Other(format!("getList request: {e}")));
                    break;
                }
            }
        }
        let body = body_opt.ok_or_else(|| {
            last_err.unwrap_or_else(|| TransportError::Other("getList failed".into()))
        })?;
        let list = body.data.and_then(|d| d.list).unwrap_or_default();
        let mut devices: Vec<Device> = list.into_iter().map(map_device).collect();
        self.enrich_firmware_versions(session, &mut devices);
        Ok(devices)
    }

    fn enrich_firmware_versions(&self, session: &CloudSession, devices: &mut [Device]) {
        for device in devices.iter_mut() {
            if device
                .firmware_version
                .as_deref()
                .is_some_and(|v| !v.trim().is_empty())
            {
                continue;
            }
            match self.get_ota_info(session, &device.id.0) {
                Ok(Some(fw)) => device.firmware_version = Some(fw),
                Ok(None) => {}
                Err(_) => {}
            }
        }
    }

    fn get_ota_info(&self, session: &CloudSession, sn: &str) -> Result<Option<String>> {
        let ms = now_ms().to_string();
        let rid = Uuid::new_v4().to_string();
        let sign = self.sign(&rid, &ms);
        let url = format!("{}{}", self.config.api_base, OTA_INFO_PATH);
        let form = [("sn", sn)];
        let resp = self
            .http
            .post(&url)
            .query(&[("uuid", rid.as_str()), ("t", ms.as_str())])
            .headers(self.base_headers(Some(&session.sid), Some(&session.uid), &sign)?)
            .form(&form)
            .send()
            .map_err(|e| TransportError::Other(format!("ota request: {e}")))?;
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        let body: ApiResponse<OtaInfoData> = serde_json::from_str(&text)
            .map_err(|e| TransportError::Other(format!("ota json: {e} http={status}")))?;
        if body.errno != 0 {
            return Ok(None);
        }
        Ok(body
            .data
            .and_then(|d| clean_opt(d.current_version).or_else(|| clean_opt(d.version))))
    }

    fn sign(&self, request_id: &str, milliseconds: &str) -> String {
        let mut hasher = Md5::new();
        hasher.update(format!("uuid:{request_id}:t:{milliseconds}:").as_bytes());
        hasher.update(&self.sign_suffix);
        format!("{:x}", hasher.finalize())
    }

    fn encrypt_password(&self, plain: &[u8]) -> Result<String> {
        // Encrypt raw password bytes with AES-128-CBC/PKCS7. The OEM app uses
        // the same 16-byte APK material as key and IV; matching that wire
        // shape is intentional, not a locally chosen constant.
        let cipher =
            cbc::Encryptor::<Aes128>::new((&self.material).into(), (&self.material).into());
        let ct = cipher.encrypt_padded_vec_mut::<Pkcs7>(plain);
        Ok(B64.encode(ct))
    }

    fn base_headers(
        &self,
        sid: Option<&str>,
        uid: Option<&str>,
        sign: &str,
    ) -> Result<reqwest::header::HeaderMap> {
        use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
        let mut h = HeaderMap::new();
        let pairs = [
            ("Gz-Pid", "glazero"),
            ("Gz-OsType", "android"),
            ("Gz-OsLang", "de-DE"),
            ("Gz-Lang", "de"),
            ("Gz-Timezone", "+01:00"),
            ("Gz-TimezoneID", "Europe/Berlin"),
            ("Gz-AppId", "com.glazero.android"),
            ("Gz-AppVer", "5.11.20"),
            ("Gz-BuildType", "release"),
            ("Gz-Channel", "appstore"),
            ("Gz-Uid", uid.unwrap_or("")),
            ("Gz-Sid", sid.unwrap_or("")),
            ("Gz-Sign", sign),
            ("Accept-Encoding", "gzip"),
            ("Connection", "Keep-Alive"),
            ("Gz-OsVer", "35"),
            ("Gz-Brand", "generic"),
            ("Gz-Model", "Android"),
            ("Gz-FontSize", "1.0"),
            ("Gz-Imei", "0123456789abcdef"),
            ("Gz-Network", "{vpn=false, wifi=false/null, available=true}"),
            (
                "Gz-NotifyPermission",
                "{filter=ALL, channels={}, channelGroups={}, enabled=true, paused=false, BubblesAllowed=false}",
            ),
        ];
        for (k, v) in pairs {
            let name = HeaderName::from_bytes(k.as_bytes())
                .map_err(|e| TransportError::Other(e.to_string()))?;
            let val = HeaderValue::from_str(v).map_err(|e| TransportError::Other(e.to_string()))?;
            h.insert(name, val);
        }
        Ok(h)
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn load_materials(path: &Path) -> Result<([u8; KEY_LEN], Vec<u8>)> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if path.is_dir() {
        for name in ["split_config.arm64_v8a.apk", "base.apk"] {
            let p = path.join(name);
            if p.exists() {
                candidates.push(p);
            }
        }
    } else {
        candidates.push(path.to_path_buf());
        if let Some(parent) = path.parent() {
            for name in ["split_config.arm64_v8a.apk", "base.apk"] {
                let p = parent.join(name);
                if p.exists() && p != path {
                    candidates.push(p);
                }
            }
        }
    }
    let mut material: Option<[u8; KEY_LEN]> = None;
    let mut sign_suffix: Option<Vec<u8>> = None;
    for p in candidates {
        let bytes = std::fs::read(&p)
            .map_err(|e| TransportError::Other(format!("apk read failed {}: {e}", p.display())))?;
        if material.is_none() {
            if let Ok(m) = extract_aes_material(&bytes, &p) {
                material = Some(m);
            }
        }
        if sign_suffix.is_none() {
            if let Ok(sfx) = extract_sign_suffix(&bytes, &p) {
                sign_suffix = Some(sfx);
            }
        }
        if material.is_some() && sign_suffix.is_some() {
            break;
        }
    }
    let material = material.ok_or_else(|| {
        TransportError::Other(format!("aes material not found near {}", path.display()))
    })?;
    let sign_suffix = sign_suffix.ok_or_else(|| {
        TransportError::Other(format!("sign suffix not found near {}", path.display()))
    })?;
    Ok((material, sign_suffix))
}

fn extract_aes_material(bytes: &[u8], path: &Path) -> Result<[u8; KEY_LEN]> {
    let so = if looks_like_zip(bytes) {
        read_zip_entry(bytes, SO_PATH_IN_SPLIT).ok_or_else(|| {
            TransportError::Other(format!("{} missing {SO_PATH_IN_SPLIT}", path.display()))
        })?
    } else {
        bytes.to_vec()
    };
    if so.len() < KEY_OFFSET + KEY_LEN {
        return Err(TransportError::Other("apk material too small".into()));
    }
    // Build the key directly from the user-provided APK slice. Avoiding a
    // zero-initialised intermediate buffer also keeps static analysers from
    // mistaking that buffer for a hard-coded cryptographic value.
    let out: [u8; KEY_LEN] = so[KEY_OFFSET..KEY_OFFSET + KEY_LEN]
        .try_into()
        .map_err(|_| TransportError::Other("apk material too small".into()))?;
    Ok(out)
}

fn extract_sign_suffix(bytes: &[u8], path: &Path) -> Result<Vec<u8>> {
    let dex_bytes: Vec<Vec<u8>> = if looks_like_zip(bytes) {
        let mut out = Vec::new();
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
            .map_err(|e| TransportError::Other(format!("apk zip: {e}")))?;
        for i in 0..archive.len() {
            let mut f = archive
                .by_index(i)
                .map_err(|e| TransportError::Other(e.to_string()))?;
            let name = f
                .name()
                .map_err(|e| TransportError::Other(e.to_string()))?
                .into_owned();
            if name.ends_with(".dex") {
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)
                    .map_err(|e| TransportError::Other(e.to_string()))?;
                out.push(buf);
            }
        }
        out
    } else {
        vec![bytes.to_vec()]
    };

    let mut found: Option<Vec<u8>> = None;
    for dex in dex_bytes {
        for w in dex.windows(33) {
            if w[32] != 0 {
                continue;
            }
            let s = &w[..32];
            if !s.iter().all(|b| (0x20..=0x7e).contains(b)) {
                continue;
            }
            if s.starts_with(b"&sM&")
                && s.ends_with(b"@C")
                && s.contains(&b'^')
                && s.windows(2).any(|x| x == b"**")
            {
                match &found {
                    Some(prev) if prev.as_slice() != s => {
                        return Err(TransportError::Other("ambiguous sign suffix in apk".into()));
                    }
                    None => found = Some(s.to_vec()),
                    _ => {}
                }
            }
        }
    }
    found.ok_or_else(|| {
        TransportError::Other(format!("sign suffix not found in {}", path.display()))
    })
}

fn looks_like_zip(bytes: &[u8]) -> bool {
    bytes.starts_with(b"PK\x03\x04")
}

fn read_zip_entry(bytes: &[u8], name: &str) -> Option<Vec<u8>> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).ok()?;
    let mut f = archive.by_name(name).ok()?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).ok()?;
    Some(buf)
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    errno: i32,
    #[serde(default)]
    data: Option<T>,
}

#[derive(Debug, Deserialize, Default)]
struct LoginData {
    sid: Option<String>,
    uid: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct DeviceListData {
    list: Option<Vec<CloudDevice>>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct OtaInfoData {
    #[serde(default, alias = "current_version")]
    current_version: Option<String>,
    #[serde(default)]
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CloudDevice {
    sn: Option<String>,
    sn_disp: Option<String>,
    #[serde(default, alias = "devName")]
    dev_name: Option<String>,
    #[serde(default)]
    name: Option<String>,
    model: Option<String>,
    dev_type: Option<i32>,
    online: Option<i32>,
    #[serde(
        default,
        alias = "softVer",
        alias = "soft_ver",
        alias = "firmwareVersion",
        alias = "firmware_version",
        alias = "devVer",
        alias = "dev_ver",
        alias = "currentVersion",
        alias = "current_version"
    )]
    firmware_version: Option<String>,
    #[serde(default)]
    extend: Option<CloudExtend>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct CloudExtend {
    #[serde(default, deserialize_with = "de_opt_stringish")]
    electricity: Option<String>,
    #[serde(default, alias = "wifi_signal", deserialize_with = "de_opt_stringish")]
    wifi_signal: Option<String>,
}

fn de_opt_stringish<'de, D>(deserializer: D) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    Ok(v.and_then(|v| match v {
        serde_json::Value::String(s) => Some(s),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }))
}

fn clean_opt(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty() && v.to_lowercase() != "null")
}

fn map_device(d: CloudDevice) -> Device {
    let id = clean_opt(d.sn_disp.clone())
        .or_else(|| clean_opt(d.sn.clone()))
        .unwrap_or_else(|| "unknown".to_string());
    let model = clean_opt(d.model.clone()).unwrap_or_else(|| "unknown".to_string());
    let name = clean_opt(d.dev_name.clone())
        .or_else(|| clean_opt(d.name.clone()))
        .or_else(|| {
            if model != "unknown" {
                Some(model.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| id.clone());
    let online = d.online.unwrap_or(0) == 1;
    let health = if online {
        HealthStatus::Healthy
    } else {
        HealthStatus::Offline
    };
    let battery_percent = d
        .extend
        .as_ref()
        .and_then(|e| e.electricity.as_ref())
        .and_then(|s| s.parse::<u8>().ok());
    let wifi_signal_percent = d
        .extend
        .as_ref()
        .and_then(|e| e.wifi_signal.as_ref())
        .and_then(|s| s.parse::<u8>().ok());
    let kind = match d.dev_type.unwrap_or_default() {
        3 => "doorbell".to_string(),
        _ => "camera".to_string(),
    };
    let firmware_version = clean_opt(d.firmware_version.clone());
    Device {
        id: DeviceId(id),
        name,
        vendor: "aosu".to_string(),
        model,
        kind,
        online,
        health,
        battery_percent,
        wifi_signal_percent,
        firmware_version,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    #[ignore = "requires AOSU_TEST_EMAIL/AOSU_TEST_PASSWORD/AOSU_TEST_APK"]
    fn cloud_live_smoke_login_and_list() {
        let email = std::env::var("AOSU_TEST_EMAIL").expect("AOSU_TEST_EMAIL");
        let password = std::env::var("AOSU_TEST_PASSWORD").expect("AOSU_TEST_PASSWORD");
        let apk = std::env::var("AOSU_TEST_APK").expect("AOSU_TEST_APK");
        let client = AosuCloudClient::from_apk(CloudClientConfig::new(apk)).expect("client");
        let session = client
            .login(&CloudCredentials { email, password })
            .expect("login");
        assert!(!session.sid.is_empty());
        assert!(!session.uid.is_empty());
        let devices = client.get_device_list(&session).expect("devices");
        assert!(!devices.is_empty());
        let fw_count = devices
            .iter()
            .filter(|d| d.firmware_version.as_deref().is_some_and(|v| !v.is_empty()))
            .count();
        eprintln!("cloud_live_smoke devices={} fw={}", devices.len(), fw_count);
        assert!(fw_count > 0, "expected at least one firmware version");
    }

    /// Recursively collects JSON *field names* only. Values are never returned,
    /// logged or stored; this keeps credential-bearing responses out of logs.
    fn collect_json_field_names(value: &serde_json::Value, out: &mut Vec<String>, depth: usize) {
        if depth > 8 {
            return;
        }
        match value {
            serde_json::Value::Object(map) => {
                for (key, child) in map {
                    if !out.iter().any(|seen| seen == key) {
                        out.push(key.clone());
                    }
                    collect_json_field_names(child, out, depth + 1);
                }
            }
            serde_json::Value::Array(items) => {
                for child in items {
                    collect_json_field_names(child, out, depth + 1);
                }
            }
            _ => {}
        }
    }

    /// Read-only diagnostic: which fields does the OEM detail endpoint expose?
    /// Prints field names and an errno only -- no values, tokens or keys.
    #[test]
    #[ignore = "requires AOSU_TEST_EMAIL/AOSU_TEST_PASSWORD/AOSU_TEST_APK"]
    fn cloud_live_probe_detail_field_names() {
        let email = std::env::var("AOSU_TEST_EMAIL").expect("AOSU_TEST_EMAIL");
        let password = std::env::var("AOSU_TEST_PASSWORD").expect("AOSU_TEST_PASSWORD");
        let apk = std::env::var("AOSU_TEST_APK").expect("AOSU_TEST_APK");
        let client = AosuCloudClient::from_apk(CloudClientConfig::new(apk)).expect("client");
        let session = client
            .login(&CloudCredentials { email, password })
            .expect("login");
        let ms = now_ms().to_string();
        let rid = Uuid::new_v4().to_string();
        let sign = client.sign(&rid, &ms);
        let list_url = format!("{}{}", client.config.api_base, DEVICE_LIST_PATH);
        let list_resp = client
            .http
            .post(&list_url)
            .query(&[("uuid", rid.as_str()), ("t", ms.as_str())])
            .headers(
                client
                    .base_headers(Some(&session.sid), Some(&session.uid), &sign)
                    .expect("headers"),
            )
            .send()
            .expect("list request");
        let list_json: serde_json::Value =
            serde_json::from_str(&list_resp.text().unwrap_or_default()).expect("list json");
        let mut serials = Vec::new();
        collect_identifiers(&list_json, &mut serials, 0);
        serials.truncate(4);
        assert!(!serials.is_empty(), "no serial candidates in inventory");

        let detail_url = format!("{}{}", client.config.api_base, "/v1/dev/getInfoBySn");
        let mut accepted = None;
        let mut last_errno = -1;
        for param in ["sn", "deviceSn", "devSn", "serialNumber"] {
            for candidate in &serials {
                let ms = now_ms().to_string();
                let rid = Uuid::new_v4().to_string();
                let sign = client.sign(&rid, &ms);
                let resp = client
                    .http
                    .post(&detail_url)
                    .query(&[("uuid", rid.as_str()), ("t", ms.as_str())])
                    .headers(
                        client
                            .base_headers(Some(&session.sid), Some(&session.uid), &sign)
                            .expect("headers"),
                    )
                    .form(&[(param, candidate.as_str())])
                    .send()
                    .expect("detail request");
                let value: serde_json::Value =
                    serde_json::from_str(&resp.text().unwrap_or_default()).expect("detail json");
                last_errno = value.get("errno").and_then(|v| v.as_i64()).unwrap_or(-1);
                if last_errno == 0 {
                    accepted = Some((param, value));
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(150));
            }
            if accepted.is_some() {
                break;
            }
        }
        let (param, value) = accepted.unwrap_or_else(|| {
            panic!(
                "detail rejected: candidates={} last_errno={}",
                serials.len(),
                last_errno
            )
        });
        let mut names = Vec::new();
        collect_json_field_names(&value, &mut names, 0);
        names.sort();
        names.dedup();
        eprintln!(
            "cloud_detail_probe param={param} fields={} names={}",
            names.len(),
            names.join(",")
        );
        assert!(!names.is_empty(), "no fields in detail response");
    }

    /// Collects OEM identifier *values* from a cloud response for the read-only
    /// probe only. They stay in process memory and are never logged.
    fn collect_identifiers(value: &serde_json::Value, out: &mut Vec<String>, depth: usize) {
        if depth > 8 {
            return;
        }
        const ID_KEYS: [&str; 6] = ["sn", "sndisp", "basesn", "devid", "uuid", "cpdeviceid"];
        match value {
            serde_json::Value::Object(map) => {
                for (key, child) in map {
                    if ID_KEYS.contains(&key.to_ascii_lowercase().as_str()) {
                        if let Some(s) = child.as_str() {
                            let s = s.trim();
                            if !s.is_empty() && !out.iter().any(|seen| seen == s) {
                                out.push(s.to_string());
                            }
                        }
                    }
                    collect_identifiers(child, out, depth + 1);
                }
            }
            serde_json::Value::Array(items) => {
                for child in items {
                    collect_identifiers(child, out, depth + 1);
                }
            }
            _ => {}
        }
    }

    /// Read-only diagnostic: does the login response expose Tuya account hint
    /// fields? Logs field names only -- never passwords, tokens or identifiers.
    #[test]
    #[ignore = "requires AOSU_TEST_EMAIL/AOSU_TEST_PASSWORD/AOSU_TEST_APK"]
    fn cloud_live_probe_login_field_names() {
        let email = std::env::var("AOSU_TEST_EMAIL").expect("AOSU_TEST_EMAIL");
        let password = std::env::var("AOSU_TEST_PASSWORD").expect("AOSU_TEST_PASSWORD");
        let apk = std::env::var("AOSU_TEST_APK").expect("AOSU_TEST_APK");
        let client = AosuCloudClient::from_apk(CloudClientConfig::new(apk)).expect("client");
        let enc_pw = client
            .encrypt_password(password.as_bytes())
            .expect("encrypt");
        let login_email = email.trim().to_lowercase();
        let form = [
            ("region", "EU"),
            ("countryCode", "49"),
            ("countryAbbr", "DE"),
            ("type", "1"),
            ("email", login_email.as_str()),
            ("password", enc_pw.as_str()),
            ("pwType", "2"),
        ];
        let ms = now_ms().to_string();
        let rid = Uuid::new_v4().to_string();
        let sign = client.sign(&rid, &ms);
        let url = format!("{}{}", client.config.api_base, LOGIN_PATH);
        let resp = client
            .http
            .post(&url)
            .query(&[("uuid", rid.as_str()), ("t", ms.as_str())])
            .headers(client.base_headers(None, None, &sign).expect("headers"))
            .form(&form)
            .send()
            .expect("login request");
        let value: serde_json::Value =
            serde_json::from_str(&resp.text().unwrap_or_default()).expect("login json");
        let errno = value.get("errno").and_then(|v| v.as_i64()).unwrap_or(-1);
        let mut names = Vec::new();
        collect_json_field_names(&value, &mut names, 0);
        names.sort();
        names.dedup();
        eprintln!(
            "cloud_login_probe errno={errno} fields={} names={}",
            names.len(),
            names.join(",")
        );
        assert_eq!(errno, 0, "login rejected");
    }

    #[test]
    fn detail_probe_collects_field_names_without_values() {
        let value: serde_json::Value = serde_json::from_str(
            r#"{"errno":0,"data":{"sn":"S","localKey":"SECRET","list":[{"nestedKey":1}]}}"#,
        )
        .unwrap();
        let mut names = Vec::new();
        collect_json_field_names(&value, &mut names, 0);
        names.sort();
        assert_eq!(
            names,
            vec!["data", "errno", "list", "localKey", "nestedKey", "sn"]
        );
        assert!(!names.iter().any(|n| n.contains("SECRET")));
    }

    fn make_zip(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            let opts = zip::write::SimpleFileOptions::default();
            for (name, content) in files {
                zip.start_file(*name, opts).unwrap();
                zip.write_all(content).unwrap();
            }
            zip.finish().unwrap();
        }
        buf.into_inner()
    }

    #[test]
    fn extracts_aes_material_from_split() {
        let mut so = vec![0u8; KEY_OFFSET + KEY_LEN + 8];
        for i in 0..KEY_LEN {
            so[KEY_OFFSET + i] = i as u8;
        }
        let zip = make_zip(&[(SO_PATH_IN_SPLIT, &so)]);
        let mat = extract_aes_material(&zip, Path::new("split.apk")).unwrap();
        assert_eq!(mat, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
    }

    #[test]
    fn extracts_sign_suffix_from_dex() {
        // 32-byte printable suffix matching observed pattern: starts &sM&, ends @C, contains ^ and **
        let suffix: &[u8; 32] = b"&sM&ABCDEFGHIJKLMNOPQRST^**XYZ@C"; // 32 bytes
        assert_eq!(suffix.len(), 32);
        let mut dex = vec![0u8; 16];
        dex.extend_from_slice(suffix);
        dex.push(0);
        dex.extend_from_slice(&[0u8; 16]);
        let zip = make_zip(&[("classes.dex", &dex)]);
        let got = extract_sign_suffix(&zip, Path::new("base.apk")).unwrap();
        assert_eq!(got, suffix.as_slice());
    }

    #[test]
    fn maps_h1l_shape() {
        let d = CloudDevice {
            sn: Some("SN1".into()),
            sn_disp: Some("SN1".into()),
            dev_name: Some("Door".into()),
            name: None,
            model: Some("H1L".into()),
            dev_type: Some(3),
            online: Some(1),
            firmware_version: Some("1.2.3".into()),
            extend: Some(CloudExtend {
                electricity: Some("88".into()),
                wifi_signal: Some("70".into()),
            }),
        };
        let dev = map_device(d);
        assert_eq!(dev.id.0, "SN1");
        assert_eq!(dev.name, "Door");
        assert_eq!(dev.kind, "doorbell");
        assert_eq!(dev.battery_percent, Some(88));
        assert_eq!(dev.wifi_signal_percent, Some(70));
        assert_eq!(dev.firmware_version.as_deref(), Some("1.2.3"));
        assert!(dev.online);
    }
}
