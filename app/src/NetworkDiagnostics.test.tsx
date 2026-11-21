import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { NetworkDiagnostics } from "./NetworkDiagnostics";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

describe("NetworkDiagnostics", () => {
  it("shows port results after a successful run", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([
      { port: 6668, open: true },
      { port: 554, open: false },
    ]);
    render(<NetworkDiagnostics />);
    await userEvent.type(screen.getByLabelText("Host oder IP"), "192.168.1.100");
    await userEvent.click(screen.getByText("Diagnose starten"));
    expect(await screen.findByText("Port 6668: offen")).toBeInTheDocument();
    expect(screen.getByText("Port 554: geschlossen")).toBeInTheDocument();
  });

  it("shows a friendly error outside the Tauri runtime", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("no tauri context"));
    render(<NetworkDiagnostics />);
    await userEvent.type(screen.getByLabelText("Host oder IP"), "192.168.1.100");
    await userEvent.click(screen.getByText("Diagnose starten"));
    expect(await screen.findByText(/Nicht verfügbar/)).toBeInTheDocument();
  });

  it("disables the button until a host is entered", () => {
    render(<NetworkDiagnostics />);
    expect(screen.getByText("Diagnose starten")).toBeDisabled();
  });
});
