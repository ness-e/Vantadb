"use client";

import { Check, X, Crosshair } from "lucide-react";
import { SEARCH_SEMANTICS, PRODUCT_BOUNDARY } from "./vanta-data";
import { Reveal } from "./reveal";
import { useLanguage } from "@/lib/language-provider";

export function SearchSemantics() {
  const { t } = useLanguage();
  const tt = (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };
  return (
    <section className="relative border-b-4 border-black bg-[#F2EDE2]  ">
      <div className="pointer-events-none absolute inset-0 grid-tech opacity-50" aria-hidden />
      <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6">
        <div className="grid grid-cols-1 gap-10 lg:grid-cols-2">
          {/* Search Semantics */}
          <Reveal direction="right">
            <h2 className="glitch-hover mt-3 font-display text-4xl uppercase leading-none text-black  sm:text-5xl">
              {tt("searchSemantics.title", "Search Semantics")}
            </h2>
            <p className="mt-2 max-w-md font-tech text-sm text-black/70 ">
              {tt(
                "searchSemantics.subtitle",
                "The shipped retrieval contract — what the engine guarantees today."
              )}
            </p>

            <div className="mt-6 space-y-3">
              {SEARCH_SEMANTICS.map((s, i) => (
                <div
                  key={s.title}
                  className="press flex items-start gap-4 border-4 border-black bg-[#FBF9F5] p-4  "
                >
                  <span className="flex h-8 w-8 shrink-0 items-center justify-center border-4 border-black bg-[#FF5500] font-display text-base text-black ">
                    {i + 1}
                  </span>
                  <div>
                    <h3 className="font-display text-lg uppercase leading-none text-black ">
                      {tt(`searchSemantics.items.${i}.title`, s.title)}
                    </h3>
                    <p className="mt-1 font-tech text-xs leading-relaxed text-black/75 ">
                      {tt(`searchSemantics.items.${i}.body`, s.body)}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </Reveal>

          {/* Product Boundary */}
          <Reveal direction="left">
          <div>
            <h2 className="glitch-hover mt-3 font-display text-4xl uppercase leading-none text-black  sm:text-5xl">
              {tt("productBoundary.title", "Product Boundary")}
            </h2>
            <p className="mt-2 max-w-md font-tech text-sm text-black/70 ">
              {tt(
                "productBoundary.subtitle",
                "VantaDB is an embedded memory engine — not a universal multimodel database or cloud platform. Honest scope, shipped today."
              )}
            </p>

            <div className="mt-6 space-y-3">
              {PRODUCT_BOUNDARY.map((p, i) => {
                const production = p.tone === "ink";
                return (
                  <div
                    key={p.label}
                    className={`flex items-start gap-3 border-4 border-black p-4  ${
                      production ? "bg-black text-[#FBF9F5]  " : "bg-[#FBF9F5] text-black  "
                    }`}
                  >
                    <span
                      className={`mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center border-2 border-black  ${
                        production
                          ? "bg-[#FF5500] text-black"
                          : "bg-[#F2EDE2] text-black/50  "
                      }`}
                    >
                      {production ? (
                        <Check className="h-3.5 w-3.5" strokeWidth={3} />
                      ) : (
                        <X className="h-3.5 w-3.5" strokeWidth={3} />
                      )}
                    </span>
                    <div>
                      <h3
                        className={`font-display text-base uppercase leading-none ${
                          production ? "text-[#FF5500]" : "text-black "
                        }`}
                      >
                        {tt(`productBoundary.items.${i}.label`, p.label)}
                      </h3>
                      <p
                        className={`mt-1 font-tech text-[11px] leading-relaxed ${
                          production ? "text-[#FBF9F5]/80 " : "text-black/70 "
                        }`}
                      >
                        {tt(`productBoundary.items.${i}.items`, p.items)}
                      </p>
                    </div>
                  </div>
                );
              })}
            </div>

            <p className="mt-4 border-l-4 border-[#FF5500] bg-[#FBF9F5] px-3 py-2 font-tech text-[11px] italic text-black/70  ">
              {tt(
                "productBoundary.footnote",
                "MVP = embedded memory + WAL + vector/BM25/hybrid + export/import + CLI/Python"
              )}
            </p>
          </div>
          </Reveal>
        </div>
      </div>
    </section>
  );
}
