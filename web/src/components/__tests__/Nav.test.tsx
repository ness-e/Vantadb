import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { NbNav } from "../NbNav";

vi.mock("@tanstack/react-router", () => ({
  Link: ({
    children,
    to,
    className,
  }: {
    children: React.ReactNode;
    to: string;
    className?: string;
  }) => (
    <a href={to} className={className}>
      {children}
    </a>
  ),
  useLocation: () => ({ pathname: "/" }),
}));

describe("NbNav", () => {
  it("renders the logo (appears in desktop header and mobile drawer)", () => {
    render(<NbNav />);
    const logos = screen.getAllByText("VantaDB");
    expect(logos.length).toBeGreaterThanOrEqual(1);
  });

  it("renders nav links in desktop nav", () => {
    render(<NbNav />);
    const desktopNav = document.querySelector(".nb-nav-desktop")!;
    expect(desktopNav).toBeInTheDocument();
    expect(desktopNav.querySelectorAll(".nb-nav-link").length).toBe(7);
    expect(desktopNav.textContent).toContain("Core Engine");
    expect(desktopNav.textContent).toContain("Architecture");
    expect(desktopNav.textContent).toContain("AI Agents");
    expect(desktopNav.textContent).toContain("Local RAG");
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
    const hamburger = screen.getByLabelText("Menu");
    expect(hamburger).toBeInTheDocument();

    fireEvent.click(hamburger);
    const closeButtons = screen.getAllByLabelText("Close menu");
    expect(closeButtons.length).toBe(1);

    fireEvent.click(closeButtons[0]);
    expect(screen.getByLabelText("Menu")).toBeInTheDocument();
  });

  it("shows overlay when mobile menu is open", () => {
    render(<NbNav />);
    fireEvent.click(screen.getByLabelText("Menu"));
    const overlay = document.querySelector(".nb-nav-overlay")!;
    expect(overlay).toBeInTheDocument();
    fireEvent.click(overlay);
    expect(screen.getByLabelText("Menu")).toBeInTheDocument();
  });
});
