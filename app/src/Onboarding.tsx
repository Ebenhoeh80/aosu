import { useState } from "react";

const STORAGE_KEY = "aosu.onboardingCompleted";

export function hasCompletedOnboarding(): boolean {
  return localStorage.getItem(STORAGE_KEY) === "true";
}

const STEPS = [
  {
    title: "Was ist diese Anwendung?",
    body: "Ein unabhängiger, quelloffener Desktop-Client für Aosu-Kameras (Tuya-basiertes Ökosystem). Nicht mit Aosu/Glazero verbunden.",
  },
  {
    title: "Was ist aktuell simuliert?",
    body: "Alle Geräte, Ereignisse und Benachrichtigungen, die Sie hier sehen, sind Demo-Daten (Mock-Transport). Es ist keine echte Kamera verbunden, und das ist für diesen Entwicklungsstand so vorgesehen.",
  },
  {
    title: "Was würde echte Hardware voraussetzen?",
    body: "Live-Video und PTZ-Steuerung erfordern eine authentifizierte Verbindung zur lokalen Basisstation oder zur Aosu-Cloud. Beides ist aktuell aus guten Gründen nicht aktiviert -- siehe docs/adr-0002-path-reassessment.md im Projekt.",
  },
  {
    title: "Was bleibt experimentell?",
    body: "Die Transport-Registry unterstützt bereits mehrere (auch fiktive) Hersteller zur Architekturprüfung -- das ist ein Entwicklungswerkzeug, kein fertiges Produktfeature.",
  },
];

export function Onboarding({ onDone }: { onDone: () => void }) {
  const [step, setStep] = useState(0);
  const isLast = step === STEPS.length - 1;

  function finish() {
    localStorage.setItem(STORAGE_KEY, "true");
    onDone();
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Einführung"
      className="fixed inset-0 bg-black/40 flex items-center justify-center p-4 z-50"
    >
      <div className="bg-white rounded-lg shadow-lg max-w-md w-full p-5">
        <h2 className="text-lg font-semibold mb-2">{STEPS[step].title}</h2>
        <p className="text-sm text-gray-700 mb-4">{STEPS[step].body}</p>
        <div className="flex items-center justify-between">
          <span className="text-xs text-gray-400">
            Schritt {step + 1} von {STEPS.length}
          </span>
          <div className="flex gap-2">
            <button className="text-sm text-gray-500" onClick={finish}>
              Überspringen
            </button>
            {!isLast && (
              <button
                className="px-3 py-1 rounded bg-blue-600 text-white text-sm"
                onClick={() => setStep((s) => s + 1)}
              >
                Weiter
              </button>
            )}
            {isLast && (
              <button
                className="px-3 py-1 rounded bg-blue-600 text-white text-sm"
                onClick={finish}
              >
                Verstanden
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
