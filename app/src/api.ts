import { invoke } from "@tauri-apps/api/core";
import * as mock from "./mockData";
import type {
  AppNotification,
  CapabilityReport,
  Device,
  DeviceEvent,
  DiagnosticsBundle,
  PortCheck,
} from "./types";

/// Whether the last call to any api.* function fell back to mock data
/// (no Tauri runtime reachable) -- the Offline/Onboarding banners read
/// this to tell a user "you're seeing demo data" honestly instead of
/// silently pretending it's live.
let lastCallWasMock = false;

export function wasLastCallMocked(): boolean {
  return lastCallWasMock;
}

async function callOrMock<T>(
  command: string,
  args: Record<string, unknown> | undefined,
  mockFallback: () => T,
): Promise<T> {
  try {
    const result = await invoke<T>(command, args);
    lastCallWasMock = false;
    return result;
  } catch {
    lastCallWasMock = true;
    return mockFallback();
  }
}

export const api = {
  listDevices: (): Promise<Device[]> => callOrMock("list_devices", undefined, mock.mockDevices),

  recentEvents: (): Promise<DeviceEvent[]> =>
    callOrMock("recent_events", undefined, mock.mockEvents),

  listNotifications: (): Promise<AppNotification[]> =>
    callOrMock("list_notifications", undefined, mock.mockNotifications),

  acknowledgeNotification: (id: string): Promise<void> =>
    callOrMock("acknowledge_notification", { id }, () => {
      mock.acknowledgeMockNotification(id);
    }),

  runNetworkDiagnostics: (host: string): Promise<PortCheck[]> =>
    callOrMock("run_network_diagnostics", { host }, () => []),

  listAvailableTransports: (): Promise<string[]> =>
    callOrMock("list_available_transports", undefined, mock.mockAvailableTransports),

  activeTransport: (): Promise<string> =>
    callOrMock("active_transport", undefined, () => "mock"),

  switchTransport: (name: string, host?: string): Promise<void> =>
    callOrMock("switch_transport", { name, host }, () => {}),

  exportDiagnostics: (networkHost?: string): Promise<string> =>
    callOrMock("export_diagnostics", { networkHost }, () =>
      JSON.stringify(mock.mockDiagnosticsBundle(), null, 2),
    ),

  capabilityReport: (): Promise<CapabilityReport> =>
    callOrMock("capability_report", undefined, mock.mockCapabilityReport),
};
