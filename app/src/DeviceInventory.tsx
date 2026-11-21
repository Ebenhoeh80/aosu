import { useMemo, useState } from "react";
import type { Device, HealthStatus } from "./types";
import { HealthBadge } from "./HealthBadge";

type SortKey = "name" | "health" | "battery";

const HEALTH_ORDER: HealthStatus[] = [
  "Offline",
  "Unsupported",
  "Degraded",
  "PartiallySupported",
  "Unknown",
  "Healthy",
];

export function DeviceInventory({
  devices,
  onSelect,
}: {
  devices: Device[];
  onSelect: (device: Device) => void;
}) {
  const [search, setSearch] = useState("");
  const [vendorFilter, setVendorFilter] = useState<string>("alle");
  const [healthFilter, setHealthFilter] = useState<string>("alle");
  const [sortKey, setSortKey] = useState<SortKey>("name");

  const vendors = useMemo(
    () => ["alle", ...Array.from(new Set(devices.map((d) => d.vendor)))],
    [devices],
  );

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    let result = devices.filter((d) => {
      if (q && !(d.name.toLowerCase().includes(q) || d.model.toLowerCase().includes(q))) {
        return false;
      }
      if (vendorFilter !== "alle" && d.vendor !== vendorFilter) return false;
      if (healthFilter !== "alle" && d.health !== healthFilter) return false;
      return true;
    });
    result = [...result].sort((a, b) => {
      if (sortKey === "name") return a.name.localeCompare(b.name);
      if (sortKey === "battery") {
        return (b.batteryPercent ?? -1) - (a.batteryPercent ?? -1);
      }
      return HEALTH_ORDER.indexOf(a.health) - HEALTH_ORDER.indexOf(b.health);
    });
    return result;
  }, [devices, search, vendorFilter, healthFilter, sortKey]);

  const grouped = useMemo(() => {
    const map = new Map<string, Device[]>();
    for (const d of filtered) {
      const list = map.get(d.vendor) ?? [];
      list.push(d);
      map.set(d.vendor, list);
    }
    return map;
  }, [filtered]);

  const healthCounts = useMemo(() => {
    const counts: Partial<Record<HealthStatus, number>> = {};
    for (const d of devices) counts[d.health] = (counts[d.health] ?? 0) + 1;
    return counts;
  }, [devices]);

  return (
    <div>
      <div className="mb-3 text-sm text-gray-600" aria-label="Statusübersicht">
        {devices.length} Geräte insgesamt
        {Object.entries(healthCounts).map(([h, c]) => (
          <span key={h} className="ml-2">
            · {c} {h}
          </span>
        ))}
      </div>
      <div className="flex flex-wrap gap-2 mb-3">
        <input
          aria-label="Geräte durchsuchen"
          placeholder="Suche nach Name oder Modell..."
          className="border rounded px-2 py-1 flex-1 min-w-[180px]"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <select
          aria-label="Nach Hersteller filtern"
          className="border rounded px-2 py-1"
          value={vendorFilter}
          onChange={(e) => setVendorFilter(e.target.value)}
        >
          {vendors.map((v) => (
            <option key={v} value={v}>
              {v}
            </option>
          ))}
        </select>
        <select
          aria-label="Nach Status filtern"
          className="border rounded px-2 py-1"
          value={healthFilter}
          onChange={(e) => setHealthFilter(e.target.value)}
        >
          <option value="alle">alle Status</option>
          {HEALTH_ORDER.map((h) => (
            <option key={h} value={h}>
              {h}
            </option>
          ))}
        </select>
        <select
          aria-label="Sortieren nach"
          className="border rounded px-2 py-1"
          value={sortKey}
          onChange={(e) => setSortKey(e.target.value as SortKey)}
        >
          <option value="name">Name</option>
          <option value="health">Status</option>
          <option value="battery">Akku</option>
        </select>
      </div>

      {filtered.length === 0 && (
        <p role="status">Keine Geräte entsprechen den aktuellen Filtern.</p>
      )}

      {Array.from(grouped.entries()).map(([vendor, list]) => (
        <section key={vendor} className="mb-4">
          <h2 className="text-sm font-semibold text-gray-500 mb-1">{vendor}</h2>
          <ul aria-label={`Geräte von ${vendor}`} className="divide-y divide-gray-200">
            {list.map((d) => (
              <li key={d.id} className="py-2">
                <button
                  className="w-full flex items-center justify-between text-left hover:bg-gray-50 rounded px-1"
                  onClick={() => onSelect(d)}
                >
                  <div>
                    <div className="font-medium">{d.name}</div>
                    <div className="text-sm text-gray-500">
                      {d.model} · {d.kind}
                      {d.firmwareVersion && <> · FW {d.firmwareVersion}</>}
                      {!d.firmwareVersion && <> · FW unbekannt</>}
                    </div>
                  </div>
                  <div className="text-sm text-right flex flex-col items-end gap-1">
                    <HealthBadge health={d.health} />
                    {d.batteryPercent !== null && <div>Akku {d.batteryPercent}%</div>}
                    {d.wifiSignalPercent !== null && <div>WLAN {d.wifiSignalPercent}%</div>}
                  </div>
                </button>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}
