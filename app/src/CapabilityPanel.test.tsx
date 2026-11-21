import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { CapabilityPanel } from "./CapabilityPanel";

describe("CapabilityPanel", () => {
  it("labels an unsupported capability clearly, e.g. PTZ", () => {
    render(<CapabilityPanel report={{ ptz: { type: "unsupported" } }} />);
    expect(screen.getByText("nicht unterstützt")).toBeInTheDocument();
  });

  it("includes the blocked reason when present", () => {
    render(
      <CapabilityPanel
        report={{ liveVideo: { type: "blocked", reason: "LocalKeyRequired" } }}
      />,
    );
    expect(screen.getByText(/blockiert \(LocalKeyRequired\)/)).toBeInTheDocument();
  });
});
