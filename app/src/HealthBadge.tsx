import type { HealthStatus } from "./types";

const LABELS: Record<HealthStatus, string> = {
  Healthy: "Gesund",
  Degraded: "Beeinträchtigt",
  Offline: "Offline",
  Unsupported: "Nicht unterstützt",
  PartiallySupported: "Teilweise unterstützt",
  Unknown: "Unbekannt",
};

const STYLES: Record<HealthStatus, string> = {
  Healthy: "bg-green-100 text-green-800",
  Degraded: "bg-yellow-100 text-yellow-800",
  Offline: "bg-gray-200 text-gray-700",
  Unsupported: "bg-red-100 text-red-800",
  PartiallySupported: "bg-orange-100 text-orange-800",
  Unknown: "bg-gray-100 text-gray-500",
};

export function HealthBadge({ health }: { health: HealthStatus }) {
  return (
    <span
      className={`inline-block px-2 py-0.5 rounded text-xs font-medium ${STYLES[health] ?? STYLES.Unknown}`}
    >
      {LABELS[health] ?? health}
    </span>
  );
}
