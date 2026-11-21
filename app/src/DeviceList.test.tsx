import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { DeviceList } from "./DeviceList";
import type { Device } from "./types";

const camera: Device = {
  id: "d1",
  name: "Terasse Norden",
  model: "C7S",
  kind: "camera",
  online: true,
  batteryPercent: 81,
  wifiSignalPercent: 70,
};

describe("DeviceList", () => {
  it("renders a message when there are no devices", () => {
    render(<DeviceList devices={[]} />);
    expect(screen.getByRole("status")).toHaveTextContent("Keine Geräte");
  });

  it("renders device name, battery and wifi for each device", () => {
    render(<DeviceList devices={[camera]} />);
    expect(screen.getByText("Terasse Norden")).toBeInTheDocument();
    expect(screen.getByText("Akku 81%")).toBeInTheDocument();
    expect(screen.getByText("WLAN 70%")).toBeInTheDocument();
  });

  it("omits battery/wifi lines when null (e.g. a base station)", () => {
    const base: Device = { ...camera, id: "d2", batteryPercent: null, wifiSignalPercent: null };
    render(<DeviceList devices={[base]} />);
    expect(screen.queryByText(/Akku/)).not.toBeInTheDocument();
    expect(screen.queryByText(/WLAN/)).not.toBeInTheDocument();
  });
});
