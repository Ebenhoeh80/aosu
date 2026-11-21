import type { CapabilityReport, Support } from "./types";

function supportLabel(s: Support | undefined): string {
  if (!s) return "unbekannt";
  switch (s.type) {
    case "supported":
      return "unterstützt";
    case "blocked":
      return `blockiert (${s.reason})`;
    case "unsupported":
      return "nicht unterstützt";
    default:
      return "unbekannt";
  }
}

export function CapabilityPanel({ report }: { report: CapabilityReport }) {
  const entries = Object.entries(report) as [string, Support][];
  return (
    <table aria-label="Fähigkeiten" className="w-full text-sm">
      <tbody>
        {entries.map(([cap, support]) => (
          <tr key={cap} className="border-b border-gray-100">
            <td className="py-1 pr-4">{cap}</td>
            <td className="py-1 text-gray-600">{supportLabel(support)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
