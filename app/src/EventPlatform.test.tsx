import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { EventPlatform } from "./EventPlatform";

describe("EventPlatform", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    URL.createObjectURL = vi.fn(() => "blob:mock-url");
    URL.revokeObjectURL = vi.fn();
  });

  it("shows an explanatory error when the active transport has no event store", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("not supported by this transport"));
    render(<EventPlatform />);
    await waitFor(() =>
      expect(screen.getByText(/Event-Plattform-Daten konnten nicht geladen werden/)).toBeInTheDocument(),
    );
    expect(screen.getByText(/nur für den Transport "push-log"/)).toBeInTheDocument();
  });

  it("renders timeline events when the backend supports it", async () => {
    vi.mocked(invoke).mockResolvedValue([
      {
        id: "e1",
        timestampMs: 1700000000000,
        eventType: "MotionDetected",
        channel: "x",
        cameraName: "Garten",
        confidence: "low",
        classificationNote: "generic",
        rawJson: "{}",
      },
    ]);
    render(<EventPlatform />);
    expect(await screen.findByText("MotionDetected")).toBeInTheDocument();
    expect(screen.getByText(/Garten/)).toBeInTheDocument();
  });

  it("switches to device activity and shows aggregated counts", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "device_activity") {
        return [
          {
            cameraName: "Garten",
            eventCount: 3,
            distinctEventTypes: ["MotionDetected", "TamperDetected"],
            lastSeenMs: 1700000000000,
          },
        ];
      }
      return [];
    });
    render(<EventPlatform />);
    await userEvent.click(screen.getByRole("tab", { name: "Geräteaktivität" }));
    expect(await screen.findByText("Garten")).toBeInTheDocument();
    expect(screen.getByText(/3 Ereignis/)).toBeInTheDocument();
  });

  it("switches to analytics and shows events-today", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "event_analytics") {
        return {
          eventsToday: 5,
          byCamera: [["Garten", 5]],
          byType: [["MotionDetected", 5]],
          dailyCountsLast14Days: Array.from({ length: 14 }, (_, i) => [`day-${i}`, i]),
        };
      }
      return [];
    });
    render(<EventPlatform />);
    await userEvent.click(screen.getByRole("tab", { name: "Analytics" }));
    expect(await screen.findByText(/5 Ereignis/)).toBeInTheDocument();
  });

  it("triggers an export without throwing", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "export_events") return "[]";
      return [];
    });
    render(<EventPlatform />);
    await screen.findByRole("tablist", { name: "Event-Plattform" });
    await userEvent.click(screen.getByText("Export JSON"));
    await waitFor(() => expect(URL.createObjectURL).toHaveBeenCalled());
  });

  it("applies retention and shows the result", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "apply_event_retention") return 3;
      return [];
    });
    render(<EventPlatform />);
    await userEvent.click(screen.getByText("Jetzt aufräumen"));
    expect(await screen.findByText(/3 Ereignis\(se\) gelöscht/)).toBeInTheDocument();
  });
});
