import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { DeviceDetail } from "./DeviceDetail";
import type { AppNotification, Device, DeviceEvent } from "./types";

const device: Device = {
  id: "cam-1",
  name: "Alpha Cam",
  vendor: "Aosu",
  model: "C7S",
  kind: "camera",
  online: true,
  health: "Healthy",
  batteryPercent: null,
  wifiSignalPercent: 70,
  firmwareVersion: "1.0",
};

const events: DeviceEvent[] = [
  { device: "cam-1", kind: "Motion", severity: "Info", atEpochSeconds: 100 },
  { device: "cam-2", kind: "Motion", severity: "Info", atEpochSeconds: 100 },
];

const notifications: AppNotification[] = [
  {
    id: "n1",
    device: "cam-1",
    severity: "Warning",
    title: "LowBattery",
    body: "x",
    atEpochSeconds: 100,
    acknowledged: false,
  },
];

describe("DeviceDetail", () => {
  it("shows an explicit 'unbekannt' instead of hiding a missing battery value", () => {
    render(
      <DeviceDetail device={device} events={events} notifications={notifications} onBack={vi.fn()} />,
    );
    expect(screen.getByText("unbekannt")).toBeInTheDocument();
  });

  it("only shows events for this specific device on the events tab", async () => {
    render(
      <DeviceDetail device={device} events={events} notifications={notifications} onBack={vi.fn()} />,
    );
    await userEvent.click(screen.getByRole("tab", { name: "Ereignisse" }));
    expect(screen.getAllByText("Motion")).toHaveLength(1);
  });

  it("calls onBack", async () => {
    const onBack = vi.fn();
    render(
      <DeviceDetail device={device} events={events} notifications={notifications} onBack={onBack} />,
    );
    await userEvent.click(screen.getByText("← Zurück zur Geräteliste"));
    expect(onBack).toHaveBeenCalled();
  });
});
