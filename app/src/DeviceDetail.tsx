import { useState } from "react";
import type { AppNotification, Device, DeviceEvent } from "./types";
import { HealthBadge } from "./HealthBadge";
import { SeverityBadge } from "./SeverityBadge";

type DetailTab = "overview" | "diagnostics" | "transport" | "events" | "notifications" | "metadata";

const TABS: { key: DetailTab; label: string }[] = [
  { key: "overview", label: "Übersicht" },
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
