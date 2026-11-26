import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { Settings } from "./Settings";

describe("Settings", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockRejectedValue(new Error("no tauri context"));
  });

  it("lists the registered transports and marks the active one", async () => {
    render(<Settings />);
    expect(await screen.findByText("mock", { selector: "button" })).toBeInTheDocument();
    expect(await screen.findByText("kestrel-mock")).toBeInTheDocument();
  });

  it("persists the verbose-logging toggle to localStorage", async () => {
    render(<Settings />);
    await screen.findByText("mock", { selector: "button" });
    const checkbox = screen.getByLabelText(/Ausführliche Protokollierung/);
    await userEvent.click(checkbox);
    expect(localStorage.getItem("aosu.verboseLogging")).toBe("true");
  });

  it("marks the updates section as not implemented rather than hiding it", () => {
    render(<Settings />);
    expect(screen.getByText(/Noch nicht implementiert/)).toBeInTheDocument();
  });

  it("passes Tuya credentials only to the session and never persists them", async () => {
    render(<Settings />);
    const button = await screen.findByText("tuya-cloud", { selector: "button" });
    await userEvent.type(screen.getByLabelText("Client ID"), "cid-only-in-ram");
    await userEvent.type(
      screen.getByLabelText("Client Secret (nur Sitzung)"),
      "secret-only-in-ram",
    );
    await userEvent.click(button);

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("switch_transport", {
        name: "tuya-cloud",
        config: expect.objectContaining({
          clientId: "cid-only-in-ram",
          clientSecret: "secret-only-in-ram",
        }),
      }),
    );
    expect(JSON.stringify(Object.entries(localStorage))).not.toContain("secret-only-in-ram");
  });
});
