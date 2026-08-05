"use client";

import { useEffect, useState, useMemo, useRef, useCallback } from "react";
import { Search, X, CornerDownLeft, ArrowUp, ArrowDown, Command } from "lucide-react";
import type { View } from "./vanta-data";
import { useFocusTrap } from "@/hooks/use-focus-trap";
import { VANTA, CORE_CAPABILITIES, CLI_COMMANDS, FAQ, DOC_LINKS } from "./vanta-data";
import { cn } from "@/lib/utils";

type SearchItem = {
  id: string;
  label: string;
  hint: string;
  group: "Pages" | "Features" | "CLI" | "FAQ" | "Docs";
  action: () => void;
  external?: boolean;
};

export function CommandPalette({
  onNavigate,
}: {
  onNavigate: (v: View) => void;
}) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const paletteRef = useRef<HTMLDivElement>(null);
  useFocusTrap(paletteRef, open);
  const listRef = useRef<HTMLDivElement>(null);

  // Build the searchable index
  const allItems = useMemo<SearchItem[]>(() => {
    const go = (v: View) => () => {
      onNavigate(v);
      setOpen(false);
    };
    const items: SearchItem[] = [
      { id: "page-home", label: "Index", hint: "Página principal", group: "Pages", action: go("home") },
      { id: "page-bench", label: "Benchmarks", hint: "Rendimiento BENCH-01 · SIFT1M", group: "Pages", action: go("benchmarks") },
      { id: "page-docs", label: "Quickstart", hint: "Guía de instalación y CLI", group: "Pages", action: go("docs") },
      { id: "page-trust", label: "Why VantaDB", hint: "Trust metrics, tech stack, licenses", group: "Pages", action: go("home") },
      { id: "page-playground", label: "Code Playground", hint: "Simulador interactivo Python", group: "Pages", action: go("docs") },
    ];

    CORE_CAPABILITIES.forEach((c) => {
      items.push({
        id: `feat-${c.title}`,
        label: c.title,
        hint: c.mechanism,
        group: "Features",
        action: () => {
          onNavigate("home");
          setOpen(false);
          setTimeout(() => {
            const els = document.querySelectorAll("article h3");
            els.forEach((el) => {
              if (el.textContent?.includes(c.title)) {
                el.scrollIntoView({ behavior: "smooth", block: "center" });
              }
            });
          }, 100);
        },
      });
    });

    CLI_COMMANDS.forEach((c) => {
      items.push({
        id: `cli-${c.cmd}`,
        label: `vanta-cli ${c.cmd}`,
        hint: c.desc,
        group: "CLI",
        action: () => {
          onNavigate("docs");
          setOpen(false);
        },
      });
    });

    FAQ.forEach((f, i) => {
      items.push({
        id: `faq-${i}`,
        label: f.q,
        hint: "Pregunta frecuente",
        group: "FAQ",
        action: () => {
          onNavigate("home");
          setOpen(false);
          setTimeout(() => {
            const els = document.querySelectorAll("section h2");
            els.forEach((el) => {
              if (el.textContent?.includes("FAQ")) {
                el.scrollIntoView({ behavior: "smooth", block: "start" });
              }
            });
          }, 100);
        },
      });
    });

    DOC_LINKS.forEach((d) => {
      items.push({
        id: `doc-${d.name}`,
        label: d.name,
        hint: d.desc,
        group: "Docs",
        action: () => {
          window.open(VANTA.repo, "_blank");
          setOpen(false);
        },
        external: true,
      });
    });

    return items;
  }, [onNavigate]);

  // Filter items by query
  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return allItems;
    return allItems.filter(
      (item) =>
        item.label.toLowerCase().includes(q) ||
        item.hint.toLowerCase().includes(q) ||
        item.group.toLowerCase().includes(q)
    );
  }, [query, allItems]);

  // Group filtered items
  const grouped = useMemo(() => {
    const groups: Record<string, SearchItem[]> = {};
    filtered.forEach((item) => {
      if (!groups[item.group]) groups[item.group] = [];
      groups[item.group].push(item);
    });
    return groups;
  }, [filtered]);

  // Keyboard shortcut: cmd/ctrl+K to open, Escape to close
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setOpen((o) => !o);
      } else if (e.key === "Escape" && open) {
        setOpen(false);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open]);

  // Focus input when opened, reset when closed
  useEffect(() => {
    if (open) {
      Promise.resolve().then(() => {
        setQuery("");
        setActiveIndex(0);
        inputRef.current?.focus();
      });
    }
  }, [open]);

  // Reset active index when query changes
  useEffect(() => {
    Promise.resolve().then(() => setActiveIndex(0));
  }, [query]);

  // Keyboard navigation within results
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setActiveIndex((i) => Math.min(i + 1, filtered.length - 1));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setActiveIndex((i) => Math.max(i - 1, 0));
      } else if (e.key === "Enter") {
        e.preventDefault();
        const item = filtered[activeIndex];
        if (item) item.action();
      }
    },
    [filtered, activeIndex]
  );

  // Scroll active item into view
  useEffect(() => {
    const el = listRef.current?.querySelector(`[data-idx="${activeIndex}"]`);
    el?.scrollIntoView({ block: "nearest" });
  }, [activeIndex]);

  if (!open) return null;

  let flatIndex = -1;

  return (
    <div
      ref={paletteRef}
      tabIndex={-1}
      className="fixed inset-0 z-[70] flex items-start justify-center p-4 pt-[12vh] outline-none"
      role="dialog"
      aria-modal="true"
      aria-label="Búsqueda rápida"
    >
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50 backdrop-blur-[2px]"
        onClick={() => setOpen(false)}
      />

      {/* Panel */}
      <div className="relative w-full max-w-xl border-4 border-black bg-[#FBF9F5] shadow-[8px_8px_0_0_#000]   ">
        {/* Search input */}
        <div className="flex items-center gap-3 border-b-4 border-black px-4 py-3 ">
          <Search className="h-5 w-5 text-[#FF5500]" strokeWidth={2.5} />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Buscar páginas, features, CLI, FAQ..."
            aria-label="Buscar"
            className="flex-1 bg-transparent font-tech text-sm text-black placeholder:text-black/40 focus:outline-none  "
          />
          <button
            onClick={() => setOpen(false)}
            className="inline-flex h-7 w-7 items-center justify-center border-2 border-black bg-[#F2EDE2] text-black transition-colors hover:bg-[#FF5500]   "
            aria-label="Cerrar búsqueda"
          >
            <X className="h-5 w-5" strokeWidth={3} />
          </button>
        </div>

        {/* Results */}
        <div
          ref={listRef}
          className="scroll-manga max-h-[50vh] overflow-y-auto p-2"
        >
          {filtered.length === 0 && (
            <div className="px-4 py-8 text-center">
              <p className="font-tech text-xs uppercase tracking-wider text-black/40 ">
                Sin resultados para &ldquo;{query}&rdquo;
              </p>
            </div>
          )}
          {Object.entries(grouped).map(([group, items]) => (
            <div key={group} className="mb-2">
              <p className="px-2 py-1 font-tech text-[9px] font-bold uppercase tracking-[0.2em] text-black/40 ">
                {group}
              </p>
              {items.map((item) => {
                flatIndex += 1;
                const idx = flatIndex;
                const isActive = idx === activeIndex;
                return (
                  <button
                    key={item.id}
                    data-idx={idx}
                    onMouseEnter={() => setActiveIndex(idx)}
                    onClick={item.action}
                    className={cn(
                      "flex w-full items-center gap-3 border-2 px-3 py-2 text-left transition-colors",
                      isActive
                        ? "border-black bg-black text-[#FF5500]   "
                        : "border-transparent text-black hover:bg-[#F2EDE2]  "
                    )}
                  >
                    <div className="flex-1 min-w-0">
                      <p className="truncate font-tech text-xs font-bold uppercase tracking-wide">
                        {item.label}
                      </p>
                      <p
                        className={cn(
                          "truncate font-tech text-[10px]",
                          isActive ? "text-[#FBF9F5]/60 " : "text-black/50 "
                        )}
                      >
                        {item.hint}
                      </p>
                    </div>
                    {item.external && (
                      <span className="font-tech text-[9px] uppercase tracking-wider opacity-60">↗</span>
                    )}
                    {isActive && (
                      <CornerDownLeft className="h-3.5 w-3.5 shrink-0" strokeWidth={2.5} />
                    )}
                  </button>
                );
              })}
            </div>
          ))}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between border-t-2 border-black/15 px-4 py-2 ">
          <div className="flex items-center gap-3 font-tech text-[9px] uppercase tracking-wider text-black/40 ">
            <span className="flex items-center gap-1">
              <ArrowUp className="h-2.5 w-2.5" />
              <ArrowDown className="h-2.5 w-2.5" />
              navegar
            </span>
            <span className="flex items-center gap-1">
              <CornerDownLeft className="h-2.5 w-2.5" />
              seleccionar
            </span>
            <span className="flex items-center gap-1">
              <span className="font-mono">esc</span>
              cerrar
            </span>
          </div>
          <span className="font-tech text-[9px] uppercase tracking-wider text-[#FF5500]">
            {filtered.length} resultados
          </span>
        </div>
      </div>
    </div>
  );
}

/**
 * CommandKHint — a small button that shows the ⌘K shortcut and opens the palette.
 * Visible in the navbar area on desktop.
 */
export function CommandKHint({ onClick }: { onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className="hidden items-center gap-2 border-4 border-black bg-[#FBF9F5] px-3 py-2 font-tech text-xs font-bold uppercase tracking-wider text-black shadow-[4px_4px_0_0_#000] transition-all hover:translate-x-[2px] hover:translate-y-[2px] hover:shadow-[2px_2px_0_0_#000]      xl:inline-flex"
      aria-label="Abrir búsqueda rápida (cmd+k)"
    >
      <Search className="h-3.5 w-3.5" strokeWidth={2.5} />
      <span>Buscar</span>
      <kbd className="flex items-center gap-0.5 border-2 border-black bg-black px-1.5 py-0.5 font-mono text-[9px] text-[#FF5500]   ">
        <Command className="h-2.5 w-2.5" strokeWidth={3} />
        K
      </kbd>
    </button>
  );
}
