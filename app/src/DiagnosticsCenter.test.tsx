import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { DiagnosticsCenter } from "./DiagnosticsCenter";

describe("DiagnosticsCenter", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockRejectedValue(new Error("no tauri context"));
    URL.createObjectURL = vi.fn(() => "blob:mock-url");
    URL.revokeObjectURL = vi.fn();
  });

  it("shows the active transport", () => {
    render(
      <DiagnosticsCenter capabilityReport={{ ptz: { type: "unsupported" } }} activeTransport="mock" />,
    );
    expect(screen.getByText(/Aktiver Transport: mock/)).toBeInTheDocument();
  });

  it("triggers a diagnostics export without throwing", async () => {
    render(
      <DiagnosticsCenter capabilityReport={{ ptz: { type: "unsupported" } }} activeTransport="mock" />,
    );
    await userEvent.click(screen.getByText(/Diagnosebericht exportieren/));
    expect(URL.createObjectURL).toHaveBeenCalled();
  });
});
