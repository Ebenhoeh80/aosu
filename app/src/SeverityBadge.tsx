import type { Severity } from "./types";

const LABELS: Record<Severity, string> = {
  Info: "Info",
  Warning: "Warnung",
  Critical: "Kritisch",
};

const STYLES: Record<Severity, string> = {
  Info: "bg-blue-100 text-blue-800",
  Warning: "bg-yellow-100 text-yellow-800",
  Critical: "bg-red-100 text-red-800",
};

export function SeverityBadge({ severity }: { severity: Severity }) {
  return (
    <span
      className={`inline-block px-2 py-0.5 rounded text-xs font-medium ${STYLES[severity] ?? STYLES.Info}`}
    >
      {LABELS[severity] ?? severity}
    </span>
  );
}
