"use client";

import { useState, useMemo } from "react";
import { GitBranch, Tag, Calendar, ArrowRight, Check, Search, X, Filter } from "lucide-react";
import { CHANGELOG, VANTA } from "./vanta-data";
import { Reveal } from "./reveal";
import { cn } from "@/lib/utils";
import { useLanguage } from "@/lib/language-provider";

const TAG_STYLES: Record<string, string> = {
  neon: "bg-[#FF5500] text-black border-black",
  ink: "bg-black text-[#FF5500] border-black   ",
  muted: "bg-[#F2EDE2] text-black/60 border-black   ",
};

const FILTERS = ["All", "MVP", "Perf", "Stable"] as const;

export function ChangelogSection() {
  const { t } = useLanguage();
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<(typeof FILTERS)[number]>("All");

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return CHANGELOG.filter((r) => {
      const matchesFilter = filter === "All" || r.tag === filter;
      const matchesQuery =
        !q ||
        r.title.toLowerCase().includes(q) ||
        r.version.includes(q) ||
        r.changes.some((c) => c.toLowerCase().includes(q));
      return matchesFilter && matchesQuery;
    });
  }, [query, filter]);

  return (
    <section className="relative border-b-4 border-black bg-[#F2EDE2]  ">
      <div className="pointer-events-none absolute inset-0 grid-tech opacity-50" aria-hidden />
      <div className="relative mx-auto max-w-4xl px-4 py-14 sm:px-6">
        {/* Header */}
        <Reveal direction="up">
          <div className="mb-8 flex flex-col gap-3 border-4 border-black bg-[#FBF9F5] p-6 shadow-[6px_6px_0_0_#000]    sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 glow-box-neon border-2 border-black bg-[#FF5500] px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black ">
                <GitBranch className="h-3 w-3" strokeWidth={3} />
                §09
              </span>
              <h2 className="glitch-hover mt-3 font-display text-4xl uppercase leading-none text-black  sm:text-5xl">
                Changelog
              </h2>
              <p className="mt-2 max-w-md font-tech text-sm text-black/70 ">
                {t("changelog.subtitle")}
              </p>
            </div>
            <a
              href={`${VANTA.repo}/releases`}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex shrink-0 items-center gap-2 border-4 border-black bg-black px-4 py-2 font-tech text-xs font-bold uppercase tracking-wider text-[#FF5500] transition-all hover:translate-x-[2px] hover:translate-y-[2px] hover:bg-[#FF5500] hover:text-black "
            >
              <Tag className="h-4 w-4" strokeWidth={2.5} />
              {t("changelog.allReleases")}
            </a>
          </div>
        </Reveal>

        {/* Search + filter bar */}
        <Reveal direction="up" delay={40}>
          <div className="mb-6 flex flex-col gap-3 border-4 border-black bg-[#FBF9F5] p-3 shadow-[4px_4px_0_0_#000]    sm:flex-row sm:items-center">
            <div className="relative flex-1">
              <Search className="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-black/40 " strokeWidth={2.5} />
              <input
                type="text"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder={t("changelog.filterPlaceholder")}
                aria-label={t("changelog.filterAria")}
                className="w-full border-2 border-black bg-[#F2EDE2] py-1.5 pl-8 pr-7 font-tech text-xs text-black placeholder:text-black/40 focus:border-[#FF5500] focus:outline-none    "
              />
              {query && (
                <button
                  onClick={() => setQuery("")}
                  className="absolute right-2 top-1/2 -translate-y-1/2 text-black/50 hover:text-[#FF5500] "
                  aria-label={t("changelog.clearSearch")}
                >
                  <X className="h-3.5 w-3.5" strokeWidth={2.5} />
                </button>
              )}
            </div>
            <div className="flex items-center gap-1">
              <Filter className="h-3.5 w-3.5 shrink-0 text-black/40 " strokeWidth={2.5} />
              {FILTERS.map((f) => (
                <button
                  key={f}
                  onClick={() => setFilter(f)}
                  className={cn(
                    "border-2 border-black px-2 py-1 font-tech text-[10px] font-bold uppercase tracking-wider transition-all ",
                    filter === f
                      ? "bg-[#FF5500] text-black"
                      : "bg-[#FBF9F5] text-black/60 hover:bg-[#F2EDE2]   "
                  )}
                >
                  {f === "All" ? t("changelog.all") : f}
                </button>
              ))}
            </div>
          </div>
        </Reveal>

        {/* Timeline */}
        <div className="relative">
          {/* Vertical line */}
          <div className="absolute left-5 top-0 bottom-0 w-0.5 bg-black " aria-hidden />

          {filtered.length === 0 ? (
            <div className="border-4 border-dashed border-black/30 py-12 text-center ">
              <p className="font-tech text-xs uppercase tracking-wider text-black/40 ">
                {t("changelog.noResults").replace("{{query}}", query).replace("{{filter}}", filter)}
              </p>
            </div>
          ) : (
          <div className="space-y-6">
            {filtered.map((release, i) => (
              <Reveal key={release.version} direction="right" delay={i * 80}>
                <div className="relative pl-14">
                  {/* Version node on the timeline */}
                  <div className="absolute left-0 top-1 flex h-10 w-10 items-center justify-center border-4 border-black bg-[#FF5500] shadow-[3px_3px_0_0_#000]  ">
                    <span className="font-display text-xs text-black">
                      {release.version.split(".")[0]}
                    </span>
                  </div>

                  {/* Release card */}
                  <div className="border-4 border-black bg-[#FBF9F5] p-5 shadow-[4px_4px_0_0_#000]   ">
                    {/* Header row */}
                    <div className="flex flex-wrap items-center gap-3">
                      <span className="font-display text-2xl text-black ">
                        v{release.version}
                      </span>
                      <span
                        className={cn(
                          "border-2 px-2 py-0.5 font-tech text-[9px] font-bold uppercase tracking-wider",
                          TAG_STYLES[release.tagColor] ?? TAG_STYLES.muted
                        )}
                      >
                        {release.tag}
                      </span>
                      <span className="flex items-center gap-1 font-tech text-[10px] uppercase tracking-wider text-black/50 ">
                        <Calendar className="h-3 w-3" strokeWidth={2.5} />
                        {release.date}
                      </span>
                    </div>

                    <h3 className="mt-2 font-display text-lg uppercase leading-none text-black ">
                      {release.title}
                    </h3>

                    {/* Changes list */}
                    <ul className="mt-3 space-y-1.5">
                      {release.changes.map((change, j) => (
                        <li
                          key={j}
                          className="flex items-start gap-2 font-tech text-xs leading-relaxed text-black/75 "
                        >
                          <Check className="mt-0.5 h-3 w-3 shrink-0 text-[#FF5500]" strokeWidth={3} />
                          <span>{change}</span>
                        </li>
                      ))}
                    </ul>
                  </div>
                </div>
              </Reveal>
            ))}
          </div>
          )}
        </div>

        {/* Footer CTA */}
        <Reveal direction="up" delay={120}>
          <div className="mt-8 flex flex-col items-center justify-between gap-4 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500] sm:flex-row">
            <div>
              <h3 className="font-display text-2xl uppercase leading-none text-[#FBF9F5]">
                {t("changelog.trackRoadmap")}
              </h3>
              <p className="mt-1 font-tech text-xs text-[#FBF9F5]/60">
                {t("changelog.trackRoadmapDesc")}
              </p>
            </div>
            <a
              href={VANTA.repo}
              target="_blank"
              rel="noopener noreferrer"
              className="press-neon inline-flex shrink-0 items-center gap-2 border-4 border-[#FBF9F5] bg-[#FF5500] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black"
            >
              {t("changelog.viewOnGithub")}
              <ArrowRight className="h-4 w-4" strokeWidth={2.5} />
            </a>
          </div>
        </Reveal>
      </div>
    </section>
  );
}
