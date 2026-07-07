import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tanstack/react-router", () => ({
  createLazyRoute: () => (opts: Record<string, unknown>) => opts,
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
}));

vi.mock("@/components/NbSubpageHero", () => ({
  NbSubpageHero: ({
    pattern,
    title,
    sub,
  }: {
    pattern?: string;
    title: React.ReactNode;
    sub?: string;
  }) => (
    <div data-testid="nb-hero">
      <span data-testid="hero-num">{pattern ?? "p01"}</span>
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
    expect(screen.getByTestId("nb-hero")).toBeInTheDocument();
    expect(screen.getByTestId("hero-num")).toHaveTextContent("p11");
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

  it("renders FAQ section with questions", () => {
    expect(screen.getByText("Is VantaDB really free?")).toBeInTheDocument();
    expect(screen.getByText("Can I use VantaDB commercially?")).toBeInTheDocument();
    expect(screen.getByText("What is included in the Cloud plans?")).toBeInTheDocument();
  });
});
