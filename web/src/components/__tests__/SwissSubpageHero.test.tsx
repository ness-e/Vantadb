import { render, screen, within } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { SwissSubpageHero } from "../SwissSubpageHero";

describe("SwissSubpageHero", () => {
  const defaultProps = {
    num: "01",
    eyebrow: "Section Label",
    title: "Page Title",
    sub: "Optional description text",
  };

  it("renders without crashing", () => {
    const { container } = render(<SwissSubpageHero {...defaultProps} />);
    expect(container).toBeInTheDocument();
  });

  it("displays the number prefix", () => {
    render(<SwissSubpageHero {...defaultProps} />);
    expect(screen.getByText("[01]")).toBeInTheDocument();
  });

  it("displays the eyebrow text", () => {
    render(<SwissSubpageHero {...defaultProps} />);
    expect(screen.getByText("Section Label")).toBeInTheDocument();
  });

  it("displays the title", () => {
    render(<SwissSubpageHero {...defaultProps} />);
    expect(screen.getByText("Page Title")).toBeInTheDocument();
  });

  it("displays the subtitle when provided", () => {
    render(<SwissSubpageHero {...defaultProps} />);
    expect(screen.getByText("Optional description text")).toBeInTheDocument();
  });

  it("does not render subtitle when sub is omitted", () => {
    render(<SwissSubpageHero num="02" eyebrow="Test" title="No Sub" />);
    expect(
      screen.queryByText("Optional description text")
    ).not.toBeInTheDocument();
  });

  it("renders with correct heading level", () => {
    render(<SwissSubpageHero {...defaultProps} />);
    const heading = screen.getByRole("heading", { level: 1 });
    expect(heading).toHaveTextContent("Page Title");
  });

  it("accepts React nodes as title", () => {
    render(
      <SwissSubpageHero
        num="03"
        eyebrow="Rich Title"
        title={<span>Rich <em>Title</em></span>}
      />
    );
    expect(screen.getByText("Rich Title")).toBeInTheDocument();
    expect(screen.getAllByText(/Rich/)).toHaveLength(2);
    const heading = screen.getByRole("heading", { level: 1 });
    expect(within(heading).getByText("Title")).toBeInTheDocument();
  });
});
