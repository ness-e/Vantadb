"use client";

import { Boxes, ArrowRight } from "lucide-react";
import { useLanguage } from "@/lib/language-provider";

// §02 — Trust Bar
// Compact horizontal credibility strip. Black band, cream ink, marquee of
// ecosystem technologies that pair naturally with VantaDB (illustrative).
// Designed as a transition strip between Hero and the rest of the Home.

const LOGOS = [
  { name: "Ollama", note: "local LLM" },
  { name: "LangChain", note: "orchestration" },
  { name: "LlamaIndex", note: "RAG" },
  { name: "CrewAI", note: "agents" },
  { name: "AutoGen", note: "multi-agent" },
  { name: "Haystack", note: "pipelines" },
];

export function TrustBar() {
  const { t } = useLanguage();

  // tt() — translation with fallback. The shared t() returns the key string
  // when the dictionary entry is missing; tt() falls back to the supplied
  // default so the section renders with Spanish copy today, and automatically
  // picks up the dictionary entry when it lands in Fase 5.
  const tt = (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };

  // Duplicate the list once for a seamless marquee loop (translateX -50%).
  const marqueeItems = [...LOGOS, ...LOGOS];

  return (
    <section
      aria-label={tt("trustBar.ariaLabel", "Ecosistema y compatibilidad")}
      className="relative border-b-4 border-black bg-black text-[#FBF9F5]  "
    >
      {/* Subtle halftone overlay (cream dots on black band) */}
      <div
        className="pointer-events-none absolute inset-0 opacity-[0.05]"
        style={{
          backgroundImage:
            "radial-gradient(circle, #FBF9F5 1.2px, transparent 1.4px)",
          backgroundSize: "12px 12px",
        }}
        aria-hidden
      />
      {/* Speed-lines edge accents */}
      <div
        className="pointer-events-none absolute inset-y-0 left-0 w-16 bg-gradient-to-r from-black to-transparent"
        aria-hidden
      />
      <div
        className="pointer-events-none absolute inset-y-0 right-0 w-16 bg-gradient-to-l from-black to-transparent"
        aria-hidden
      />

      <div className="relative mx-auto max-w-7xl px-4 py-6 sm:px-6">
        <div className="flex flex-col items-center gap-4 sm:flex-row sm:items-center sm:justify-between">
          {/* Label / micro-header */}
          <div className="flex shrink-0 items-center gap-2">
            <span className="flex items-center gap-2 font-tech text-[10px] font-bold uppercase tracking-[0.2em] text-[#FBF9F5]/60">
              <Boxes className="h-3.5 w-3.5 text-[#FF5500]" strokeWidth={2.5} />
              {tt("trustBar.label", "Compatible con tu stack local")}
            </span>
          </div>

          {/* Marquee row */}
          <div
            className="relative w-full overflow-hidden sm:max-w-[68%]"
            role="marquee"
            aria-label={tt("trustBar.marqueeLabel", "Tecnologías del ecosistema")}
          >
            <div className="flex w-max animate-marquee items-center gap-3">
              {marqueeItems.map((logo, i) => (
                <span
                  key={`${logo.name}-${i}`}
                  className="group inline-flex shrink-0 items-center gap-2 border-2 border-[#FBF9F5]/15 bg-[#1A1A1A] px-3 py-1.5 transition-colors hover:border-[#FF5500]/60"
                >
                  <span className="h-2 w-2 bg-[#FF5500]" aria-hidden />
                  <span className="font-display text-sm uppercase tracking-wider text-[#FBF9F5]">
                    {tt(`trustBar.logos.${i % LOGOS.length}.name`, logo.name)}
                  </span>
                  <span className="font-tech text-[9px] uppercase tracking-[0.2em] text-[#FF5500]">
                    {tt(`trustBar.logos.${i % LOGOS.length}.note`, logo.note)}
                  </span>
                </span>
              ))}
            </div>
          </div>

          {/* Right-side cue */}
          <span className="hidden shrink-0 items-center gap-1 font-tech text-[10px] uppercase tracking-[0.2em] text-[#FBF9F5]/40 lg:inline-flex">
            {tt("trustBar.tag", "Plug & play")}
            <ArrowRight className="h-3 w-3" strokeWidth={2.5} />
          </span>
        </div>
      </div>
    </section>
  );
}

// i18n keys usadas (añadir a dictionaries.ts en Fase 5):
// trustBar.ariaLabel, trustBar.label, trustBar.marqueeLabel, trustBar.tag,
// trustBar.logos.0.name, trustBar.logos.0.note,
// trustBar.logos.1.name, trustBar.logos.1.note,
// trustBar.logos.2.name, trustBar.logos.2.note,
// trustBar.logos.3.name, trustBar.logos.3.note,
// trustBar.logos.4.name, trustBar.logos.4.note,
// trustBar.logos.5.name, trustBar.logos.5.note
