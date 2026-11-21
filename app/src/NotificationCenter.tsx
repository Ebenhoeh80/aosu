import { useMemo, useState } from "react";
import type { AppNotification, Severity } from "./types";
import { SeverityBadge } from "./SeverityBadge";

export function NotificationCenter({
  notifications,
  onAcknowledge,
}: {
  notifications: AppNotification[];
  onAcknowledge: (id: string) => void;
}) {
  const [severityFilter, setSeverityFilter] = useState<Severity | "alle">("alle");
  const [showAcknowledged, setShowAcknowledged] = useState(true);

  const filtered = useMemo(
    () =>
      [...notifications]
        .filter((n) => severityFilter === "alle" || n.severity === severityFilter)
        .filter((n) => showAcknowledged || !n.acknowledged)
        .sort((a, b) => b.atEpochSeconds - a.atEpochSeconds),
    [notifications, severityFilter, showAcknowledged],
  );

  const unacknowledgedCount = notifications.filter((n) => !n.acknowledged).length;

  return (
    <div>
      <div className="mb-3 text-sm text-gray-600">
        {unacknowledgedCount} ungelesen von {notifications.length}
      </div>
      <div className="flex flex-wrap gap-2 mb-3 items-center">
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
        <label className="text-sm flex items-center gap-1">
          <input
            type="checkbox"
            checked={showAcknowledged}
            onChange={(e) => setShowAcknowledged(e.target.checked)}
          />
          Gelesene anzeigen
        </label>
      </div>

      {filtered.length === 0 && <p role="status">Keine Benachrichtigungen.</p>}

      <ul aria-label="Benachrichtigungen" className="divide-y divide-gray-200">
        {filtered.map((n) => (
          <li key={n.id} className="py-2 flex items-center justify-between gap-2">
            <div>
              <div className="font-medium flex items-center gap-2">
                {n.title}
                <SeverityBadge severity={n.severity} />
              </div>
              <div className="text-sm text-gray-500">{n.body}</div>
              <div className="text-xs text-gray-400">
                {new Date(n.atEpochSeconds * 1000).toLocaleString("de-DE")}
              </div>
            </div>
            {n.acknowledged ? (
              <span className="text-xs text-gray-400">gelesen</span>
            ) : (
              <button
                className="text-xs px-2 py-1 rounded bg-gray-100 hover:bg-gray-200"
                onClick={() => onAcknowledge(n.id)}
              >
                Als gelesen markieren
              </button>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}
