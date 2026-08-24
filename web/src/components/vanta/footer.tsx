"use client";

import { Github, Terminal, Heart } from "lucide-react";
import { useRouter } from "next/navigation";
import { VANTA, PRODUCT } from "./vanta-data";
import { VantaLogoMark } from "./logo-mark";
import { useLanguage } from "@/lib/language-provider";

/**
 * Footer — full site navigation + centralized product data.
 * Mark logo (static, same as navbar). All links use router.push for App Router.
 * CTA band removed (now only in CtaFinal section, not duplicated in footer).
 */
export function Footer() {
  const { t, tt } = useLanguage();
  const router = useRouter();

  const nav = (path: string) => {
    router.push(path);
    if (typeof window !== "undefined") window.scrollTo({ top: 0, behavior: "auto" });
  };

  // Footer navigation groups — all 31 routes organized
  const FOOTER_GROUPS = [
    {
      title: tt("footer.colPlatform", "Platform"),
      links: [
        { label: "Core Engine", path: "/engine" },
        { label: "Architecture", path: "/architecture" },
        { label: "Benchmarks", path: "/benchmarks" },
        { label: "Playground", path: "/playground" },
        { label: "Latency", path: "/latency" },
        { label: "Storage", path: "/storage" },
      ],
    },
    {
      title: tt("footer.colSolutions", "Solutions"),
      links: [
        { label: "AI Agents", path: "/solutions/ai-agents" },
        { label: "Local RAG", path: "/solutions/local-rag" },
        { label: "IDE Tooling", path: "/solutions/ai-ide-tooling" },
        { label: "Use Cases", path: "/use-cases" },
        { label: "Integrations", path: "/integrations" },
      ],
    },
    {
      title: tt("footer.colResources", "Resources"),
      links: [
        { label: "Docs", path: "/docs" },
        { label: "Changelog", path: "/changelog" },
        { label: "Blog", path: "/blog" },
        { label: "Case Studies", path: "/case-studies" },
        { label: "Showcase", path: "/showcase" },
        { label: "Config", path: "/config" },
      ],
    },
    {
      title: tt("footer.colCompany", "Company"),
      links: [
        { label: "Why VantaDB", path: "/why-vantadb" },
        { label: "Pricing", path: "/pricing" },
        { label: "Security", path: "/security" },
        { label: "Cost", path: "/cost" },
        { label: "Maintenance", path: "/maint" },
        { label: "About", path: "/about/company" },
        { label: "Team", path: "/about/team" },
        { label: "Community", path: "/about/community" },
        { label: "Contact", path: "/about/contact" },
      ],
    },
  ];

  return (
    <footer className="mt-auto bg-black text-[#FBF9F5]">
      {/* Top border accent */}
      <div className="animated-gradient-border" />

      {/* Main footer grid */}
      <div className="mx-auto grid max-w-7xl grid-cols-2 gap-8 px-4 py-12 sm:px-6 md:grid-cols-6">
        {/* Brand + mark */}
        <div className="col-span-2 md:col-span-2">
          <div className="flex items-center gap-3">
            <span className="text-black ">
              <VantaLogoMark size={48} />
            </span>
            <div>
              <span className="font-display text-3xl uppercase text-[#FBF9F5]">
                Vanta<span className="text-[#FF5500]">DB</span>
              </span>
              <p className="font-tech text-[9px] uppercase tracking-[0.3em] text-[#FBF9F5]/50">
                {PRODUCT.versions.vantadb} · embedded rust
              </p>
            </div>
          </div>
          <p className="mt-4 font-tech text-xs leading-relaxed text-[#FBF9F5]/60">
            {tt("footer.tagline", VANTA.tagline)}
          </p>
          {/* Centralized product data badges */}
          <div className="mt-4 flex flex-wrap gap-2">
            <span className="border-2 border-[#FBF9F5]/30 px-2 py-0.5 font-tech text-[9px] uppercase tracking-wider text-[#FBF9F5]/70">
              {PRODUCT.versions.license}
            </span>
            <span className="border-2 border-[#FF5500] px-2 py-0.5 font-tech text-[9px] uppercase tracking-wider text-[#FF5500]">
              Rust {PRODUCT.versions.rust}
            </span>
            <span className="border-2 border-[#FBF9F5]/30 px-2 py-0.5 font-tech text-[9px] uppercase tracking-wider text-[#FBF9F5]/70">
              Python {PRODUCT.versions.python}
            </span>
            <span className="border-2 border-[#FBF9F5]/30 px-2 py-0.5 font-tech text-[9px] uppercase tracking-wider text-[#FBF9F5]/70">
              {PRODUCT.versions.vantadb}
            </span>
          </div>
          {/* Community links */}
          <div className="mt-4 flex gap-3">
            <a
              href={VANTA.repo}
              target="_blank"
              rel="noopener noreferrer"
              className="neon-underline inline-flex items-center gap-1.5 font-tech text-xs text-[#FBF9F5]/70 transition-colors hover:text-[#FF5500]"
            >
              <Github className="h-3.5 w-3.5" /> GitHub
            </a>
            <a
              href={VANTA.pypi}
              target="_blank"
              rel="noopener noreferrer"
              className="neon-underline inline-flex items-center gap-1.5 font-tech text-xs text-[#FBF9F5]/70 transition-colors hover:text-[#FF5500]"
            >
              <Terminal className="h-3.5 w-3.5" /> PyPI
            </a>
            <a
              href={VANTA.discord}
              target="_blank"
              rel="noopener noreferrer"
              className="neon-underline inline-flex items-center gap-1.5 font-tech text-xs text-[#FBF9F5]/70 transition-colors hover:text-[#FF5500]"
            >
              <span className="h-2 w-2 animate-flicker bg-[#FF5500]" /> Discord
            </a>
          </div>
        </div>

        {/* Navigation groups */}
        {FOOTER_GROUPS.map((group) => (
          <div key={group.title}>
            <h4 className="mb-3 font-display text-sm uppercase tracking-wider text-[#FF5500]">
              {group.title}
            </h4>
            <ul className="space-y-1.5">
              {group.links.map((link) => (
                <li key={link.path}>
                  <button
                    onClick={() => nav(link.path)}
                    className="neon-underline text-left font-tech text-[11px] text-[#FBF9F5]/60 transition-colors hover:text-[#FF5500]"
                  >
                    {link.label}
                  </button>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>

      {/* Bottom bar */}
      <div className="border-t-2 border-[#FBF9F5]/15">
        <div className="mx-auto flex max-w-7xl flex-col items-center justify-between gap-2 px-4 py-4 sm:px-6 md:flex-row">
          <p className="font-tech text-[10px] uppercase tracking-wider text-[#FBF9F5]/40">
            © {new Date().getFullYear()} {VANTA.name} · {PRODUCT.versions.license} · built by ness-e
          </p>
          <p className="flex items-center gap-1.5 font-tech text-[10px] uppercase tracking-wider text-[#FBF9F5]/40">
            <Heart className="h-3 w-3 fill-[#FF5500] text-[#FF5500]" />
            {tt("footer.forgeLine", "forged in rust · printed on cream stock")}
          </p>
        </div>
      </div>
    </footer>
  );
}
