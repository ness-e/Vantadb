"use client";

import { useState, useMemo } from "react";
import { GraduationCap, Clock, ArrowRight, BookOpen, Search, X, Filter } from "lucide-react";
import { TUTORIALS, VANTA } from "./vanta-data";
import { Reveal } from "./reveal";
import { TutorialModal } from "./tutorial-modal";
import type { View } from "./vanta-data";
import { cn } from "@/lib/utils";
import { useLanguage } from "@/lib/language-provider";

const LEVEL_STYLES: Record<string, string> = {
  Beginner: "bg-[#a3d9a5] text-black border-black",
  Intermediate: "bg-[#ffd479] text-black border-black",
  Advanced: "bg-[#FF5500] text-black border-black",
};

const LEVELS = ["All", "Beginner", "Intermediate", "Advanced"] as const;

export function TutorialsSection({ onNavigate }: { onNavigate: (v: View) => void }) {
  const { t } = useLanguage();
  const [selected, setSelected] = useState<number | null>(null);
  const [query, setQuery] = useState("");
  const [levelFilter, setLevelFilter] = useState<(typeof LEVELS)[number]>("All");

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return TUTORIALS.filter((tutorial) => {
      const matchesLevel = levelFilter === "All" || tutorial.level === levelFilter;
      const matchesQuery =
        !q ||
        tutorial.title.toLowerCase().includes(q) ||
        tutorial.desc.toLowerCase().includes(q) ||
        tutorial.tags.some((tag) => tag.includes(q));
      return matchesLevel && matchesQuery;
    });
  }, [query, levelFilter]);

  const selectedTutorial = selected !== null ? TUTORIALS[selected] : null;

  return (
    <section className="relative border-b-4 border-black bg-[#FBF9F5]  ">
      <div className="pointer-events-none absolute inset-0 grid-tech opacity-50" aria-hidden />
      <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6">
        {/* Header */}
        <Reveal direction="up">
          <div className="mb-6 flex flex-col gap-3 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row sm:items-end sm:justify-between">
            <div>
              <h2 className="glitch-hover mt-3 font-display text-4xl uppercase leading-none sm:text-5xl">
                Tutorials
              </h2>
              <p className="mt-2 max-w-xl font-tech text-sm text-[#FBF9F5]/70">
                {t("tutorials.subtitle")}
              </p>
            </div>
            <a
              href={VANTA.repo}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex shrink-0 items-center gap-2 border-4 border-[#FBF9F5] bg-[#1A1A1A] px-4 py-2 font-tech text-xs font-bold uppercase tracking-wider text-[#FF5500] transition-all hover:translate-x-[2px] hover:translate-y-[2px] hover:bg-[#FF5500] hover:text-black"
            >
              <BookOpen className="h-4 w-4" strokeWidth={2.5} />
              {t("tutorials.allDocs")}
            </a>
          </div>
        </Reveal>

        {/* Search + filter bar */}
        <Reveal direction="up" delay={60}>
          <div className="mb-6 flex flex-col gap-3 border-4 border-black bg-[#FBF9F5] p-3 shadow-[4px_4px_0_0_#000]    sm:flex-row sm:items-center">
            {/* Search */}
            <div className="relative flex-1">
              <Search className="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-black/40 " strokeWidth={2.5} />
              <input
                type="text"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder={t("tutorials.filterPlaceholder")}
                aria-label={t("tutorials.filterAria")}
                className="w-full border-2 border-black bg-[#F2EDE2] py-1.5 pl-8 pr-7 font-tech text-xs text-black placeholder:text-black/40 focus:border-[#FF5500] focus:outline-none    "
              />
              {query && (
                <button
                  onClick={() => setQuery("")}
                  className="absolute right-2 top-1/2 -translate-y-1/2 text-black/50 hover:text-[#FF5500] "
                  aria-label={t("tutorials.clearSearch")}
                >
                  <X className="h-3.5 w-3.5" strokeWidth={2.5} />
                </button>
              )}
            </div>
            {/* Level filter */}
            <div className="flex items-center gap-1">
              <Filter className="h-3.5 w-3.5 shrink-0 text-black/40 " strokeWidth={2.5} />
              {LEVELS.map((lvl) => (
                <button
                  key={lvl}
                  onClick={() => setLevelFilter(lvl)}
                  className={cn(
                    "border-2 border-black px-2 py-1 font-tech text-[10px] font-bold uppercase tracking-wider transition-all ",
                    levelFilter === lvl
                      ? "bg-[#FF5500] text-black"
                      : "bg-[#FBF9F5] text-black/60 hover:bg-[#F2EDE2]   "
                  )}
                >
                  {lvl === "All" ? t("tutorials.all") : lvl}
                </button>
              ))}
            </div>
          </div>
        </Reveal>

        {/* Tutorial cards */}
        {filtered.length === 0 ? (
          <div className="border-4 border-dashed border-black/30 py-12 text-center ">
            <p className="font-tech text-xs uppercase tracking-wider text-black/40 ">
              {t("tutorials.noResults").replace("{{query}}", query).replace("{{filter}}", levelFilter)}
            </p>
          </div>
        ) : (
          <div className="grid grid-cols-1 gap-5 md:grid-cols-2">
            {filtered.map((tutorial, i) => {
              const originalIdx = TUTORIALS.indexOf(tutorial);
              return (
                <Reveal key={tutorial.num} direction={i % 2 === 0 ? "right" : "left"} delay={i * 60}>
                  <button
                    onClick={() => setSelected(originalIdx)}
                    className="press-lg group block h-full w-full border-4 border-black bg-[#FBF9F5] p-5 text-left  "
                    aria-label={`Abrir tutorial: ${tutorial.title}`}
                  >
                    <div className="flex items-start justify-between gap-3">
                      <div className="flex items-center gap-3">
                        <span className="font-display text-4xl text-[#FF5500]">{tutorial.num}</span>
                        <div>
                          <h3 className="font-display text-xl uppercase leading-none text-black ">
                            {tutorial.title}
                          </h3>
                          <div className="mt-1.5 flex items-center gap-2">
                            <span
                              className={cn(
                                "border-2 px-1.5 py-0.5 font-tech text-[9px] font-bold uppercase tracking-wider",
                                LEVEL_STYLES[tutorial.level] ?? LEVEL_STYLES.Beginner
                              )}
                            >
                              {tutorial.level === "Beginner" ? t("tutorials.levels.beginner") : tutorial.level === "Intermediate" ? t("tutorials.levels.intermediate") : t("tutorials.levels.advanced")}
                            </span>
                            <span className="flex items-center gap-1 font-tech text-[9px] uppercase tracking-wider text-black/50 ">
                              <Clock className="h-2.5 w-2.5" strokeWidth={2.5} />
                              {tutorial.duration}
                            </span>
                            <span className="font-tech text-[9px] uppercase tracking-wider text-black/50 ">
                              · {tutorial.steps.length} {t("tutorials.steps").replace("{{count}}", "").trim()}
                            </span>
                          </div>
                        </div>
                      </div>
                      <ArrowRight className="h-5 w-5 shrink-0 text-black/30 transition-all group-hover:translate-x-1 group-hover:text-[#FF5500] " strokeWidth={2.5} />
                    </div>

                    <p className="mt-3 font-tech text-xs leading-relaxed text-black/70 ">
                      {tutorial.desc}
                    </p>

                    <div className="mt-4 flex flex-wrap gap-1.5">
                      {tutorial.tags.map((tag) => (
                        <span
                          key={tag}
                          className="border border-black/30 bg-[#F2EDE2] px-1.5 py-0.5 font-mono text-[9px] text-black/60   "
                        >
                          {tag}
                        </span>
                      ))}
                    </div>

                    <div className="mt-4 h-1 w-full speed-lines opacity-30" />
                  </button>
                </Reveal>
              );
            })}
          </div>
        )}

        {/* CTA */}
        <Reveal direction="up" delay={120}>
          <div className="mt-8 flex flex-col items-center justify-between gap-4 border-4 border-black bg-[#FF5500] p-6 shadow-[6px_6px_0_0_#000]   sm:flex-row">
            <div>
              <h3 className="font-display text-2xl uppercase leading-none text-black">
                {t("tutorials.readyToBuild")}
              </h3>
              <p className="mt-1 font-tech text-xs text-black/80">
                {t("tutorials.readyToBuildDesc")}
              </p>
            </div>
            <button
              onClick={() => onNavigate("docs")}
              className="press inline-flex shrink-0 items-center gap-2 border-4 border-black bg-black px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-[#FBF9F5]"
            >
              {t("tutorials.openQuickstart")}
              <ArrowRight className="h-4 w-4" strokeWidth={2.5} />
            </button>
          </div>
        </Reveal>
      </div>

      {/* Tutorial modal */}
      <TutorialModal tutorial={selectedTutorial} onClose={() => setSelected(null)} />
    </section>
  );
}
