import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface PortCheck {
  port: number;
  open: boolean;
}

export function NetworkDiagnostics() {
  const [host, setHost] = useState("");
  const [results, setResults] = useState<PortCheck[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [running, setRunning] = useState(false);

  async function run() {
    setRunning(true);
    setError(null);
    setResults(null);
    try {
      const r = await invoke<PortCheck[]>("run_network_diagnostics", { host });
      setResults(r);
    } catch (e) {
      setError(
        "Nicht verfügbar (läuft diese Seite außerhalb der Desktop-App?): " +
          String(e),
      );
    } finally {
      setRunning(false);
    }
  }

  return (
    <div>
      <p className="text-sm text-gray-500 mb-2">
        Rein lesender Port-Scan (keine Anmeldedaten, keine Steuerbefehle). Siehe
        tools/lan_probe.py für die Hintergrundmessung.
      </p>
      <div className="flex gap-2 mb-3">
        <input
          aria-label="Host oder IP"
          className="border rounded px-2 py-1 flex-1"
          placeholder="z. B. 192.168.1.100"
          value={host}
          onChange={(e) => setHost(e.target.value)}
        />
        <button
          className="px-3 py-1 rounded bg-blue-600 text-white disabled:opacity-50"
          disabled={!host || running}
          onClick={run}
        >
          {running ? "Prüfe..." : "Diagnose starten"}
        </button>
      </div>
      {error && <p className="text-red-600 text-sm">{error}</p>}
      {results && (
        <ul aria-label="Port-Scan-Ergebnisse" className="text-sm">
          {results.map((r) => (
            <li key={r.port}>
              Port {r.port}: {r.open ? "offen" : "geschlossen"}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
