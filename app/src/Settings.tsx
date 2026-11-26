import { useEffect, useState } from "react";
import { LazyStore } from "@tauri-apps/plugin-store";
import { api } from "./api";
import { ErrorNotice } from "./ErrorNotice";

const settingsStore = new LazyStore("settings.json");

export function Settings({ onTransportChanged }: { onTransportChanged?: () => void | Promise<void> }) {
  const [transports, setTransports] = useState<string[]>([]);
  const [active, setActive] = useState<string>("");
  const [switching, setSwitching] = useState(false);
  const [error, setError] = useState<{ target: string; message: string } | null>(null);
  const [pushLogPath, setPushLogPath] = useState("aosu-events.jsonl");
  const [cloudEmail, setCloudEmail] = useState("");
  const [cloudPassword, setCloudPassword] = useState("");
  const [cloudApkPath, setCloudApkPath] = useState("");
  const [tuyaClientId, setTuyaClientId] = useState("");
  const [tuyaClientSecret, setTuyaClientSecret] = useState("");
  const [tuyaRegion, setTuyaRegion] = useState("eu");
  const [tuyaUid, setTuyaUid] = useState("");
  const [tuyaGo2rtcBase, setTuyaGo2rtcBase] = useState("");
  const [tuyaGo2rtcStream, setTuyaGo2rtcStream] = useState("");
  const [tuyaCallbackBase, setTuyaCallbackBase] = useState("");
  const [tuyaPtzEnabled, setTuyaPtzEnabled] = useState(false);
  const [verboseLogging, setVerboseLogging] = useState(
    () => localStorage.getItem("aosu.verboseLogging") === "true",
  );

  useEffect(() => {
    api.listAvailableTransports().then(setTransports);
    api.activeTransport().then(setActive);
    (async () => {
      try {
        const email = await settingsStore.get<string>("cloud.email");
        const apkPath = await settingsStore.get<string>("cloud.apkPath");
        const pushLog = await settingsStore.get<string>("pushlog.path");
        if (email) setCloudEmail(email);
        if (apkPath) setCloudApkPath(apkPath);
        if (pushLog) setPushLogPath(pushLog);
        // Clean up legacy plaintext password from older builds.
        await settingsStore.delete("cloud.password");
        await settingsStore.save();
      } catch {
        // store unavailable in browser/mock mode
      }
    })();
  }, []);

  async function saveSettings() {
    try {
      await settingsStore.set("cloud.email", cloudEmail);
      await settingsStore.set("cloud.apkPath", cloudApkPath);
      // Password is intentionally not persisted to disk; keep it session-only.
      await settingsStore.delete("cloud.password");
      await settingsStore.set("pushlog.path", pushLogPath);
      await settingsStore.save();
    } catch {
      // store unavailable in browser/mock mode
    }
  }

  async function handleSwitch(name: string) {
    setSwitching(true);
    setError(null);
    try {
      await saveSettings();
      const config =
        name === "push-log"
          ? { path: pushLogPath }
          : name === "cloud"
            ? { email: cloudEmail, password: cloudPassword, apkPath: cloudApkPath }
            : name === "tuya-cloud"
              ? {
                  clientId: tuyaClientId,
                  clientSecret: tuyaClientSecret,
                  region: tuyaRegion,
                  uid: tuyaUid,
                  go2rtcBase: tuyaGo2rtcBase,
                  go2rtcStream: tuyaGo2rtcStream,
                  callbackBase: tuyaCallbackBase,
                  ptzEnabled: tuyaPtzEnabled,
                }
              : undefined;
      await api.switchTransport(name, config);
      setActive(name);
      await onTransportChanged?.();
    } catch (e) {
      setError({ target: name, message: String(e) });
    } finally {
      setSwitching(false);
    }
  }

  function toggleVerboseLogging(checked: boolean) {
    setVerboseLogging(checked);
    localStorage.setItem("aosu.verboseLogging", String(checked));
  }

  return (
    <div className="space-y-6 text-sm">
      <section>
        <h2 className="font-semibold mb-1">Anwendung</h2>
        <p className="text-gray-600">
          Version 0.1.0 (Entwicklungsstand). Siehe{" "}
          <code>docs/ROADMAP.md</code> im Projekt für den aktuellen Fortschritt.
        </p>
      </section>

      <section>
        <h2 className="font-semibold mb-1">Transport</h2>
        <p className="text-gray-600 mb-2">Aktiv: {active || "wird geladen..."}</p>
        <div className="flex gap-2 flex-wrap">
          {transports.map((t) => (
            <button
              key={t}
              disabled={switching || t === active}
              className={`px-2 py-1 rounded border ${
                t === active ? "bg-blue-600 text-white border-blue-600" : "bg-white"
              }`}
              onClick={() => handleSwitch(t)}
            >
              {t}
            </button>
          ))}
        </div>
        {error && (
          <div className="mt-2">
            <ErrorNotice
              explanation={`Transport "${error.target}" konnte nicht aktiviert werden (${error.message}).`}
              probableCause="Der Transport ist registriert, aber (noch) nicht authentifiziert -- siehe Diagnose-Center für Details."
              suggestedAction="Bei mock/kestrel-mock sollte das nie passieren; bei local/cloud ist das aktuell erwartet (siehe docs/adr-0002-path-reassessment.md)."
            />
          </div>
        )}
        {transports.includes("push-log") && (
          <div className="mt-3">
            <label className="block text-xs text-gray-500 mb-1" htmlFor="push-log-path">
              Pfad zur JSONL-Datei für „push-log" (siehe{" "}
              <code>tools/notification_bridge.py --jsonl-out</code>)
            </label>
            <input
              id="push-log-path"
              className="border rounded px-2 py-1 w-full"
              value={pushLogPath}
              onChange={(e) => setPushLogPath(e.target.value)}
            />
          </div>
        )}
        {transports.includes("cloud") && (
          <div className="mt-3 space-y-2 border rounded p-2">
            <p className="text-xs text-gray-600">
              Cloud (read-only): Login + Geräteliste. E-Mail und APK-Pfad werden lokal gespeichert; das Passwort bleibt nur für diese Sitzung im Speicher und wird nicht auf die Festplatte geschrieben.
            </p>
            <label className="block text-xs text-gray-500" htmlFor="cloud-email">Aosu E-Mail</label>
            <input id="cloud-email" className="border rounded px-2 py-1 w-full" value={cloudEmail} onChange={(e) => setCloudEmail(e.target.value)} autoComplete="off" />
            <label className="block text-xs text-gray-500" htmlFor="cloud-password">Aosu Passwort (nur Sitzung)</label>
            <input id="cloud-password" type="password" className="border rounded px-2 py-1 w-full" value={cloudPassword} onChange={(e) => setCloudPassword(e.target.value)} autoComplete="off" />
            <label className="block text-xs text-gray-500" htmlFor="cloud-apk-path">Pfad zu Aosu APK/Split (base.apk oder split_config.arm64_v8a.apk)</label>
            <input id="cloud-apk-path" className="border rounded px-2 py-1 w-full" value={cloudApkPath} onChange={(e) => setCloudApkPath(e.target.value)} placeholder="C:\\Pfad\\base.apk" />
            <div className="flex gap-2 items-center">
              <button className="px-2 py-1 rounded border bg-white" onClick={() => void saveSettings()} disabled={switching}>
                Einstellungen speichern
              </button>
              <span className="text-xs text-gray-500">E-Mail/APK-Pfad werden gespeichert; Passwort bei Neustart neu eingeben.</span>
            </div>
          </div>
        )}
        {transports.includes("tuya-cloud") && (
          <div className="mt-3 space-y-2 border rounded p-2">
            <p className="text-xs text-gray-600">
              Offizieller Tuya-Cloud-Adapter: eigenes IoT-Projekt, „Link App Account“ und
              dokumentierte OpenAPI-Endpunkte. Keine Secrets aus der AOSU-APK. Alle Felder
              bleiben nur für diese Sitzung im Speicher und werden nicht gespeichert.
            </p>
            <label className="block text-xs text-gray-500" htmlFor="tuya-client-id">Client ID</label>
            <input id="tuya-client-id" className="border rounded px-2 py-1 w-full" value={tuyaClientId} onChange={(e) => setTuyaClientId(e.target.value)} autoComplete="off" />
            <label className="block text-xs text-gray-500" htmlFor="tuya-client-secret">Client Secret (nur Sitzung)</label>
            <input id="tuya-client-secret" type="password" className="border rounded px-2 py-1 w-full" value={tuyaClientSecret} onChange={(e) => setTuyaClientSecret(e.target.value)} autoComplete="off" />
            <label className="block text-xs text-gray-500" htmlFor="tuya-region">Region (eu/us/cn/in/sg)</label>
            <select id="tuya-region" className="border rounded px-2 py-1 w-full" value={tuyaRegion} onChange={(e) => setTuyaRegion(e.target.value)}>
              {["eu", "us", "cn", "in", "sg"].map((r) => <option key={r} value={r}>{r}</option>)}
            </select>
            <label className="block text-xs text-gray-500" htmlFor="tuya-uid">UID (optional, sonst aus dem Token)</label>
            <input id="tuya-uid" className="border rounded px-2 py-1 w-full" value={tuyaUid} onChange={(e) => setTuyaUid(e.target.value)} autoComplete="off" />
            <label className="block text-xs text-gray-500" htmlFor="tuya-go2rtc">go2rtc-Basis-URL (optional, für Gegensprechen)</label>
            <input id="tuya-go2rtc" className="border rounded px-2 py-1 w-full" value={tuyaGo2rtcBase} onChange={(e) => setTuyaGo2rtcBase(e.target.value)} placeholder="http://127.0.0.1:1984" />
            <label className="block text-xs text-gray-500" htmlFor="tuya-go2rtc-stream">Name des go2rtc-Streams (optional). Die <code>tuya://</code>-Quelle mit den IoT-Zugangsdaten legst du selbst in go2rtc an; hier kommt nur der Streamname hin.</label>
            <input id="tuya-go2rtc-stream" className="border rounded px-2 py-1 w-full" value={tuyaGo2rtcStream} onChange={(e) => setTuyaGo2rtcStream(e.target.value)} placeholder="tuya_webrtc" autoComplete="off" />
            <label className="block text-xs text-gray-500" htmlFor="tuya-callback">Öffentliche Callback-Basis-URL für Cloud-Aufnahmen (optional)</label>
            <input id="tuya-callback" className="border rounded px-2 py-1 w-full" value={tuyaCallbackBase} onChange={(e) => setTuyaCallbackBase(e.target.value)} placeholder="https://example.org/aosu-key" />
            <label className="flex items-center gap-2 text-xs text-gray-600">
              <input type="checkbox" checked={tuyaPtzEnabled} onChange={(e) => setTuyaPtzEnabled(e.target.checked)} />
              PTZ freischalten (erst nach erfolgreichem STOP-Test)
            </label>
            <p className="text-xs text-gray-500">
              Ohne Callback-URL bleiben Aufnahmen blockiert. Gegensprechen benötigt Basis-URL
              und Streamnamen einer lokal konfigurierten go2rtc-Quelle. Fähigkeiten werden erst
              nach einem erfolgreichen Aufruf als verfügbar gemeldet. PTZ bleibt bis zum STOP-Test
              gesperrt.
            </p>
          </div>
        )}
      </section>

      <section>
        <h2 className="font-semibold mb-1">Protokollierung</h2>
        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={verboseLogging}
            onChange={(e) => toggleVerboseLogging(e.target.checked)}
          />
          Ausführliche Protokollierung (nur Anzeige-Einstellung, wirkt sich noch nicht auf echtes
          Logging aus)
        </label>
      </section>

      <section>
        <h2 className="font-semibold mb-1">Diagnose</h2>
        <p className="text-gray-600">
          Export-Verhalten siehe Diagnose-Center. Es werden nie Zugangsdaten oder Schlüssel in
          einen Export aufgenommen (siehe <code>NEVER_LOG_FIELDS</code> im Rust-Kern).
        </p>
      </section>

      <section>
        <h2 className="font-semibold mb-1">Updates</h2>
        <p className="text-gray-500 italic">
          Noch nicht implementiert -- es gibt noch keinen Veröffentlichungskanal.
        </p>
      </section>

      <section>
        <h2 className="font-semibold mb-1">Entwickler</h2>
        <p className="text-gray-600">
          Registrierte Transporte: {transports.join(", ") || "lade..."}
        </p>
      </section>
    </div>
  );
}
