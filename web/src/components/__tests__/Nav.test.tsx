import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { NbNav } from "../NbNav";

vi.mock("@tanstack/react-router", () => ({
  Link: ({
    children,
    to,
    className,
    "aria-label": ariaLabel,
  }: {
    children: React.ReactNode;
    to: string;
    className?: string;
    "aria-label"?: string;
  }) => (
    <a href={to} className={className} aria-label={ariaLabel}>
      {children}
    </a>
  ),
  useLocation: () => ({ pathname: "/" }),
  useNavigate: () => vi.fn(),
}));

describe("NbNav", () => {
  it("renders the logo (appears in desktop header and mobile drawer)", () => {
    render(<NbNav />);
    const homeLinks = screen.getAllByLabelText("VantaDB home");
    expect(homeLinks.length).toBeGreaterThanOrEqual(1);
  });

  it("renders nav links in desktop nav", () => {
    render(<NbNav />);
    const desktopNav = document.querySelector(".nc-nav-links")!;
    expect(desktopNav).toBeInTheDocument();
    expect(desktopNav.querySelectorAll(".nc-nav-link").length).toBe(3);
    expect(desktopNav.querySelectorAll(".nc-nav-group-btn").length).toBe(3);
    expect(desktopNav.textContent).toContain("Security");
    expect(desktopNav.textContent).toContain("Use Cases");
    expect(desktopNav.textContent).toContain("Pricing");
  });

  it("renders Docs and GitHub CTAs", () => {
    render(<NbNav />);
    const docsLinks = screen.getAllByText("Docs");
    expect(docsLinks.length).toBeGreaterThanOrEqual(1);
    const ghLinks = screen.getAllByText("GitHub");
    expect(ghLinks.length).toBeGreaterThanOrEqual(1);
  });

  it("toggles mobile menu on hamburger click", () => {
    render(<NbNav />);
    const hamburger = screen.getByLabelText("Open menu");
    expect(hamburger).toBeInTheDocument();

    fireEvent.click(hamburger);
    const closeButtons = screen.getAllByLabelText("Close menu");
    expect(closeButtons.length).toBeGreaterThanOrEqual(1);

    fireEvent.click(closeButtons[0]);
    expect(screen.getByLabelText("Open menu")).toBeInTheDocument();
  });

  it("shows overlay when mobile menu is open", () => {
    render(<NbNav />);
    fireEvent.click(screen.getByLabelText("Open menu"));
    const overlay = document.querySelector(".nc-nav-overlay")!;
    expect(overlay).toBeInTheDocument();
    fireEvent.click(overlay);
    expect(screen.getByLabelText("Open menu")).toBeInTheDocument();
  });
});
