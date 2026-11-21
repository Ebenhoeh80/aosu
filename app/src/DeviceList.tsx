import type { Device } from "./types";

export function DeviceList({ devices }: { devices: Device[] }) {
  if (devices.length === 0) {
    return <p role="status">Keine Geräte gefunden.</p>;
  }
  return (
    <ul aria-label="Geräteliste" className="divide-y divide-gray-200">
      {devices.map((d) => (
        <li key={d.id} className="py-2 flex items-center justify-between">
          <div>
            <div className="font-medium">{d.name}</div>
            <div className="text-sm text-gray-500">
              {d.model} · {d.kind}
            </div>
          </div>
          <div className="text-sm text-right">
            <div>{d.online ? "online" : "offline"}</div>
            {d.batteryPercent !== null && <div>Akku {d.batteryPercent}%</div>}
            {d.wifiSignalPercent !== null && (
              <div>WLAN {d.wifiSignalPercent}%</div>
            )}
          </div>
        </li>
      ))}
    </ul>
  );
}
