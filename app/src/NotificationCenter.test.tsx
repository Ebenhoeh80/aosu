import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { NotificationCenter } from "./NotificationCenter";
import type { AppNotification } from "./types";

const notifications: AppNotification[] = [
  {
    id: "n1",
    device: "cam-1",
    severity: "Warning",
    title: "LowBattery",
    body: "cam-1 battery low",
    atEpochSeconds: 100,
    acknowledged: false,
  },
  {
    id: "n2",
    device: "cam-2",
    severity: "Critical",
    title: "Tamper",
    body: "cam-2 tampered",
    atEpochSeconds: 200,
    acknowledged: true,
  },
];

describe("NotificationCenter", () => {
  it("shows the unacknowledged count", () => {
    render(<NotificationCenter notifications={notifications} onAcknowledge={vi.fn()} />);
    expect(screen.getByText("1 ungelesen von 2")).toBeInTheDocument();
  });

  it("calls onAcknowledge with the right id", async () => {
    const onAcknowledge = vi.fn();
    render(<NotificationCenter notifications={notifications} onAcknowledge={onAcknowledge} />);
    await userEvent.click(screen.getByText("Als gelesen markieren"));
    expect(onAcknowledge).toHaveBeenCalledWith("n1");
  });

  it("can hide already-acknowledged notifications", async () => {
    render(<NotificationCenter notifications={notifications} onAcknowledge={vi.fn()} />);
    await userEvent.click(screen.getByLabelText("Gelesene anzeigen"));
    expect(screen.queryByText("Tamper")).not.toBeInTheDocument();
    expect(screen.getByText("LowBattery")).toBeInTheDocument();
  });

  it("filters by severity", async () => {
    render(<NotificationCenter notifications={notifications} onAcknowledge={vi.fn()} />);
    await userEvent.selectOptions(screen.getByLabelText("Nach Schweregrad filtern"), "Critical");
    expect(screen.queryByText("LowBattery")).not.toBeInTheDocument();
    expect(screen.getByText("Tamper")).toBeInTheDocument();
  });
});
