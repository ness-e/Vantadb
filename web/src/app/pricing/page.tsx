"use client";

import { Check, ArrowRight, Star } from "lucide-react";
import Link from "next/link";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { PRICING_PLANS } from "@/components/vanta/vanta-data";
import { cn } from "@/lib/utils";
import { useLanguage } from "@/lib/language-provider";

const TAG_STYLES: Record<string, string> = {
  ink: "bg-black text-[#FF5500] border-black   ",
  neon: "bg-[#FF5500] text-black border-black",
  muted:
    "bg-[#F2EDE2] text-black/60 border-black   ",
};

export default function PricingPage() {
  const { t, tt } = useLanguage();
  return (
    <div className="animate-rise">
      <PageHeader
        badge="§PRICING"
        title={tt("pricingPage.title", "Pricing")}
        subtitle={tt("pricingPage.subtitle", "The engine is Apache 2.0 and free forever. Pay only for support, SLAs, and enterprise artifacts. No per-query billing, no egress fees, no pods.")}
        tag={tt("pricingPage.tag", "Apache 2.0 core · paid support")}
      />

      <PageSection variant="cream">
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          {PRICING_PLANS.map((plan, i) => (
            <Reveal key={plan.name} direction="up" delay={i * 80} as="article">
              <article
                className={cn(
                  "press-lg group relative flex h-full flex-col border-4 bg-[#FBF9F5] p-6 ",
                  plan.highlight
                    ? "border-[#FF5500] shadow-[8px_8px_0_0_#FF5500]  lg:-mt-3 lg:mb-3"
                    : "border-black shadow-[6px_6px_0_0_#000]  "
                )}
              >
                {plan.highlight && (
                  <span className="absolute -top-3 left-1/2 -translate-x-1/2 inline-flex items-center gap-1 border-2 border-black bg-[#FF5500] px-3 py-0.5 font-tech text-[9px] font-bold uppercase tracking-[0.25em] text-black shadow-[3px_3px_0_0_#000]  ">
                    <Star className="h-3 w-3 fill-current" strokeWidth={0} />
                    {tt("pricingPage.mostPopular", "Most popular")}
                  </span>
                )}

                <div className="flex items-center justify-between">
                  <h3 className="font-display text-2xl uppercase leading-none text-black ">
                    {plan.name}
                  </h3>
                  <span
                    className={cn(
                      "border-2 px-2 py-0.5 font-tech text-[9px] font-bold uppercase tracking-[0.2em]",
                      TAG_STYLES[plan.tagColor] ?? TAG_STYLES.muted
                    )}
                  >
                    {plan.tag}
                  </span>
                </div>

                <div className="mt-4 flex items-baseline gap-2">
                  <span className="font-display text-5xl uppercase leading-none text-black ">
                    {plan.price}
                  </span>
                  {plan.period && (
                    <span className="font-tech text-xs uppercase tracking-wider text-black/60 ">
                      {plan.period}
                    </span>
                  )}
                </div>

                <p className="mt-3 font-tech text-xs leading-relaxed text-black/70 ">
                  {tt(`pricingPage.plan.${i}.description`, plan.description)}
                </p>

                <div className="my-5 h-1 w-full speed-lines opacity-30" />

                <ul className="flex-1 space-y-2">
                  {plan.features.map((f, idx) => (
                    <li
                      key={idx}
                      className="flex items-start gap-2 font-tech text-xs leading-relaxed text-black/80 "
                    >
                      <Check
                        className={cn(
                          "mt-0.5 h-3.5 w-3.5 shrink-0",
                          plan.highlight ? "text-[#FF5500]" : "text-black "
                        )}
                        strokeWidth={3}
                      />
                      <span>{tt(`pricingPage.plan.${i}.feature.${idx}`, f)}</span>
                    </li>
                  ))}
                </ul>

                <Link
                  href={plan.ctaLink}
                  target="_blank"
                  rel="noopener noreferrer"
                  className={cn(
                    "press mt-6 inline-flex items-center justify-between gap-2 border-4 px-4 py-3 font-tech text-xs font-bold uppercase tracking-wider transition-colors",
                    plan.highlight
                      ? "border-black bg-[#FF5500] text-black hover:bg-black hover:text-[#FF5500]   "
                      : "border-black bg-black text-[#FF5500] hover:bg-[#FF5500] hover:text-black    "
                  )}
                >
                  <span>{tt(`pricingPage.plan.${i}.cta`, plan.cta)}</span>
                  <ArrowRight className="h-4 w-4" strokeWidth={2.5} />
                </Link>
              </article>
            </Reveal>
          ))}
        </div>

        <Reveal direction="up" delay={120}>
          <div className="mt-10 border-4 border-dashed border-black/30 p-5 text-center ">
            <p className="font-tech text-xs uppercase tracking-wider text-black/60 ">
              {tt("pricingPage.allPlansNote", "All plans include the full Rust engine, PyO3 bindings, vanta-cli, JSONL export, and community Discord. No telemetry, no per-query billing, no egress.")}
            </p>
          </div>
        </Reveal>
      </PageSection>
    </div>
  );
}
