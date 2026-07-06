import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import VantaDBLogo from "../VantaDBLogo";

describe("VantaDBLogo", () => {
  it("renders default (full variant, not inverted)", () => {
    render(<VantaDBLogo />);
    const logo = screen.getByRole("img", { name: "VantaDB" });
    expect(logo).toBeInTheDocument();
    const mark = document.querySelector(".vdb-mark");
    expect(mark).toBeInTheDocument();
  });

  it("renders mark variant", () => {
    render(<VantaDBLogo variant="mark" />);
    const logo = screen.getByRole("img", { name: "VantaDB" });
    expect(logo).toBeInTheDocument();
    expect(screen.queryByText("VantaDB")).not.toBeInTheDocument();
  });

  it("renders with custom className", () => {
    render(<VantaDBLogo className="custom-class" />);
    const logo = screen.getByRole("img", { name: "VantaDB" });
    expect(logo.className).toContain("custom-class");
  });

  it("renders with inverted prop", () => {
    const { container } = render(<VantaDBLogo inverted />);
    const wordmark = container.querySelector(".vdb-wordmark")!;
    expect(wordmark).toBeInTheDocument();
    const logoFull = container.querySelector(".vdb-logo-full")!;
    expect(logoFull.classList).toContain("vdb-logo-full--inverted");
  });

  it("renders custom aria-label for mark variant", () => {
    render(<VantaDBLogo variant="mark" aria-label="VantaDB mark" />);
    expect(screen.getByRole("img", { name: "VantaDB mark" })).toBeInTheDocument();
  });
});
