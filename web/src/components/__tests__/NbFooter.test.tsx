import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { NbFooter } from "../NbFooter";

vi.mock("@tanstack/react-router", () => ({
  Link: ({ children, to }: { children: React.ReactNode; to: string }) => (
    <a href={to}>{children}</a>
  ),
}));

describe("NbFooter", () => {
  it("renders the logo", () => {
    render(<NbFooter />);
    expect(screen.getByText("VantaDB")).toBeInTheDocument();
  });

  it("renders tagline", () => {
    render(<NbFooter />);
    expect(screen.getByText("Embedded Cognitive Memory.")).toBeInTheDocument();
  });

  it("renders PRODUCT section links", () => {
    render(<NbFooter />);
    expect(screen.getByText("Core Engine")).toBeInTheDocument();
    expect(screen.getByText("Architecture")).toBeInTheDocument();
    expect(screen.getByText("Integrations")).toBeInTheDocument();
    expect(screen.getByText("AI Agents")).toBeInTheDocument();
    expect(screen.getByText("Local RAG")).toBeInTheDocument();
    expect(screen.getByText("Pricing")).toBeInTheDocument();
  });

  it("renders RESOURCES section links", () => {
    render(<NbFooter />);
    expect(screen.getByText("Documentation")).toBeInTheDocument();
    expect(screen.getByText("Use Cases")).toBeInTheDocument();
    expect(screen.getByText("Security")).toBeInTheDocument();
    expect(screen.getByText("Changelog")).toBeInTheDocument();
    expect(screen.getByText("Benchmarks")).toBeInTheDocument();
    expect(screen.getByText("Blog")).toBeInTheDocument();
  });

  it("renders COMPANY section links", () => {
    render(<NbFooter />);
    expect(screen.getByText("About")).toBeInTheDocument();
    expect(screen.getByText("Community")).toBeInTheDocument();
    expect(screen.getByText("Contact")).toBeInTheDocument();
  });

  it("renders bottom bar external links", () => {
    render(<NbFooter />);
    const ghLink = screen.getByText("GitHub").closest("a");
    expect(ghLink).toHaveAttribute("href", "https://github.com/ness-e/Vantadb");
    const twitterLink = screen.getByText("Twitter").closest("a");
    expect(twitterLink).toHaveAttribute("href", "https://twitter.com/vantadb");
    const discordLink = screen.getByText("Discord").closest("a");
    expect(discordLink).toHaveAttribute("href", "https://discord.gg/vantadb");
  });

  it("renders copyright with current year", () => {
    render(<NbFooter />);
    const year = new Date().getFullYear();
    expect(screen.getByText(new RegExp(String(year)))).toBeInTheDocument();
  });
});
