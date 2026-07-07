import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { type VsRow, VsTable } from "../VsTable";

describe("VsTable", () => {
  const sampleRows: VsRow[] = [
    { feature: "Latency", legacy: "100ms", vantadb: "<5ms" },
    { feature: "Throughput", legacy: "1K ops/s", vantadb: "100K ops/s" },
  ];

  it("renders without crashing", () => {
    const { container } = render(<VsTable rows={sampleRows} />);
    expect(container).toBeInTheDocument();
  });

  it("renders table headers", () => {
    render(<VsTable rows={sampleRows} />);
    expect(screen.getByText("Feature")).toBeInTheDocument();
    expect(screen.getByText("Legacy")).toBeInTheDocument();
    expect(screen.getByText("VantaDB")).toBeInTheDocument();
  });

  it("renders all row features", () => {
    render(<VsTable rows={sampleRows} />);
    expect(screen.getByText("Latency")).toBeInTheDocument();
    expect(screen.getByText("Throughput")).toBeInTheDocument();
  });

  it("renders legacy and vantadb values", () => {
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

  it("renders optional subtitle", () => {
    render(<VsTable rows={sampleRows} subtitle="Quick summary" />);
    expect(screen.getByText("Quick summary")).toBeInTheDocument();
  });

  it("does not render title when omitted", () => {
    render(<VsTable rows={sampleRows} />);
    expect(screen.queryByText("Performance Comparison")).not.toBeInTheDocument();
  });

  it("applies custom className", () => {
    const { container } = render(<VsTable rows={sampleRows} className="custom-class" />);
    const section = container.firstChild as HTMLElement;
    expect(section.className).toContain("custom-class");
  });

  it("renders empty rows array without crashing", () => {
    const { container } = render(<VsTable rows={[]} />);
    expect(container).toBeInTheDocument();
  });

  it("handles React nodes in legacy and vantadb", () => {
    const rows: VsRow[] = [
      { feature: "Setup", legacy: <span>Complex</span>, vantadb: <strong>Simple</strong> },
    ];
    render(<VsTable rows={rows} />);
    expect(screen.getByText("Complex")).toBeInTheDocument();
    expect(screen.getByText("Simple")).toBeInTheDocument();
  });
});
