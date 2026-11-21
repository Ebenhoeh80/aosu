import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { DeviceInventory } from "./DeviceInventory";
import type { Device } from "./types";

const devices: Device[] = [
  {
    id: "a",
    name: "Alpha Cam",
    vendor: "Aosu",
    model: "C7S",
    kind: "camera",
    online: true,
    health: "Healthy",
    batteryPercent: 90,
    wifiSignalPercent: 80,
    firmwareVersion: "1.0",
  },
  {
    id: "b",
    name: "Beta Cam",
    vendor: "Kestrel",
    model: "K2",
    kind: "camera",
    online: false,
    health: "Offline",
    batteryPercent: null,
    wifiSignalPercent: null,
    firmwareVersion: null,
  },
];

describe("DeviceInventory", () => {
  it("groups devices by vendor", () => {
    render(<DeviceInventory devices={devices} onSelect={vi.fn()} />);
    expect(screen.getByRole("heading", { name: "Aosu" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Kestrel" })).toBeInTheDocument();
  });

  it("filters by search text", async () => {
    render(<DeviceInventory devices={devices} onSelect={vi.fn()} />);
    await userEvent.type(screen.getByLabelText("Geräte durchsuchen"), "Beta");
    expect(screen.queryByText("Alpha Cam")).not.toBeInTheDocument();
    expect(screen.getByText("Beta Cam")).toBeInTheDocument();
  });

  it("filters by health status", async () => {
    render(<DeviceInventory devices={devices} onSelect={vi.fn()} />);
    await userEvent.selectOptions(screen.getByLabelText("Nach Status filtern"), "Offline");
    expect(screen.queryByText("Alpha Cam")).not.toBeInTheDocument();
    expect(screen.getByText("Beta Cam")).toBeInTheDocument();
  });

  it("shows an explicit message when no device matches the filters", async () => {
    render(<DeviceInventory devices={devices} onSelect={vi.fn()} />);
    await userEvent.type(screen.getByLabelText("Geräte durchsuchen"), "does-not-exist");
    expect(screen.getByRole("status")).toHaveTextContent("Keine Geräte");
  });

  it("calls onSelect with the clicked device", async () => {
    const onSelect = vi.fn();
    render(<DeviceInventory devices={devices} onSelect={onSelect} />);
    await userEvent.click(screen.getByText("Alpha Cam"));
    expect(onSelect).toHaveBeenCalledWith(devices[0]);
  });

  it("shows an explicit 'unbekannt' for a missing firmware version", () => {
    render(<DeviceInventory devices={devices} onSelect={vi.fn()} />);
    expect(screen.getByText(/FW unbekannt/)).toBeInTheDocument();
  });
});
