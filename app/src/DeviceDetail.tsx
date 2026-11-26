import { useState } from "react";
import { api } from "./api";
import { ErrorNotice } from "./ErrorNotice";
import { HealthBadge } from "./HealthBadge";
import { SeverityBadge } from "./SeverityBadge";
import type {
  AppNotification,
  AudioChannelInfo,
  Device,
  DeviceCommand,
  DeviceEvent,
  MediaStreamInfo,
  RecordingInfo,
} from "./types";

type DetailTab =
  | "overview"
  | "media"
  | "diagnostics"
  | "transport"
  | "events"
  | "notifications"
  | "metadata";

const TABS: { key: DetailTab; label: string }[] = [
  { key: "overview", label: "Übersicht" },
  { key: "media", label: "Medien & Steuerung" },
  { key: "diagnostics", label: "Diagnose" },
  { key: "transport", label: "Transport" },
  { key: "events", label: "Ereignisse" },
  { key: "notifications", label: "Benachrichtigungen" },
  { key: "metadata", label: "Metadaten" },
];

/** A field this app genuinely doesn't know yet (not "0" or "-", an
 * explicit statement) -- see product requirement "do not hide
 * uncertainty". */
function UnknownField() {
  return <span className="italic text-gray-400">unbekannt</span>;
}

