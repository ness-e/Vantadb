import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";

vi.mock("@tanstack/react-router", () => ({
  Link: vi.fn(({ to, children, ...props }) => (
    <a href={to} {...props}>
      {children}
    </a>
  )),
}));

import { SwissFooter } from "../SwissFooter";

describe("SwissFooter", () => {
  it("renders without crashing", () => {
    const { container } = render(<SwissFooter />);
    expect(container).toBeInTheDocument();
  });

  it("displays the VantaDB logo text", () => {
    render(<SwissFooter />);
    expect(screen.getByText("VantaDB")).toBeInTheDocument();
  });

  it("renders PRODUCT section links", () => {
    render(<SwissFooter />);
    expect(screen.getByText("PRODUCT")).toBeInTheDocument();
    expect(screen.getByText("Core Engine")).toBeInTheDocument();
    expect(screen.getByText("Architecture")).toBeInTheDocument();
    expect(screen.getByText("Integrations")).toBeInTheDocument();
    expect(screen.getByText("Pricing")).toBeInTheDocument();
  });

  it("renders RESOURCES section links", () => {
    render(<SwissFooter />);
    expect(screen.getByText("RESOURCES")).toBeInTheDocument();
    expect(screen.getByText("Documentation")).toBeInTheDocument();
    expect(screen.getByText("Use Cases")).toBeInTheDocument();
    expect(screen.getByText("Blog")).toBeInTheDocument();
  });

  it("renders COMPANY section links", () => {
    render(<SwissFooter />);
    expect(screen.getByText("COMPANY")).toBeInTheDocument();
    expect(screen.getByText("About")).toBeInTheDocument();
    expect(screen.getByText("Community")).toBeInTheDocument();
    expect(screen.getByText("Contact")).toBeInTheDocument();
  });

  it("renders copyright with current year", () => {
    render(<SwissFooter />);
    const year = new Date().getFullYear();
    expect(screen.getByText(`© ${year} VantaDB. All rights reserved.`)).toBeInTheDocument();
  });

  it("renders external social links", () => {
    render(<SwissFooter />);
    expect(screen.getByText("GitHub")).toBeInTheDocument();
    expect(screen.getByText("Twitter")).toBeInTheDocument();
    expect(screen.getByText("Discord")).toBeInTheDocument();
  });
});
