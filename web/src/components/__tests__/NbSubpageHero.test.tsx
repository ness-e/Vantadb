import { render } from "@testing-library/react";
import { screen, within } from "@testing-library/dom";
import { describe, expect, it } from "vitest";
import { NbSubpageHero } from "../NbSubpageHero";

describe("NbSubpageHero", () => {
  const defaultProps = {
    title: "Page Title",
    sub: "Optional description text",
  };

  it("renders without crashing", () => {
    const { container } = render(<NbSubpageHero {...defaultProps} />);
    expect(container).toBeInTheDocument();
  });

  it("displays the title", () => {
    render(<NbSubpageHero {...defaultProps} />);
    expect(screen.getByText("Page Title")).toBeInTheDocument();
  });

  it("displays the subtitle when provided", () => {
    render(<NbSubpageHero {...defaultProps} />);
    expect(screen.getByText("Optional description text")).toBeInTheDocument();
  });

  it("does not render subtitle when sub is omitted", () => {
    render(<NbSubpageHero title="No Sub" />);
    expect(screen.queryByText("Optional description text")).not.toBeInTheDocument();
  });

  it("renders with correct heading level", () => {
    render(<NbSubpageHero {...defaultProps} />);
    const heading = screen.getByRole("heading", { level: 1 });
    expect(heading).toHaveTextContent("Page Title");
  });

  it("accepts React nodes as title", () => {
    render(
      <NbSubpageHero
        title={
          <span>
            Rich <em>Title</em>
          </span>
        }
      />,
    );
    expect(screen.getAllByText(/Rich/)).toHaveLength(1);
    const heading = screen.getByRole("heading", { level: 1 });
    expect(within(heading).getByText("Title")).toBeInTheDocument();
  });
});
