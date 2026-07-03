import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { VsTable } from "../VsTable";

describe("VsTable", () => {
  const sampleRows = [
    { label: "Latency", legacy: "100ms", vanta: "<5ms" },
    { label: "Throughput", legacy: "1K ops/s", vanta: "100K ops/s" },
  ];

  it("renders without crashing", () => {
    const { container } = render(<VsTable rows={sampleRows} />);
    expect(container).toBeInTheDocument();
  });

  it("renders table headers", () => {
    render(<VsTable rows={sampleRows} />);
    expect(screen.getByText("Legacy Stack")).toBeInTheDocument();
    expect(screen.getByText("VantaDB")).toBeInTheDocument();
  });

  it("renders all row labels", () => {
    render(<VsTable rows={sampleRows} />);
    expect(screen.getByText("Latency")).toBeInTheDocument();
    expect(screen.getByText("Throughput")).toBeInTheDocument();
  });

  it("renders legacy and vanta values", () => {
    render(<VsTable rows={sampleRows} />);
    expect(screen.getByText("100ms")).toBeInTheDocument();
    expect(screen.getByText("<5ms")).toBeInTheDocument();
    expect(screen.getByText("1K ops/s")).toBeInTheDocument();
    expect(screen.getByText("100K ops/s")).toBeInTheDocument();
  });

  it("renders optional title", () => {
    render(<VsTable rows={sampleRows} title="Performance Comparison" />);
    expect(screen.getByText("Performance Comparison")).toBeInTheDocument();
  });

  it("does not render title when omitted", () => {
    render(<VsTable rows={sampleRows} />);
    expect(screen.queryByText("Performance Comparison")).not.toBeInTheDocument();
  });

  it("applies custom className", () => {
    const { container } = render(<VsTable rows={sampleRows} className="custom-class" />);
    const div = container.firstChild as HTMLElement;
    expect(div.className).toBe("custom-class");
  });

  it("renders empty rows array without crashing", () => {
    const { container } = render(<VsTable rows={[]} />);
    expect(container).toBeInTheDocument();
  });
});
