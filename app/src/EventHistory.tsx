import { useMemo, useState } from "react";
import type { DeviceEvent, Severity } from "./types";
import { SeverityBadge } from "./SeverityBadge";

export function EventHistory({ events }: { events: DeviceEvent[] }) {
  const [search, setSearch] = useState("");
  const [severityFilter, setSeverityFilter] = useState<Severity | "alle">("alle");
  const [kindFilter, setKindFilter] = useState<string>("alle");

  const kinds = useMemo(
    () => ["alle", ...Array.from(new Set(events.map((e) => e.kind)))],
    [events],
  );

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    return [...events]
      .filter((e) => {
        if (severityFilter !== "alle" && e.severity !== severityFilter) return false;
        if (kindFilter !== "alle" && e.kind !== kindFilter) return false;
        if (q && !(e.kind.toLowerCase().includes(q) || e.device.toLowerCase().includes(q))) {
          return false;
        }
        return true;
      })
      .sort((a, b) => b.atEpochSeconds - a.atEpochSeconds);
  }, [events, search, severityFilter, kindFilter]);

  // Group consecutive same-kind/device events for a lightweight
  // "grouping" without a full dedicated clustering algorithm.
  const grouped: { event: DeviceEvent; count: number }[] = [];
  for (const e of filtered) {
    const last = grouped[grouped.length - 1];
    if (last && last.event.kind === e.kind && last.event.device === e.device) {
      last.count += 1;
    } else {
      grouped.push({ event: e, count: 1 });
    }
  }

  return (
    <div>
      <div className="flex flex-wrap gap-2 mb-3">
        <input
          aria-label="Ereignisse durchsuchen"
          placeholder="Suche nach Art oder Gerät..."
          className="border rounded px-2 py-1 flex-1 min-w-[180px]"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <select
          aria-label="Nach Art filtern"
          className="border rounded px-2 py-1"
          value={kindFilter}
          onChange={(e) => setKindFilter(e.target.value)}
        >
          {kinds.map((k) => (
            <option key={k} value={k}>
              {k}
            </option>
          ))}
        </select>
        <select
          aria-label="Nach Schweregrad filtern"
          className="border rounded px-2 py-1"
          value={severityFilter}
          onChange={(e) => setSeverityFilter(e.target.value as Severity | "alle")}
        >
          <option value="alle">alle Schweregrade</option>
          <option value="Info">Info</option>
          <option value="Warning">Warnung</option>
          <option value="Critical">Kritisch</option>
        </select>
      </div>

      {grouped.length === 0 && <p role="status">Keine Ereignisse gefunden.</p>}

      <ol aria-label="Ereigniszeitachse" className="relative border-l border-gray-200 pl-4">
        {grouped.map(({ event, count }, i) => (
          <li key={i} className="mb-3">
            <div className="flex items-center gap-2">
              <span className="font-medium">{event.kind}</span>
              {count > 1 && (
                <span className="text-xs text-gray-500">×{count}</span>
              )}
              <SeverityBadge severity={event.severity} />
            </div>
            <div className="text-xs text-gray-500">
              {event.device} · {new Date(event.atEpochSeconds * 1000).toLocaleString("de-DE")}
            </div>
          </li>
        ))}
      </ol>
    </div>
  );
}
