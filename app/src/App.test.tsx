import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import userEvent from "@testing-library/user-event";
import { App } from "./App";

describe("App", () => {
  it("shows the device list by default", () => {
    render(<App />);
    expect(screen.getByText("Terasse Norden (Demo)")).toBeInTheDocument();
  });

  it("switches to the diagnostics tab and shows the capability table", async () => {
    render(<App />);
    await userEvent.click(screen.getByText("Diagnose"));
    expect(screen.getByLabelText("Fähigkeiten")).toBeInTheDocument();
  });
});
