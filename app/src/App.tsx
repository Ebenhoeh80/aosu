import { useMemo, useState } from "react";
import { DeviceList } from "./DeviceList";
import { CapabilityPanel } from "./CapabilityPanel";
import { mockCapabilityReport, mockDevices } from "./mockData";

type Tab = "devices" | "diagnostics";

export function App() {
  const [tab, setTab] = useState<Tab>("devices");
  const devices = useMemo(() => mockDevices(), []);
  const report = useMemo(() => mockCapabilityReport(), []);

  return (
    <main className="max-w-xl mx-auto p-4 font-sans">
      <h1 className="text-xl font-semibold mb-1">Aosu Desktop (inoffiziell)</h1>
      <p className="text-sm text-gray-500 mb-4">
        Zeigt derzeit Demo-Daten (Mock-Transport). Siehe docs/architecture.md.
      </p>
      <nav className="flex gap-2 mb-4">
        <button
          className={`px-3 py-1 rounded ${tab === "devices" ? "bg-blue-600 text-white" : "bg-gray-100"}`}
          onClick={() => setTab("devices")}
        >
          Geräte
        </button>
        <button
          className={`px-3 py-1 rounded ${tab === "diagnostics" ? "bg-blue-600 text-white" : "bg-gray-100"}`}
          onClick={() => setTab("diagnostics")}
        >
          Diagnose
        </button>
      </nav>
      {tab === "devices" ? (
        <DeviceList devices={devices} />
      ) : (
        <CapabilityPanel report={report} />
      )}
    </main>
  );
}
