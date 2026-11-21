import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { EventHistory } from "./EventHistory";
import type { DeviceEvent } from "./types";

const events: DeviceEvent[] = [
  { device: "cam-1", kind: "Motion", severity: "Info", atEpochSeconds: 100 },
  { device: "cam-2", kind: "LowBattery", severity: "Warning", atEpochSeconds: 200 },
  { device: "cam-1", kind: "Tamper", severity: "Critical", atEpochSeconds: 300 },
];

function timeline() {
  return screen.getByRole("list", { name: "Ereigniszeitachse" });
}

describe("EventHistory", () => {
  it("renders every event kind", () => {
    render(<EventHistory events={events} />);
    const list = within(timeline());
    expect(list.getByText("Motion")).toBeInTheDocument();
    expect(list.getByText("LowBattery")).toBeInTheDocument();
    expect(list.getByText("Tamper")).toBeInTheDocument();
  });

  it("filters by severity", async () => {
    render(<EventHistory events={events} />);
    await userEvent.selectOptions(screen.getByLabelText("Nach Schweregrad filtern"), "Critical");
    const list = within(timeline());
    expect(list.queryByText("Motion")).not.toBeInTheDocument();
    expect(list.getByText("Tamper")).toBeInTheDocument();
  });

  it("shows the most recent event first", () => {
    render(<EventHistory events={events} />);
    const items = within(timeline()).getAllByRole("listitem");
    expect(items[0]).toHaveTextContent("Tamper");
  });

  it("shows an explicit message when nothing matches", async () => {
    render(<EventHistory events={events} />);
    await userEvent.type(screen.getByLabelText("Ereignisse durchsuchen"), "nonexistent-kind");
    expect(screen.getByRole("status")).toHaveTextContent("Keine Ereignisse");
  });
});
