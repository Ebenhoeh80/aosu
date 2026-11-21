import { useEffect, useState } from "react";
import { api, wasLastCallMocked } from "./api";
import type {
  AppNotification,
  CapabilityReport,
  Device,
  DeviceEvent,
} from "./types";
import { DeviceInventory } from "./DeviceInventory";
import { DeviceDetail } from "./DeviceDetail";
import { EventHistory } from "./EventHistory";
import { NotificationCenter } from "./NotificationCenter";
import { DiagnosticsCenter } from "./DiagnosticsCenter";
import { Settings } from "./Settings";
import { Onboarding, hasCompletedOnboarding } from "./Onboarding";
import { ErrorNotice } from "./ErrorNotice";

type Tab = "devices" | "events" | "notifications" | "diagnostics" | "settings";

const NAV: { key: Tab; label: string }[] = [
  { key: "devices", label: "Geräte" },
  { key: "events", label: "Ereignisse" },
  { key: "notifications", label: "Benachrichtigungen" },
  { key: "diagnostics", label: "Diagnose" },
  { key: "settings", label: "Einstellungen" },
];

export function App() {
  const [showOnboarding, setShowOnboarding] = useState(!hasCompletedOnboarding());
  const [tab, setTab] = useState<Tab>("devices");
  const [devices, setDevices] = useState<Device[]>([]);
  const [events, setEvents] = useState<DeviceEvent[]>([]);
  const [notifications, setNotifications] = useState<AppNotification[]>([]);
  const [capabilityReport, setCapabilityReport] = useState<CapabilityReport>({});
  const [activeTransport, setActiveTransport] = useState("");
  const [selectedDevice, setSelectedDevice] = useState<Device | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [isMockData, setIsMockData] = useState(false);

  async function loadAll() {
    try {
      const [d, e, n, cap, active] = await Promise.all([
        api.listDevices(),
        api.recentEvents(),
        api.listNotifications(),
        api.capabilityReport(),
        api.activeTransport(),
      ]);
      setDevices(d);
      setEvents(e);
      setNotifications(n);
      setCapabilityReport(cap);
      setActiveTransport(active);
      setIsMockData(wasLastCallMocked());
      setLoadError(null);
    } catch (e) {
      setLoadError(String(e));
    }
  }

  useEffect(() => {
    loadAll();
  }, []);

  async function handleAcknowledge(id: string) {
    await api.acknowledgeNotification(id);
    const refreshed = await api.listNotifications();
    setNotifications(refreshed);
  }

  if (showOnboarding) {
    return <Onboarding onDone={() => setShowOnboarding(false)} />;
  }

  return (
    <main className="max-w-2xl mx-auto p-4 font-sans">
      <h1 className="text-xl font-semibold mb-1">Aosu Desktop (inoffiziell)</h1>
      {isMockData && (
        <p
          role="status"
          className="text-sm bg-amber-50 border border-amber-300 text-amber-900 rounded px-2 py-1 mb-3"
        >
          Demo-Modus: Es sind nur simulierte Geräte verbunden, keine echte Hardware. Siehe
          Einstellungen für Details.
        </p>
      )}
      {loadError && (
        <div className="mb-3">
          <ErrorNotice
            explanation="Daten konnten nicht geladen werden."
            probableCause={loadError}
            suggestedAction="Erneut versuchen oder in den Einstellungen den Transport prüfen."
          />
        </div>
      )}

      <nav
        role="tablist"
        aria-label="Hauptnavigation"
        className="flex gap-2 mb-4 flex-wrap"
      >
        {NAV.map((n) => (
          <button
            key={n.key}
            role="tab"
            aria-selected={tab === n.key}
            className={`px-3 py-1 rounded ${
              tab === n.key ? "bg-blue-600 text-white" : "bg-gray-100"
            }`}
            onClick={() => {
              setTab(n.key);
              setSelectedDevice(null);
            }}
          >
            {n.label}
          </button>
        ))}
      </nav>

      {tab === "devices" &&
        (selectedDevice ? (
          <DeviceDetail
            device={selectedDevice}
            events={events}
            notifications={notifications}
            onBack={() => setSelectedDevice(null)}
          />
        ) : (
          <DeviceInventory devices={devices} onSelect={setSelectedDevice} />
        ))}

      {tab === "events" && <EventHistory events={events} />}

      {tab === "notifications" && (
        <NotificationCenter notifications={notifications} onAcknowledge={handleAcknowledge} />
      )}

      {tab === "diagnostics" && (
        <DiagnosticsCenter
          capabilityReport={capabilityReport}
          activeTransport={activeTransport}
        />
      )}

      {tab === "settings" && <Settings />}
    </main>
  );
}
