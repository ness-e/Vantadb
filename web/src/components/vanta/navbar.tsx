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
import { cn } from "@/lib/utils";

// ── Marquee items (technical strings, not i18n) ──
const MARQUEE_ITEMS = [
  "pip install vantadb-py",
  "BM25 + HNSW via RRF",
  "WAL · CRC32C checksums",
  "1.2ms in-process latency",
  "Apache 2.0",
  "Rust 1.94.1+",
  "Zero network",
  "Local-first",
] as const;

// ── Dropdown group definitions (keys reference i18n) ──
interface NavDropdownItem {
  labelKey: string;
  /** The path slug used for future App Router migration */
  path: string;
  /** If this matches a known View, use it directly */
  view?: View;
}

interface NavGroup {
  labelKey: string;
  items: NavDropdownItem[];
}

interface FlatLink {
  labelKey: string;
  path: string;
  view?: View;
}

const NAV_GROUPS: NavGroup[] = [
  {
    labelKey: "nav.platform",
    items: [
      { labelKey: "nav.coreEngine", path: "/engine" },
      { labelKey: "nav.architecture", path: "/architecture" },
    ],
  },
  {
    labelKey: "nav.solutions",
    items: [
      { labelKey: "nav.aiAgents", path: "/solutions/ai-agents" },
      { labelKey: "nav.localRag", path: "/solutions/local-rag" },
      { labelKey: "nav.aiIdeTooling", path: "/solutions/ai-ide-tooling" },
    ],
  },
  {
    labelKey: "nav.resources",
    items: [
      { labelKey: "nav.whyVantadb", path: "/why-vantadb" },
      { labelKey: "nav.benchmarks", path: "/benchmarks", view: "benchmarks" },
      { labelKey: "nav.playground", path: "/playground" },
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

const FLAT_LINKS: FlatLink[] = [
  { labelKey: "nav.security", path: "/security" },
  { labelKey: "nav.useCases", path: "/use-cases" },
  { labelKey: "nav.pricing", path: "/pricing" },
];

// ── Resolve a dropdown/flat item click to a View (for live Tier 1 routes) ──
function resolveView(item: NavDropdownItem | FlatLink): View | null {
  if ("view" in item && item.view) return item.view;
  return null;
}

// ── Check if item is live (has a real page.tsx in current phase) ──
function isItemLive(item: NavDropdownItem | FlatLink): boolean {
  if ("view" in item && item.view) return true; // Tier 1 views are always live
  return isLiveRoute(item.path);
}

// ── Command palette trigger ──
function openCommandPalette() {
  window.dispatchEvent(
    new KeyboardEvent("keydown", { key: "k", metaKey: true, bubbles: true })
  );
}

// ── Dropdown component ──
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
  onItemSelect: (item: NavDropdownItem) => void;
  isActiveItem: (item: NavDropdownItem) => boolean;
}) {
  const ref = useRef<HTMLDivElement>(null);

  // Close on outside click
  useEffect(() => {
    if (!open) return;
    function handler(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        onToggle();
      }
    }
    document.addEventListener("mousedown", handler);
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
          "group flex items-center gap-1.5 border-4 border-black px-3 py-2 font-tech text-xs font-bold uppercase tracking-wider transition-all",
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

      {/* Dropdown panel */}
      {open && (
        <div
          className={cn(
            "absolute left-0 top-full z-50 mt-1 min-w-[200px] border-4 border-black bg-[#FBF9F5] shadow-[6px_6px_0_0_#000]   "
          )}
          role="menu"
        >
          {group.items.map((item) => (
            <button
              key={item.path}
              type="button"
              role="menuitem"
              onClick={() => onItemSelect(item)}
              className={cn(
                "flex w-full items-center gap-2 px-4 py-2.5 font-tech text-xs font-bold uppercase tracking-wider transition-colors text-left",
                isActiveItem(item)
                  ? "bg-[#FF5500] text-black"
                  : "hover:bg-[#F2EDE2] "
              )}
            >
              {t(item.labelKey)}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

// ═══════════════════════════════════════════════════════
// Main Navbar
// ═══════════════════════════════════════════════════════
export function Navbar({
  onNavigate,
  extraActions,
}: {
  /** Optional: override current view (defaults to deriving from pathname) */
  view?: View;
  onNavigate: (v: View) => void;
  extraActions?: React.ReactNode;
}) {
  const { t } = useLanguage();
  const router = useRouter();
  const pathname = usePathname();

  // Derive current view from pathname (App Router)
  const currentView: View =
    pathname === "/benchmarks" ? "benchmarks" : pathname === "/docs" ? "docs" : "home";

  const [scrolled, setScrolled] = useState(false);
  const [mobileOpen, setMobileOpen] = useState(false);
  const [openGroup, setOpenGroup] = useState<string | null>(null);
  const mobileNavRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 8);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  // Close mobile menu on Escape
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape" && mobileOpen) setMobileOpen(false);
    }
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [mobileOpen]);

  // Navigate an item: live routes push, others show "coming soon" toast
  const handleItemNav = useCallback(
    (item: NavDropdownItem | FlatLink) => {
      const v = resolveView(item);
      if (v) {
        onNavigate(v);
      } else if (isItemLive(item)) {
        router.push(item.path);
        if (typeof window !== "undefined") {
          window.scrollTo({ top: 0, behavior: "auto" });
        }
      } else {
        toast.info(`${t(item.labelKey)} — coming soon`);
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

  const toggleGroup = useCallback((key: string) => {
    setOpenGroup((prev) => (prev === key ? null : key));
  }, []);

  const toggleMobileGroup = useCallback((key: string) => {
    setOpenGroup((prev) => (prev === key ? null : key));
  }, []);

  // Check if a path/view is active
  const isActive = (item: NavDropdownItem | FlatLink) => {
    if ("view" in item && item.view) return currentView === item.view;
    return pathname === item.path;
  };

  return (
    <header
      className={cn(
        "sticky top-0 z-50 border-b-4 border-black bg-[#FBF9F5] transition-shadow  ",
        scrolled && "shadow-[0_4px_0_0_#000] "
      )}
    >
      {/* ── Top marquee strip ── */}
      <div className="h-6 overflow-hidden border-b-2 border-black bg-black text-[#FBF9F5]">
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

      {/* ── Main bar ── */}
      <div className="mx-auto flex max-w-7xl items-center justify-between gap-2 px-4 py-2.5 sm:px-6 lg:gap-4">
        {/* Logo */}
        <button
          onClick={() => handleNav("home")}
          className="group flex shrink-0 items-center gap-3"
          aria-label={t("nav.home")}
        >
          <span className="relative inline-flex h-11 w-11 items-center justify-center border-4 border-black bg-black press-neon">
            <img
              src="/assets/avatar_gato.png"
              alt="VantaDB mascot avatar"
              className="h-full w-full object-cover"
            />
            <span className="absolute -right-1 -top-1 h-3 w-3 border-2 border-black bg-[#FF5500]" />
          </span>
          <span className="flex flex-col items-start leading-none">
            <span className="font-display text-2xl uppercase tracking-tight text-black ">
              Vanta<span className="text-[#FF5500]">DB</span>
            </span>
            <span className="font-tech text-[9px] uppercase tracking-[0.3em] text-black/60 ">
              v0.1 · embedded rust
            </span>
          </span>
        </button>

        {/* Desktop nav links + dropdowns */}
        <nav className="hidden items-center gap-2 lg:flex" aria-label="Main navigation">
          {/* Dropdown groups */}
          {NAV_GROUPS.map((group) => (
            <DesktopDropdown
              key={group.labelKey}
              group={group}
              t={t}
              open={openGroup === group.labelKey}
              onToggle={() => toggleGroup(group.labelKey)}
              onMouseEnter={() => setOpenGroup(group.labelKey)}
              onMouseLeave={() => setOpenGroup(null)}
              onItemSelect={handleItemNav}
              isActiveItem={isActive}
            />
          ))}

          {/* Separator + flat links */}
          <span className="hidden text-black/30 xl:inline-block" aria-hidden="true">│</span>
          {FLAT_LINKS.map((item, i) => (
            <button
              key={item.path}
              onClick={() => handleItemNav(item)}
              className={cn(
                "hidden border-4 border-black px-3 py-2 font-tech text-xs font-bold uppercase tracking-wider transition-all xl:inline-flex",
                isActive(item)
                  ? "bg-black text-[#FBF9F5] shadow-[4px_4px_0_0_#FF5500]   "
                  : "bg-[#FBF9F5] text-black shadow-[4px_4px_0_0_#000] hover:translate-x-[2px] hover:translate-y-[2px] hover:shadow-[2px_2px_0_0_#000]     "
              )}
            >
              {t(item.labelKey)}
            </button>
          ))}
        </nav>

        {/* ── Right actions ── */}
        <div className="flex items-center gap-1.5 sm:gap-2">
          {/* Search ⌘K */}
          <button
            onClick={openCommandPalette}
            className={cn(
              "hidden h-9 w-9 items-center justify-center border-4 border-black bg-[#FBF9F5] text-black shadow-[4px_4px_0_0_#000] transition-all hover:translate-x-[2px] hover:translate-y-[2px] hover:shadow-[2px_2px_0_0_#000]",
              "    ",
              "sm:inline-flex"
            )}
            aria-label={`${t("nav.search")} (⌘K)`}
            title={`${t("nav.search")} (⌘K)`}
          >
            <Search className="h-4 w-4" strokeWidth={2.5} />
          </button>

          {/* Lang Toggle */}
          <LangToggle />

          {/* extraActions (slot for future features) */}
          {extraActions}

          {/* GitHub button */}
          <a
            href={VANTA.repo}
            target="_blank"
            rel="noopener noreferrer"
            className={cn(
              "group inline-flex items-center gap-2 border-4 border-black bg-[#FF5500] px-3 py-2 font-tech text-xs font-bold uppercase tracking-wider text-black shadow-[4px_4px_0_0_#000] transition-all hover:translate-x-[2px] hover:translate-y-[2px] hover:shadow-[2px_2px_0_0_#000] active:translate-x-[4px] active:translate-y-[4px] active:shadow-none",
              "hidden sm:inline-flex"
            )}
          >
            <Github className="h-4 w-4" strokeWidth={2.5} />
            <span>{t("nav.github")}</span>
          </a>

          {/* Mobile hamburger */}
          <button
            onClick={() => setMobileOpen((o) => !o)}
            className={cn(
              "inline-flex h-10 w-10 items-center justify-center border-4 border-black bg-[#FBF9F5] text-black shadow-[4px_4px_0_0_#000]",
              "   ",
              "lg:hidden"
            )}
            aria-label="Toggle menu"
            aria-expanded={mobileOpen}
          >
            {mobileOpen ? <X className="h-5 w-5" /> : <Menu className="h-5 w-5" />}
          </button>
        </div>
      </div>

      {/* ── Mobile menu drawer ── */}
      {mobileOpen && (
        <div
          ref={mobileNavRef}
          className={cn(
            "border-t-4 border-black bg-[#FBF9F5]  ",
            "lg:hidden"
          )}
        >
          <nav
            className="mx-auto flex max-w-7xl flex-col px-4 py-4"
            aria-label="Mobile navigation"
          >
            {/* Dropdown groups as expandable sections */}
            {NAV_GROUPS.map((group) => (
              <div key={group.labelKey} className="mb-2">
                {/* Section header (click to expand) */}
                <button
                  onClick={() => toggleMobileGroup(group.labelKey)}
                  className={cn(
                    "flex w-full items-center justify-between border-4 border-black px-4 py-3 font-tech text-sm font-bold uppercase tracking-wider transition-all",
                    "bg-black text-[#FBF9F5]  "
                  )}
                  aria-expanded={openGroup === group.labelKey}
                >
                  <span className="flex items-center gap-3">
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

                {/* Expanded items */}
                {openGroup === group.labelKey && (
                  <div className="mt-1 ml-2 flex flex-col gap-1 border-l-4 border-black pl-2 ">
                    {group.items.map((item) => (
                      <button
                        key={item.path}
                        onClick={() => handleItemNav(item)}
                        className={cn(
                          "flex items-center gap-3 px-3 py-2.5 font-tech text-xs font-bold uppercase tracking-wider transition-colors text-left",
                          isActive(item)
                            ? "bg-black text-[#FBF9F5]  "
                            : "text-black hover:bg-[#F2EDE2]  "
                        )}
                      >
                        <span className="text-[#FF5500]">›</span>
                        {t(item.labelKey)}
                      </button>
                    ))}
                  </div>
                )}
              </div>
            ))}

            {/* Flat links */}
            <div className="mt-3 flex flex-col gap-2 border-t-4 border-black pt-3 ">
              {FLAT_LINKS.map((item) => (
                <button
                  key={item.path}
                  onClick={() => handleItemNav(item)}
                  className={cn(
                    "flex items-center gap-3 border-4 border-black px-4 py-3 font-tech text-sm font-bold uppercase tracking-wider transition-all",
                    isActive(item)
                      ? "bg-black text-[#FBF9F5] shadow-[4px_4px_0_0_#FF5500]   "
                      : "bg-[#FBF9F5] text-black shadow-[4px_4px_0_0_#000]   "
                  )}
                >
                  {t(item.labelKey)}
                </button>
              ))}

              {/* Discord link in mobile */}
              <a
                href={VANTA.discord}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-3 border-4 border-black bg-[#FBF9F5] px-4 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black shadow-[4px_4px_0_0_#000]   "
              >
                {t("nav.discord")}
              </a>
            </div>

            {/* Mobile action buttons */}
            <div className="mt-4 flex items-center gap-2 border-t-4 border-black pt-4 ">
              <button
                onClick={openCommandPalette}
                className={cn(
                  "flex flex-1 items-center justify-center gap-2 border-4 border-black bg-[#FBF9F5] px-4 py-3 font-tech text-xs font-bold uppercase tracking-wider text-black shadow-[4px_4px_0_0_#000]",
                  "   "
                )}
              >
                <Search className="h-4 w-4" />
                {t("nav.search")} (⌘K)
              </button>
              <a
                href={VANTA.repo}
                target="_blank"
                rel="noopener noreferrer"
                className="flex flex-1 items-center justify-center gap-2 border-4 border-black bg-[#FF5500] px-4 py-3 font-tech text-xs font-bold uppercase tracking-wider text-black shadow-[4px_4px_0_0_#000]"
              >
                <Github className="h-4 w-4" />
                {t("nav.github")}
              </a>
            </div>
          </nav>
        </div>
      )}
    </header>
  );
}
