import { useEffect, useState } from "react";
import { api } from "./api";
import { ErrorNotice } from "./ErrorNotice";

export function Settings() {
  const [transports, setTransports] = useState<string[]>([]);
  const [active, setActive] = useState<string>("");
  const [switching, setSwitching] = useState(false);
  const [error, setError] = useState<{ target: string; message: string } | null>(null);
  const [pushLogPath, setPushLogPath] = useState("aosu-events.jsonl");
  const [verboseLogging, setVerboseLogging] = useState(
    () => localStorage.getItem("aosu.verboseLogging") === "true",
  );

  useEffect(() => {
    api.listAvailableTransports().then(setTransports);
    api.activeTransport().then(setActive);
  }, []);

  async function handleSwitch(name: string) {
    setSwitching(true);
    setError(null);
    try {
      await api.switchTransport(name, undefined, name === "push-log" ? pushLogPath : undefined);
      setActive(name);
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
