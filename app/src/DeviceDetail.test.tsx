import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { api } from "./api";
import { DeviceDetail } from "./DeviceDetail";
import type { AppNotification, Device, DeviceEvent } from "./types";

vi.mock("./api", () => ({
  api: {
    liveStream: vi.fn(),
    openTalk: vi.fn(),
    listRecordings: vi.fn(),
    sendDeviceCommand: vi.fn(),
  },
}));

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
  beforeEach(() => {
    vi.mocked(api.liveStream).mockReset();
    vi.mocked(api.openTalk).mockReset();
    vi.mocked(api.listRecordings).mockReset();
    vi.mocked(api.sendDeviceCommand).mockReset();
  });

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

  it("opens a real stream from the active transport, without mock data", async () => {
    vi.mocked(api.liveStream).mockResolvedValueOnce({
      description: "Tuya Cloud HLS stream",
      url: "https://stream.example/live.m3u8",
      protocol: "hls",
      sessionId: null,
    });
    render(
      <DeviceDetail device={device} events={events} notifications={notifications} onBack={vi.fn()} />,
    );
    await userEvent.click(screen.getByRole("tab", { name: "Medien & Steuerung" }));
    await userEvent.click(screen.getByRole("button", { name: "Live-Stream öffnen" }));
    expect(api.liveStream).toHaveBeenCalledWith("cam-1");
    expect(await screen.findByRole("link", { name: "Stream öffnen" })).toHaveAttribute(
      "href",
      "https://stream.example/live.m3u8",
    );
  });

  it("shows a real error when a command is blocked", async () => {
    vi.mocked(api.sendDeviceCommand).mockRejectedValueOnce(
      new Error("not implemented yet: PTZ STOP unvalidated"),
    );
    render(
      <DeviceDetail device={device} events={events} notifications={notifications} onBack={vi.fn()} />,
    );
    await userEvent.click(screen.getByRole("tab", { name: "Medien & Steuerung" }));
    await userEvent.click(screen.getByRole("button", { name: "STOPP" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("PTZ STOP unvalidated");
  });
});
