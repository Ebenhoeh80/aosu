import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Onboarding, hasCompletedOnboarding } from "./Onboarding";

describe("Onboarding", () => {
  it("reports not completed before anything is stored", () => {
    localStorage.clear();
    expect(hasCompletedOnboarding()).toBe(false);
  });

  it("marks onboarding completed and calls onDone when finishing the last step", async () => {
    localStorage.clear();
    const onDone = vi.fn();
    render(<Onboarding onDone={onDone} />);
    // Click "Weiter" through all but the last step.
    while (screen.queryByText("Weiter")) {
      await userEvent.click(screen.getByText("Weiter"));
    }
    await userEvent.click(screen.getByText("Verstanden"));
    expect(onDone).toHaveBeenCalled();
    expect(hasCompletedOnboarding()).toBe(true);
  });

  it("can be skipped immediately", async () => {
    localStorage.clear();
    const onDone = vi.fn();
    render(<Onboarding onDone={onDone} />);
    await userEvent.click(screen.getByText("Überspringen"));
    expect(onDone).toHaveBeenCalled();
    expect(hasCompletedOnboarding()).toBe(true);
  });
});
