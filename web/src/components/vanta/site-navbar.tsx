"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import { useRouter, usePathname } from "next/navigation";
import {
  Github,
  Menu,
  X,
  ChevronDown,
  Search,
} from "lucide-react";
import type { View } from "./vanta-data";
import { VANTA } from "./vanta-data";
import { LangToggle } from "./lang-toggle";
import { useLanguage } from "@/lib/language-provider";
import { isLiveRoute } from "@/hooks/use-vanta-navigate";
import { toast } from "./toast";
import { VantaLogoMark } from "./logo-mark";
import { cn } from "@/lib/utils";

// ── Marquee items (technical strings, not i18n) — personality strip ──
const MARQUEE_ITEMS = [
  "pip install vantadb-py",
  "BM25 + HNSW via RRF",
  "WAL · CRC32C checksums",
  "1.2ms HNSW p50 · 10K",
  "Apache 2.0",
  "Rust 1.94.1+",
  "Zero network",
  "Local-first",
] as const;

// ── Navigation structure (4 groups + flat links) ──
interface NavItem {
  labelKey: string;
  path: string;
  view?: View;
  descKey?: string;
}

interface NavGroup {
  labelKey: string;
  items: NavItem[];
}

const NAV_GROUPS: NavGroup[] = [
  {
    labelKey: "nav.platform",
    items: [
      { labelKey: "nav.coreEngine", path: "/engine", descKey: "nav.coreEngineDesc" },
      { labelKey: "nav.architecture", path: "/architecture", descKey: "nav.architectureDesc" },
      { labelKey: "nav.benchmarks", path: "/benchmarks", view: "benchmarks", descKey: "nav.benchmarksDesc" },
    ],
  },
  {
    labelKey: "nav.solutions",
    items: [
      { labelKey: "nav.aiAgents", path: "/solutions/ai-agents", descKey: "nav.aiAgentsDesc" },
      { labelKey: "nav.localRag", path: "/solutions/local-rag", descKey: "nav.localRagDesc" },
      { labelKey: "nav.aiIdeTooling", path: "/solutions/ai-ide-tooling", descKey: "nav.aiIdeToolingDesc" },
    ],
  },
  {
    labelKey: "nav.resources",
    items: [
      { labelKey: "nav.whyVantadb", path: "/why-vantadb" },
      { labelKey: "nav.playground", path: "/playground" },
      { labelKey: "nav.docs", path: "/docs", view: "docs" },
      { labelKey: "nav.changelog", path: "/changelog" },
      { labelKey: "nav.caseStudies", path: "/case-studies" },
      { labelKey: "nav.blog", path: "/blog" },
    ],
  },
  {
    labelKey: "nav.about",
    items: [
      { labelKey: "nav.company", path: "/about/company" },
      { labelKey: "nav.team", path: "/about/team" },
      { labelKey: "nav.community", path: "/about/community" },
      { labelKey: "nav.contact", path: "/about/contact" },
    ],
  },
];

const FLAT_LINKS: NavItem[] = [
  { labelKey: "nav.security", path: "/security" },
  { labelKey: "nav.pricing", path: "/pricing" },
];

// ── Helper: open command palette ──
function openCommandPalette() {
  if (typeof window === "undefined") return;
  window.dispatchEvent(
    new KeyboardEvent("keydown", { key: "k", metaKey: true, bubbles: true })
  );
}

