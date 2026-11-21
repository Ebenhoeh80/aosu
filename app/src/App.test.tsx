import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { App } from "./App";

function skipOnboarding() {
  localStorage.setItem("aosu.onboardingCompleted", "true");
}

describe("App", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.mocked(invoke).mockReset();
    // No Tauri runtime in these tests -> every api.* call falls back to
    // mock data, exercising the actual offline-mode code path.
    vi.mocked(invoke).mockRejectedValue(new Error("no tauri context"));
  });

  it("shows onboarding on first run", () => {
    render(<App />);
    expect(screen.getByRole("dialog", { name: "Einführung" })).toBeInTheDocument();
  });

  it("does not show onboarding once completed", async () => {
    skipOnboarding();
    render(<App />);
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    await waitFor(() => expect(screen.getByText("Terasse Norden (Demo)")).toBeInTheDocument());
  });

  it("shows the demo-mode banner when data came from the mock fallback", async () => {
    skipOnboarding();
    render(<App />);
    await waitFor(() =>
      expect(screen.getByRole("status")).toHaveTextContent("Demo-Modus"),
    );
  });

  it("can drill into a device and back out", async () => {
    skipOnboarding();
    render(<App />);
    const user = userEvent.setup();
    await screen.findByText("Terasse Norden (Demo)");
    await user.click(screen.getByText("Terasse Norden (Demo)"));
    expect(screen.getByRole("tablist", { name: "Gerätedetails" })).toBeInTheDocument();
    await user.click(screen.getByText("← Zurück zur Geräteliste"));
    expect(await screen.findByText("Terasse Norden (Demo)")).toBeInTheDocument();
  });

  it("switches to the notifications tab and can acknowledge one", async () => {
    skipOnboarding();
    render(<App />);
    const user = userEvent.setup();
    await screen.findByText("Terasse Norden (Demo)");
    await user.click(screen.getByRole("tab", { name: "Benachrichtigungen" }));
    const ackButtons = await screen.findAllByText("Als gelesen markieren");
    await user.click(ackButtons[0]);
    await waitFor(() => expect(screen.getAllByText("gelesen").length).toBeGreaterThan(0));
  });

  it("switches to the diagnostics tab and shows the capability table", async () => {
    skipOnboarding();
    render(<App />);
    const user = userEvent.setup();
    await screen.findByText("Terasse Norden (Demo)");
    await user.click(screen.getByRole("tab", { name: "Diagnose" }));
    expect(await screen.findByLabelText("Fähigkeiten")).toBeInTheDocument();
  });

  it("switches to settings and shows the transport switcher", async () => {
    skipOnboarding();
    render(<App />);
    const user = userEvent.setup();
    await screen.findByText("Terasse Norden (Demo)");
    await user.click(screen.getByRole("tab", { name: "Einstellungen" }));
    expect(await screen.findByText("mock", { selector: "button" })).toBeInTheDocument();
  });
});
