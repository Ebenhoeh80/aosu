import { useEffect, useState } from "react";
import { api } from "./api";
import { ErrorNotice } from "./ErrorNotice";
import type { DeviceActivity, EventAnalytics, StoredEvent } from "./types";

type SubTab = "timeline" | "activity" | "analytics";

const PAGE_SIZE = 20;

/// Not every transport has a persistent event store (today: only
/// "push-log" does, by design -- see docs/event-pipeline.md). This
/// component makes that limitation visible via ErrorNotice instead of
/// hiding the tab or silently showing empty/fake data.
export function EventPlatform() {
  const [subTab, setSubTab] = useState<SubTab>("timeline");
  const [events, setEvents] = useState<StoredEvent[] | null>(null);
  const [activity, setActivity] = useState<DeviceActivity[] | null>(null);
  const [analytics, setAnalytics] = useState<EventAnalytics | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(0);
  const [cameraFilter, setCameraFilter] = useState("");
  const [searchText, setSearchText] = useState("");
  const [exportError, setExportError] = useState<string | null>(null);
  const [retentionDays, setRetentionDays] = useState(30);
  const [retentionResult, setRetentionResult] = useState<string | null>(null);

  async function loadTimeline() {
    setError(null);
    try {
      const rows = await api.queryEvents({
        limit: PAGE_SIZE,
        offset: page * PAGE_SIZE,
        cameraName: cameraFilter || undefined,
        searchText: searchText || undefined,
      });
      setEvents(rows);
    } catch (e) {
      setError(String(e));
    }
  }

  async function loadActivity() {
    setError(null);
    try {
      setActivity(await api.deviceActivity());
    } catch (e) {
      setError(String(e));
    }
  }

  async function loadAnalytics() {
    setError(null);
    try {
      setAnalytics(await api.eventAnalytics());
    } catch (e) {
      setError(String(e));
    }
  }

  useEffect(() => {
    if (subTab === "timeline") loadTimeline();
    if (subTab === "activity") loadActivity();
    if (subTab === "analytics") loadAnalytics();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [subTab, page]);

  async function handleFilterSubmit(e: React.FormEvent) {
    e.preventDefault();
    setPage(0);
    await loadTimeline();
  }

  async function handleExport(format: "json" | "csv") {
    setExportError(null);
    try {
      const content = await api.exportEvents(
        { limit: 10_000, offset: 0, cameraName: cameraFilter || undefined },
        format,
      );
      const blob = new Blob([content], {
        type: format === "csv" ? "text/csv" : "application/json",
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `aosu-events-${Date.now()}.${format}`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      setExportError(String(e));
    }
  }

  async function handleRetention() {
    setRetentionResult(null);
    try {
      const deleted = await api.applyEventRetention(retentionDays);
      setRetentionResult(`${deleted} Ereignis(se) gelöscht (älter als ${retentionDays} Tage).`);
    } catch (e) {
      setRetentionResult(null);
      setError(String(e));
    }
  }

  const SUB_TABS: { key: SubTab; label: string }[] = [
    { key: "timeline", label: "Timeline" },
    { key: "activity", label: "Geräteaktivität" },
    { key: "analytics", label: "Analytics" },
  ];

  return (
    <div>
      <div role="tablist" aria-label="Event-Plattform" className="flex gap-1 mb-3">
        {SUB_TABS.map((t) => (
          <button
            key={t.key}
            role="tab"
            aria-selected={subTab === t.key}
            className={`px-2 py-1 text-sm rounded ${
              subTab === t.key ? "bg-blue-600 text-white" : "bg-gray-100"
            }`}
            onClick={() => setSubTab(t.key)}
          >
            {t.label}
          </button>
        ))}
      </div>

      {error && (
        <div className="mb-3">
          <ErrorNotice
            explanation="Event-Plattform-Daten konnten nicht geladen werden."
            probableCause={`${error} -- die persistente Ereignis-Speicherung ist aktuell nur für den Transport "push-log" implementiert.`}
            suggestedAction='In den Einstellungen auf "push-log" wechseln und einen Pfad zur JSONL-Datei angeben.'
          />
        </div>
      )}

      {subTab === "timeline" && (
        <div>
          <form onSubmit={handleFilterSubmit} className="flex flex-wrap gap-2 mb-3">
            <input
              aria-label="Nach Kamera filtern"
              placeholder="Kameraname..."
              className="border rounded px-2 py-1"
              value={cameraFilter}
              onChange={(e) => setCameraFilter(e.target.value)}
            />
            <input
              aria-label="Volltextsuche"
              placeholder="Suche..."
              className="border rounded px-2 py-1 flex-1 min-w-[150px]"
              value={searchText}
              onChange={(e) => setSearchText(e.target.value)}
            />
            <button type="submit" className="px-2 py-1 rounded bg-blue-600 text-white text-sm">
              Filtern
            </button>
          </form>

          {events && events.length === 0 && <p role="status">Keine Ereignisse gefunden.</p>}

          {events && events.length > 0 && (
            <ul aria-label="Ereignis-Timeline" className="divide-y divide-gray-100 text-sm">
              {events.map((e) => (
                <li key={e.id} className="py-2">
                  <div className="flex items-center justify-between">
                    <span className="font-medium">{e.eventType}</span>
                    <span className="text-xs text-gray-400">{e.confidence}</span>
                  </div>
                  <div className="text-xs text-gray-500">
                    {e.cameraName ?? "unbekannte Kamera"} ·{" "}
                    {new Date(e.timestampMs).toLocaleString("de-DE")}
                  </div>
                  {e.classificationNote && (
                    <div className="text-xs text-gray-400 italic">{e.classificationNote}</div>
                  )}
                </li>
              ))}
            </ul>
          )}

          <div className="flex items-center justify-between mt-3">
            <button
              className="text-sm disabled:opacity-50"
              disabled={page === 0}
              onClick={() => setPage((p) => Math.max(0, p - 1))}
            >
              ← Zurück
            </button>
            <span className="text-xs text-gray-400">Seite {page + 1}</span>
            <button
              className="text-sm disabled:opacity-50"
              disabled={!events || events.length < PAGE_SIZE}
              onClick={() => setPage((p) => p + 1)}
            >
              Weiter →
            </button>
          </div>

          <div className="mt-4 flex gap-2">
            <button
              className="px-2 py-1 text-sm rounded bg-gray-100"
              onClick={() => handleExport("json")}
            >
              Export JSON
            </button>
            <button
              className="px-2 py-1 text-sm rounded bg-gray-100"
              onClick={() => handleExport("csv")}
            >
              Export CSV
            </button>
          </div>
          {exportError && (
            <p role="alert" className="text-red-600 text-sm mt-2">
              Export fehlgeschlagen: {exportError}
            </p>
          )}

          <div className="mt-4 border-t pt-3">
            <label className="text-sm block mb-1" htmlFor="retention-days">
              Aufbewahrung: Ereignisse löschen, die älter sind als (Tage)
            </label>
            <div className="flex gap-2">
              <input
                id="retention-days"
                type="number"
                min={1}
                className="border rounded px-2 py-1 w-24"
                value={retentionDays}
                onChange={(e) => setRetentionDays(Number(e.target.value))}
              />
              <button
                className="px-2 py-1 text-sm rounded bg-gray-100"
                onClick={handleRetention}
              >
                Jetzt aufräumen
              </button>
            </div>
            {retentionResult && <p className="text-sm text-gray-600 mt-1">{retentionResult}</p>}
          </div>
        </div>
      )}

      {subTab === "activity" && activity && (
        <ul aria-label="Geräteaktivität" className="divide-y divide-gray-100 text-sm">
          {activity.length === 0 && <li className="py-2 text-gray-500">Keine Daten.</li>}
          {activity.map((a) => (
            <li key={a.cameraName} className="py-2">
              <div className="font-medium">{a.cameraName}</div>
              <div className="text-xs text-gray-500">
                {a.eventCount} Ereignis(se) · zuletzt{" "}
                {new Date(a.lastSeenMs).toLocaleString("de-DE")}
              </div>
              <div className="text-xs text-gray-400">
                Kategorien: {a.distinctEventTypes.join(", ")}
              </div>
            </li>
          ))}
        </ul>
      )}

      {subTab === "analytics" && analytics && (
        <div className="text-sm space-y-4">
          <div>
            <span className="font-medium">Heute:</span> {analytics.eventsToday} Ereignis(se)
          </div>
          <div>
            <h3 className="font-medium mb-1">Nach Kamera</h3>
            <ul>
              {analytics.byCamera.map(([name, count]) => (
                <li key={name}>
                  {name}: {count}
                </li>
              ))}
            </ul>
          </div>
          <div>
            <h3 className="font-medium mb-1">Nach Ereignistyp</h3>
            <ul>
              {analytics.byType.map(([type, count]) => (
                <li key={type}>
                  {type}: {count}
                </li>
              ))}
            </ul>
          </div>
          <div>
            <h3 className="font-medium mb-1">Verlauf (14 Tage)</h3>
            <ul aria-label="Verlauf 14 Tage" className="flex gap-1 items-end h-16">
              {analytics.dailyCountsLast14Days.map(([day, count]) => (
                <li
                  key={day}
                  title={`${day}: ${count}`}
                  className="bg-blue-400 w-3"
                  style={{ height: `${Math.min(100, count * 10 + 4)}%` }}
                />
              ))}
            </ul>
          </div>
        </div>
      )}
    </div>
  );
}
