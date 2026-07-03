import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tanstack/react-router", () => ({
  createLazyRoute: () => (opts: Record<string, unknown>) => opts,
  Link: ({ children, to, className }: { children: React.ReactNode; to: string; className?: string }) => (
    <a href={to} className={className}>
      {children}
    </a>
  ),
}));

vi.mock("@/components/SwissSubpageHero", () => ({
  SwissSubpageHero: ({ num, eyebrow, title, sub }: { num: string; eyebrow: string; title: React.ReactNode; sub?: string }) => (
    <div data-testid="swiss-hero">
      <span data-testid="hero-num">{num}</span>
      <span data-testid="hero-eyebrow">{eyebrow}</span>
      <div data-testid="hero-title">{title}</div>
      {sub && <p data-testid="hero-sub">{sub}</p>}
    </div>
  ),
}));

import { Route } from "../../routes/pricing.lazy";
const PricingPage = (Route as unknown as { component: React.ComponentType }).component;

describe("PricingPage", () => {
  beforeEach(() => {
    render(<PricingPage />);
  });

  it("renders the hero section", () => {
    expect(screen.getByTestId("swiss-hero")).toBeInTheDocument();
    expect(screen.getByTestId("hero-eyebrow")).toHaveTextContent("Pricing");
  });

  it("renders all four pricing tiers", () => {
    expect(screen.getAllByText("Self-Hosted").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("Cloud Pro").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("Cloud Business").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("Enterprise").length).toBeGreaterThanOrEqual(1);
  });

  it("displays pricing amounts", () => {
    expect(screen.getByText("$0")).toBeInTheDocument();
    expect(screen.getByText("$29")).toBeInTheDocument();
    expect(screen.getByText("$149")).toBeInTheDocument();
    expect(screen.getByText("Custom")).toBeInTheDocument();
  });

  it("shows CTA buttons for each tier", () => {
    expect(screen.getByText("Get Started")).toBeInTheDocument();
    expect(screen.getAllByText("Join Waitlist")).toHaveLength(2);
    expect(screen.getByText("Contact Sales")).toBeInTheDocument();
  });

  it("shows 'EARLY ACCESS' badge on Cloud Pro (featured tier)", () => {
    const badges = screen.getAllByText("EARLY ACCESS");
    expect(badges.length).toBeGreaterThanOrEqual(1);
  });

  it("renders the feature comparison section", () => {
    expect(screen.getByText(/02 \/ 03/)).toBeInTheDocument();
  });

  it("renders FAQ section with questions", () => {
    expect(screen.getByText(/03 \/ 03/)).toBeInTheDocument();
    expect(screen.getByText("Is VantaDB really free?")).toBeInTheDocument();
    expect(screen.getByText("Can I use VantaDB commercially?")).toBeInTheDocument();
    expect(screen.getByText("What is included in the Cloud plans?")).toBeInTheDocument();
  });
});