// ═══════════════════════════════════════════════════════
// Desktop Dropdown
// ═══════════════════════════════════════════════════════
function DesktopDropdown({
  group,
  t,
  open,
  onToggle,
  onMouseEnter,
  onMouseLeave,
  onItemSelect,
  isActiveItem,
}: {
  group: NavGroup;
  t: (key: string) => string;
  open: boolean;
  onToggle: () => void;
  onMouseEnter: () => void;
  onMouseLeave: () => void;
  onItemSelect: (item: NavItem) => void;
  isActiveItem: (item: NavItem) => boolean;
}) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    function handler(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        onToggle();
      }
    }
    document.addEventListener("mousedown", handler, { passive: true });
    return () => document.removeEventListener("mousedown", handler);
  }, [open, onToggle]);

  return (
    <div
      ref={ref}
      className="relative"
      onMouseEnter={onMouseEnter}
      onMouseLeave={onMouseLeave}
    >
      <button
        type="button"
        onClick={onToggle}
        className={cn(
          "group flex items-center gap-1.5 border-4 border-black px-3 py-2 font-tech text-[11px] font-bold uppercase tracking-wider transition-all",
          open
            ? "bg-black text-[#FBF9F5] shadow-[4px_4px_0_0_#FF5500]   "
            : "bg-[#FBF9F5] text-black shadow-[4px_4px_0_0_#000] hover:translate-x-[2px] hover:translate-y-[2px] hover:shadow-[2px_2px_0_0_#000]     "
        )}
        aria-haspopup="true"
        aria-expanded={open}
      >
        {t(group.labelKey)}
        <ChevronDown
          className={cn(
            "h-3 w-3 transition-transform duration-200",
            open && "rotate-180"
          )}
        />
      </button>

      {/* Dropdown panel — clean, minimal border, neon accent on hover */}
      {open && (
        <div
          className="absolute left-0 top-full z-50 mt-1 min-w-[220px] border-4 border-black bg-[#FBF9F5] shadow-[6px_6px_0_0_#000]   "
          role="menu"
        >
          {group.items.map((item) => (
            <button
              key={item.path}
              type="button"
              role="menuitem"
              onClick={() => onItemSelect(item)}
              className={cn(
                "flex w-full flex-col items-start gap-0.5 px-4 py-2.5 text-left transition-colors",
                isActiveItem(item)
                  ? "bg-[#FF5500] text-black"
                  : "hover:bg-[#F2EDE2] "
              )}
            >
              <span className="font-tech text-xs font-bold uppercase tracking-wider">
                {t(item.labelKey)}
              </span>
              {item.descKey && (
                <span
                  className={cn(
                    "font-tech text-[10px] normal-case tracking-normal",
                    isActiveItem(item)
                      ? "text-black/70"
                      : "text-black/70 "
                  )}
                >
                  {t(item.descKey)}
                </span>
              )}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

// ═══════════════════════════════════════════════════════
// SiteNavbar — redesigned, cleaner, professional
// ═══════════════════════════════════════════════════════
export function SiteNavbar({
  onNavigate,
  extraActions,
}: {
  onNavigate: (v: View) => void;
  extraActions?: React.ReactNode;
}) {
  const { t, tt } = useLanguage();
  const router = useRouter();
  const pathname = usePathname();

  const [scrolled, setScrolled] = useState(false);
  const [mobileOpen, setMobileOpen] = useState(false);
  const [openGroup, setOpenGroup] = useState<string | null>(null);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 8);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape" && mobileOpen) setMobileOpen(false);
    }
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [mobileOpen]);

  const handleItemNav = useCallback(
    (item: NavItem) => {
      if (item.view) {
        onNavigate(item.view);
      } else if (isLiveRoute(item.path)) {
        router.push(item.path);
        if (typeof window !== "undefined") {
          window.scrollTo({ top: 0, behavior: "auto" });
        }
      } else {
        toast.info(`${t(item.labelKey)} · ${tt("common.comingSoon", "coming soon")}`);
      }
      setMobileOpen(false);
      setOpenGroup(null);
    },
    [onNavigate, router, t]
  );

  const handleNav = useCallback(
    (v: View) => {
      onNavigate(v);
      setMobileOpen(false);
      setOpenGroup(null);
    },
    [onNavigate]
  );

  const isActive = (item: NavItem) => {
    if (item.view) {
      if (item.view === "home") return pathname === "/";
      return pathname === `/${item.view}`;
    }
    return pathname === item.path;
  };

  return (
    <header
      className={cn(
        "sticky top-0 z-50 border-b-4 border-black bg-[#FBF9F5] transition-shadow  ",
        scrolled && "shadow-[0_4px_0_0_#000] "
      )}
    >
      {/* ── Top marquee strip — personality, technical ticker ── */}
      <div className="h-7 overflow-hidden border-b-2 border-black bg-black text-[#FBF9F5]">
        <div className="flex h-full w-max animate-marquee items-center font-tech text-[10px] uppercase tracking-[0.25em]">
          {Array.from({ length: 2 }).map((_, i) => (
            <span key={i} className="flex items-center">
              {MARQUEE_ITEMS.map((item) => (
                <span key={item} className="mx-4 flex items-center gap-3">
                  <span className="text-[#FF5500]">◆</span>
                  {item}
                </span>
              ))}
            </span>
          ))}
        </div>
      </div>
      {/* Main bar — single row, no marquee strip */}
      <div className="mx-auto flex max-w-7xl items-center justify-between gap-3 px-4 py-2.5 sm:px-6">
        {/* Logo — SVG mark (clean circle, no box) + wordmark + subtitle.
            Sin aria-label: el nombre accesible es el texto visible "VantaDB…"
            (axe label-content-name-mismatch). */}
        <button
          onClick={() => handleNav("home")}
          className="group flex shrink-0 items-center gap-2.5"
        >
          <span className="relative inline-flex items-center justify-center transition-transform group-hover:rotate-[8deg]">
            <VantaLogoMark size={40} />
            <span className="absolute -right-0.5 -top-0.5 h-2.5 w-2.5 border-2 border-black bg-[#FF5500] " />
          </span>
          <span className="flex flex-col items-start leading-none">
            <span className="font-display text-2xl uppercase tracking-tight text-black ">
              Vanta<span className="text-[#FF5500]">DB</span>
            </span>
            <span className="font-tech text-[9px] uppercase tracking-[0.3em] text-black/60 ">
              v0.5.0 · embedded rust
            </span>
          </span>
        </button>

        {/* Desktop nav — centered dropdowns */}
        <nav
          className="hidden items-center gap-1 lg:flex"
          aria-label={tt("a11y.mainNav", "Main navigation")}
        >
          {NAV_GROUPS.map((group) => (
            <DesktopDropdown
              key={group.labelKey}
              group={group}
              t={t}
              open={openGroup === group.labelKey}
              onToggle={() =>
                setOpenGroup((prev) => (prev === group.labelKey ? null : group.labelKey))
              }
              onMouseEnter={() => setOpenGroup(group.labelKey)}
              onMouseLeave={() => setOpenGroup(null)}
              onItemSelect={handleItemNav}
              isActiveItem={isActive}
            />
          ))}

          {/* Flat links — heavy borders, brutalist personality */}
          {FLAT_LINKS.map((item) => (
            <button
              key={item.path}
              onClick={() => handleItemNav(item)}
              className={cn(
                "border-4 border-black px-3 py-2 font-tech text-[11px] font-bold uppercase tracking-wider transition-all",
                isActive(item)
                  ? "bg-black text-[#FBF9F5] shadow-[4px_4px_0_0_#FF5500]   "
                  : "bg-[#FBF9F5] text-black shadow-[4px_4px_0_0_#000] hover:translate-x-[2px] hover:translate-y-[2px] hover:shadow-[2px_2px_0_0_#000]     "
              )}
            >
              {t(item.labelKey)}
            </button>
          ))}
        </nav>

        {/* Right actions — search, lang, github, hamburger */}
        <div className="flex items-center gap-1.5 sm:gap-2">
          <button
            onClick={openCommandPalette}
            className="inline-flex h-9 w-9 items-center justify-center border-2 border-black bg-[#FBF9F5] text-black transition-all hover:translate-x-[1px] hover:translate-y-[1px]    sm:inline-flex"
            aria-label={`${t("nav.search")} (⌘K)`}
            title={`${t("nav.search")} (⌘K)`}
          >
            <Search className="h-4 w-4" strokeWidth={2.5} />
          </button>

          <LangToggle />
          {extraActions}

          <a
            href={VANTA.repo}
            target="_blank"
            rel="noopener noreferrer"
            className="hidden items-center gap-1.5 border-2 border-black bg-[#FF5500] px-3 py-1.5 font-tech text-[11px] font-bold uppercase tracking-wider text-black transition-all hover:translate-x-[1px] hover:translate-y-[1px] sm:inline-flex"
            aria-label="GitHub"
          >
            <Github className="h-4 w-4" strokeWidth={2.5} />
            <span className="hidden md:inline">GitHub</span>
          </a>

          <button
            onClick={() => setMobileOpen((o) => !o)}
            className="inline-flex h-9 w-9 items-center justify-center border-2 border-black bg-[#FBF9F5] text-black    lg:hidden"
            aria-label={tt("a11y.toggleMenu", "Toggle menu")}
            aria-expanded={mobileOpen}
          >
            {mobileOpen ? <X className="h-5 w-5" /> : <Menu className="h-5 w-5" />}
          </button>
        </div>
      </div>

      {/* ── Mobile menu drawer ── */}
      {mobileOpen && (
        <div className="border-t-4 border-black bg-[#FBF9F5]   lg:hidden">
          <nav
            className="mx-auto flex max-w-7xl flex-col px-4 py-4"
            aria-label={tt("a11y.mobileNav", "Mobile navigation")}
          >
            {NAV_GROUPS.map((group) => (
              <div key={group.labelKey} className="mb-2">
                <button
                  onClick={() =>
                    setOpenGroup((prev) => (prev === group.labelKey ? null : group.labelKey))
                  }
                  className="flex w-full items-center justify-between border-b-2 border-black/10 px-1 py-2.5 font-tech text-xs font-bold uppercase tracking-wider text-black  "
                  aria-expanded={openGroup === group.labelKey}
                >
                  <span className="flex items-center gap-2">
                    <span className="text-[#FF5500]">◆</span>
                    {t(group.labelKey)}
                  </span>
                  <ChevronDown
                    className={cn(
                      "h-4 w-4 transition-transform duration-200",
                      openGroup === group.labelKey && "rotate-180"
                    )}
                  />
                </button>

                {openGroup === group.labelKey && (
                  <div className="mt-1 ml-3 flex flex-col gap-0.5 border-l-2 border-[#FF5500] pl-3">
                    {group.items.map((item) => (
                      <button
                        key={item.path}
                        onClick={() => handleItemNav(item)}
                        className={cn(
                          "px-2 py-2 text-left font-tech text-[11px] font-bold uppercase tracking-wider transition-colors",
                          isActive(item)
                            ? "text-[#FF5500]"
                            : "text-black/70 hover:text-black  "
                        )}
                      >
                        {t(item.labelKey)}
                      </button>
                    ))}
                  </div>
                )}
              </div>
            ))}

            {/* Flat links in mobile */}
            <div className="mt-3 flex gap-2 border-t-2 border-black/10 pt-3 ">
              {FLAT_LINKS.map((item) => (
                <button
                  key={item.path}
                  onClick={() => handleItemNav(item)}
                  className={cn(
                    "flex-1 border-2 border-black px-3 py-2 font-tech text-[11px] font-bold uppercase tracking-wider transition-colors ",
                    isActive(item)
                      ? "bg-[#FF5500] text-black"
                      : "text-black "
                  )}
                >
                  {t(item.labelKey)}
                </button>
              ))}
            </div>
          </nav>
        </div>
      )}
    </header>
  );
}
