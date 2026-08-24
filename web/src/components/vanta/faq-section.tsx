"use client";

import { useState } from "react";
import { HelpCircle, Plus, Minus, MessagesSquare } from "lucide-react";
import { FAQ, VANTA } from "./vanta-data";
import { Reveal } from "./reveal";
import { cn } from "@/lib/utils";
import { useLanguage } from "@/lib/language-provider";

export function FaqSection() {
  const { t } = useLanguage();
  const [open, setOpen] = useState<number | null>(0);

  return (
    <section className="relative border-b-4 border-black bg-[#F2EDE2]  ">
      <div className="pointer-events-none absolute inset-0 grid-tech opacity-50" aria-hidden />
      <div className="relative mx-auto max-w-4xl px-4 py-14 sm:px-6">
        <Reveal direction="up" className="mb-8">
          <div className="flex flex-col gap-3 border-4 border-black bg-[#FBF9F5] p-6 shadow-[6px_6px_0_0_#000]    sm:flex-row sm:items-end sm:justify-between">
            <div>
              <h2 className="glitch-hover mt-3 font-display text-4xl uppercase leading-none text-black  sm:text-5xl">
                {t("faq.title")}
              </h2>
              <p className="mt-2 max-w-md font-tech text-sm text-black/70 ">
                {t("faq.subtitle")}
              </p>
            </div>
            <div className="flex items-center gap-2 font-tech text-[10px] uppercase tracking-wider text-black/70 ">
              <MessagesSquare className="h-3.5 w-3.5" />
              {FAQ.length} {t("faq.questionsCount").replace("{{count}}", "").trim()}
            </div>
          </div>
        </Reveal>

        <div className="space-y-3">
          {FAQ.map((item, i) => {
            const isOpen = open === i;
            return (
              <Reveal key={i} direction="up" delay={i * 40}>
                <div
                  className={cn(
                    "border-4 border-black bg-[#FBF9F5] transition-all  ",
                    isOpen && "shadow-[6px_6px_0_0_#FF5500] "
                  )}
                >
                  <button
                    onClick={() => setOpen(isOpen ? null : i)}
                    className="flex w-full items-center gap-4 p-4 text-left"
                    aria-expanded={isOpen}
                    aria-controls={`faq-panel-${i}`}
                  >
                    <span
                      className={cn(
                        "flex h-9 w-9 shrink-0 items-center justify-center border-4 border-black font-display text-base transition-colors ",
                        isOpen ? "bg-[#FF5500] text-black" : "bg-black text-[#FF5500]  "
                      )}
                    >
                      {String(i + 1).padStart(2, "0")}
                    </span>
                    <span className="flex-1 font-tech text-sm font-bold uppercase tracking-wide text-black  sm:text-base">
                      {item.q}
                    </span>
                    <span
                      className={cn(
                        "flex h-7 w-7 shrink-0 items-center justify-center border-2 border-black transition-transform ",
                        isOpen ? "rotate-180 bg-[#FF5500]" : "bg-[#FBF9F5] "
                      )}
                    >
                      {isOpen ? (
                        <Minus className="h-4 w-4 text-black" strokeWidth={3} />
                      ) : (
                        <Plus className="h-4 w-4 text-black " strokeWidth={3} />
                      )}
                    </span>
                  </button>
                  <div
                    id={`faq-panel-${i}`}
                    className={cn(
                      "grid transition-all duration-300 ease-default",
                      isOpen ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0"
                    )}
                  >
                    <div className="overflow-hidden">
                      <div className="border-t-2 border-black/15 px-4 py-4 ">
                        <p className="pl-13 font-tech text-sm leading-relaxed text-black/80  ">
                          {item.a}
                        </p>
                      </div>
                    </div>
                  </div>
                </div>
              </Reveal>
            );
          })}
        </div>

        {/* Still have questions CTA */}
        <Reveal direction="up" delay={120}>
          <div className="mt-8 flex flex-col items-center justify-between gap-4 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500] sm:flex-row">
            <div>
              <h3 className="font-display text-2xl uppercase leading-none text-[#FBF9F5]">
                {t("faq.moreQuestions")}
              </h3>
              <p className="mt-1 font-tech text-xs text-[#FBF9F5]/60">
                {t("faq.joinCommunity")}
              </p>
            </div>
            <div className="flex gap-2">
              <a
                href={VANTA.discord}
                target="_blank"
                rel="noopener noreferrer"
                className="press-neon inline-flex items-center gap-2 border-4 border-[#FBF9F5] bg-[#FF5500] px-4 py-2.5 font-tech text-xs font-bold uppercase tracking-wider text-black"
              >
                {t("faq.discord")}
              </a>
              <a
                href={VANTA.repo}
                target="_blank"
                rel="noopener noreferrer"
                className="press inline-flex items-center gap-2 border-4 border-[#FBF9F5] bg-[#1A1A1A] px-4 py-2.5 font-tech text-xs font-bold uppercase tracking-wider text-[#FBF9F5]"
              >
                {t("faq.githubDocs")}
              </a>
            </div>
          </div>
        </Reveal>
      </div>
    </section>
  );
}