export function DeviceDetail({
  device,
  events,
  notifications,
  onBack,
}: {
  device: Device;
  events: DeviceEvent[];
  notifications: AppNotification[];
  onBack: () => void;
}) {
  const [tab, setTab] = useState<DetailTab>("overview");
  const deviceEvents = events.filter((e) => e.device === device.id);
  const deviceNotifications = notifications.filter((n) => n.device === device.id);
  const [stream, setStream] = useState<MediaStreamInfo | null>(null);
  const [audio, setAudio] = useState<AudioChannelInfo | null>(null);
  const [recordings, setRecordings] = useState<RecordingInfo[] | null>(null);
  const [mediaError, setMediaError] = useState<{ action: string; message: string } | null>(null);
  const [busy, setBusy] = useState(false);

  async function runMediaAction(action: string, fn: () => Promise<void>) {
    setBusy(true);
    setMediaError(null);
    try {
      await fn();
    } catch (e) {
      setMediaError({ action, message: String(e) });
    } finally {
      setBusy(false);
    }
  }

  async function send(command: DeviceCommand) {
    await runMediaAction("Befehl senden", async () => {
      await api.sendDeviceCommand(device.id, command);
    });
  }

  return (
    <div>
      <button className="text-sm text-blue-600 mb-2" onClick={onBack}>
        ← Zurück zur Geräteliste
      </button>
      <h2 className="text-lg font-semibold">{device.name}</h2>
      <div className="mb-3">
        <HealthBadge health={device.health} />
      </div>

      <div role="tablist" aria-label="Gerätedetails" className="flex gap-1 mb-3 flex-wrap">
        {TABS.map((t) => (
          <button
            key={t.key}
            role="tab"
            aria-selected={tab === t.key}
            className={`px-2 py-1 text-sm rounded ${
              tab === t.key ? "bg-blue-600 text-white" : "bg-gray-100"
            }`}
            onClick={() => setTab(t.key)}
          >
            {t.label}
          </button>
        ))}
      </div>

      {tab === "overview" && (
        <dl className="text-sm grid grid-cols-[auto_1fr] gap-x-4 gap-y-1">
          <dt className="text-gray-500">Hersteller</dt>
          <dd>{device.vendor}</dd>
          <dt className="text-gray-500">Modell</dt>
          <dd>{device.model}</dd>
          <dt className="text-gray-500">Art</dt>
          <dd>{device.kind}</dd>
          <dt className="text-gray-500">Online</dt>
          <dd>{device.online ? "ja" : "nein"}</dd>
          <dt className="text-gray-500">Akku</dt>
          <dd>{device.batteryPercent !== null ? `${device.batteryPercent}%` : <UnknownField />}</dd>
          <dt className="text-gray-500">WLAN-Signal</dt>
          <dd>
            {device.wifiSignalPercent !== null ? `${device.wifiSignalPercent}%` : <UnknownField />}
          </dd>
          <dt className="text-gray-500">Firmware</dt>
          <dd>{device.firmwareVersion ?? <UnknownField />}</dd>
        </dl>
      )}

      {tab === "media" && (
        <div className="space-y-4 text-sm">
          {mediaError && (
            <ErrorNotice
              explanation={`${mediaError.action} fehlgeschlagen: ${mediaError.message}`}
              probableCause="Der aktive Transport unterstützt diese Funktion nicht oder die Zugangsdaten wurden noch nicht konfiguriert."
              suggestedAction="Bei tuya-cloud: eigenes IoT-Projekt verbinden, Optional go2rtc/Callback-URL prüfen und PTZ erst nach STOP-Test freischalten."
            />
          )}

          <section className="space-y-2 border rounded p-2">
            <h3 className="font-medium">Live & Gegensprechen</h3>
            <div className="flex gap-2 flex-wrap">
              <button
                className="px-2 py-1 rounded border bg-white disabled:opacity-50"
                disabled={busy}
                onClick={() =>
                  void runMediaAction("Live-Stream", async () => {
                    setStream(await api.liveStream(device.id));
                  })
                }
              >
                Live-Stream öffnen
              </button>
              <button
                className="px-2 py-1 rounded border bg-white disabled:opacity-50"
                disabled={busy}
                onClick={() =>
                  void runMediaAction("Gegensprechen", async () => {
                    setAudio(await api.openTalk(device.id));
                  })
                }
              >
                Gegensprechen öffnen
              </button>
            </div>
            {stream && (
              <p className="text-xs text-gray-600">
                {stream.protocol ?? "unbekanntes Protokoll"} – {stream.description}
                {stream.url && (
                  <>
                    {" · "}
                    <a className="text-blue-600 underline" href={stream.url} target="_blank" rel="noreferrer">
                      Stream öffnen
                    </a>
                  </>
                )}
              </p>
            )}
            {audio && (
              <p className="text-xs text-gray-600">
                {audio.protocol ?? "unbekanntes Protokoll"} – {audio.twoWay ? "Zwei-Wege-Audio" : "nur Hören"}
                {audio.url && (
                  <>
                    {" · "}
                    <a className="text-blue-600 underline" href={audio.url} target="_blank" rel="noreferrer">
                      Audio öffnen
                    </a>
                  </>
                )}
              </p>
            )}
          </section>

          <section className="space-y-2 border rounded p-2">
            <h3 className="font-medium">Cloud-Aufnahmen</h3>
            <button
              className="px-2 py-1 rounded border bg-white disabled:opacity-50"
              disabled={busy}
              onClick={() =>
                void runMediaAction("Aufnahmen laden", async () => {
                  const to = Math.floor(Date.now() / 1000);
                  setRecordings(await api.listRecordings(device.id, to - 86_400, to));
                })
              }
            >
              Aufnahmen der letzten 24 Stunden laden
            </button>
            {recordings && recordings.length === 0 && (
              <p className="text-xs text-gray-500">Keine Aufnahmen im Zeitraum.</p>
            )}
            {recordings && recordings.length > 0 && (
              <ul className="divide-y divide-gray-100">
                {recordings.map((r, i) => (
                  <li key={i} className="py-1 flex items-center justify-between gap-2">
                    <span>
                      {new Date(r.startedAtEpochSeconds * 1000).toLocaleString("de-DE")} ·{" "}
                      {r.durationSecs}s · {r.source}
                    </span>
                    {r.url && (
                      <a className="text-blue-600 underline" href={r.url} target="_blank" rel="noreferrer">
                        Abspielen
                      </a>
                    )}
                  </li>
                ))}
              </ul>
            )}
          </section>

          <section className="space-y-2 border rounded p-2">
            <h3 className="font-medium">Gerätebefehle</h3>
            <div className="flex gap-2 flex-wrap">
              <button disabled={busy} className="px-2 py-1 rounded border bg-white" onClick={() => void send({ type: "setLight", on: true })}>
                Licht an
              </button>
              <button disabled={busy} className="px-2 py-1 rounded border bg-white" onClick={() => void send({ type: "setLight", on: false })}>
                Licht aus
              </button>
              <button disabled={busy} className="px-2 py-1 rounded border bg-white" onClick={() => void send({ type: "setSiren", on: true })}>
                Sirene an
              </button>
              <button disabled={busy} className="px-2 py-1 rounded border bg-white" onClick={() => void send({ type: "setSiren", on: false })}>
                Sirene aus
              </button>
            </div>
            <div className="flex gap-2 flex-wrap" aria-label="PTZ-Steuerung">
              <button disabled={busy} className="px-2 py-1 rounded border bg-white" onClick={() => void send({ type: "ptz", pan: 0, tilt: 1, durationMs: 300 })}>↑</button>
              <button disabled={busy} className="px-2 py-1 rounded border bg-white" onClick={() => void send({ type: "ptz", pan: -1, tilt: 0, durationMs: 300 })}>←</button>
              <button disabled={busy} className="px-2 py-1 rounded border bg-white font-medium" onClick={() => void send({ type: "ptzStop" })}>STOPP</button>
              <button disabled={busy} className="px-2 py-1 rounded border bg-white" onClick={() => void send({ type: "ptz", pan: 1, tilt: 0, durationMs: 300 })}>→</button>
              <button disabled={busy} className="px-2 py-1 rounded border bg-white" onClick={() => void send({ type: "ptz", pan: 0, tilt: -1, durationMs: 300 })}>↓</button>
            </div>
            <p className="text-xs text-gray-500">
              PTZ ist serverseitig standardmäßig blockiert und erst nach einem erfolgreichen STOPP-Test freizuschalten.
            </p>
          </section>
        </div>
      )}

      {tab === "diagnostics" && (
        <div className="text-sm">
          <p className="text-gray-600 mb-2">
            Geräteweite Diagnose (Netzwerk, Fähigkeiten) finden Sie im Diagnose-Center. Hier:
            gerätespezifische Rohwerte.
          </p>
          <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1">
            <dt className="text-gray-500">Health-Status (roh)</dt>
            <dd>{device.health}</dd>
            <dt className="text-gray-500">Geräte-ID</dt>
            <dd className="font-mono">{device.id}</dd>
          </dl>
        </div>
      )}

      {tab === "transport" && (
        <p className="text-sm text-gray-600">
          Dieses Gerät wird über den aktuell aktiven Transport verwaltet (siehe Einstellungen →
          Transport). Pro-Geräte-Transport-Zuordnung ist noch nicht implementiert.
        </p>
      )}

      {tab === "events" && (
        <ul aria-label="Geräteereignisse" className="text-sm divide-y divide-gray-100">
          {deviceEvents.length === 0 && <li className="py-2 text-gray-500">Keine Ereignisse.</li>}
          {deviceEvents.map((e, i) => (
            <li key={i} className="py-2 flex items-center justify-between">
              <span>{e.kind}</span>
              <SeverityBadge severity={e.severity} />
            </li>
          ))}
        </ul>
      )}

      {tab === "notifications" && (
        <ul aria-label="Gerätebenachrichtigungen" className="text-sm divide-y divide-gray-100">
          {deviceNotifications.length === 0 && (
            <li className="py-2 text-gray-500">Keine Benachrichtigungen.</li>
          )}
          {deviceNotifications.map((n) => (
            <li key={n.id} className="py-2 flex items-center justify-between">
              <span>{n.title}</span>
              <span className="flex items-center gap-2">
                <SeverityBadge severity={n.severity} />
                {n.acknowledged ? "✓ gelesen" : "ungelesen"}
              </span>
            </li>
          ))}
        </ul>
      )}

      {tab === "metadata" && (
        <pre className="text-xs bg-gray-50 p-2 rounded overflow-auto">
          {JSON.stringify(device, null, 2)}
        </pre>
      )}
    </div>
  );
}
