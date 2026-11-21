import { useState } from "react";
import { CapabilityPanel } from "./CapabilityPanel";
import { NetworkDiagnostics } from "./NetworkDiagnostics";
import { api } from "./api";
import type { CapabilityReport } from "./types";

export function DiagnosticsCenter({
  capabilityReport,
  activeTransport,
}: {
  capabilityReport: CapabilityReport;
  activeTransport: string;
}) {
  const [exporting, setExporting] = useState(false);
  const [exportError, setExportError] = useState<string | null>(null);

  async function handleExport() {
    setExporting(true);
    setExportError(null);
    try {
      const json = await api.exportDiagnostics();
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `aosu-diagnostics-${Date.now()}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      setExportError(String(e));
    } finally {
      setExporting(false);
    }
  }

  return (
    <div>
      <section className="mb-4">
        <h2 className="text-sm font-semibold text-gray-500 mb-1">Laufzeit</h2>
        <p className="text-sm">Aktiver Transport: {activeTransport}</p>
      </section>

      <section className="mb-4">
        <h2 className="text-sm font-semibold text-gray-500 mb-1">Fähigkeiten</h2>
        <CapabilityPanel report={capabilityReport} />
      </section>

      <section className="mb-4">
        <h2 className="text-sm font-semibold text-gray-500 mb-1">Netzwerk</h2>
        <NetworkDiagnostics />
      </section>

      <section>
        <h2 className="text-sm font-semibold text-gray-500 mb-1">Export</h2>
        <button
          className="px-3 py-1 rounded bg-blue-600 text-white disabled:opacity-50"
          disabled={exporting}
          onClick={handleExport}
        >
          {exporting ? "Exportiere..." : "Diagnosebericht exportieren (JSON)"}
        </button>
        {exportError && (
          <p role="alert" className="text-red-600 text-sm mt-2">
            Export fehlgeschlagen: {exportError}
          </p>
        )}
      </section>
    </div>
  );
}
