"use client";

import { useRouter } from "next/navigation";
import { ArrowLeft, ArrowRight, Home, BookOpen, Gauge, CreditCard, Newspaper } from "lucide-react";
import { type LucideIcon } from "lucide-react";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { useLanguage } from "@/lib/language-provider";

type SuggestedLink = {
  href: string;
  label: string;
  labelKey: string;
  labelFallback: string;
  description: string;
  descriptionKey: string;
  descriptionFallback: string;
  icon: LucideIcon;
};

const SUGGESTED: SuggestedLink[] = [
  { href: "/", labelKey: "notFound.suggested.0.label", labelFallback: "Home", label: "Home", descriptionKey: "notFound.suggested.0.description", descriptionFallback: "Start here", description: "Start here", icon: Home },
  { href: "/docs", labelKey: "notFound.suggested.1.label", labelFallback: "Docs", label: "Docs", descriptionKey: "notFound.suggested.1.description", descriptionFallback: "How it works", description: "How it works", icon: BookOpen },
  { href: "/benchmarks", labelKey: "notFound.suggested.2.label", labelFallback: "Benchmarks", label: "Benchmarks", descriptionKey: "notFound.suggested.2.description", descriptionFallback: "1.2ms hybrid", description: "1.2ms hybrid", icon: Gauge },
  { href: "/pricing", labelKey: "notFound.suggested.3.label", labelFallback: "Pricing", label: "Pricing", descriptionKey: "notFound.suggested.3.description", descriptionFallback: "$0 forever", description: "$0 forever", icon: CreditCard },
  { href: "/blog", labelKey: "notFound.suggested.4.label", labelFallback: "Blog", label: "Blog", descriptionKey: "notFound.suggested.4.description", descriptionFallback: "Latest writing", description: "Latest writing", icon: Newspaper },
];

export default function CatchAllNotFoundPage() {
  const router = useRouter();
  const { t } = useLanguage();
  const tt = (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };

  return (
    <div className="animate-rise">
      <PageHeader
        badge="§404"
        title={tt("notFoundPage.title", "Not Found")}
        subtitle={tt("notFoundPage.subtitle", "The page you're looking for doesn't exist or hasn't been built yet. Maybe it's local-first and lives only on your machine.")}
        tag={tt("notFoundPage.tag", "404 · out of bounds")}
      >
        <div className="flex flex-wrap items-center gap-3">
          <span className="inline-flex items-center gap-2 border-2 border-[#FF5500] bg-[#FF5500] px-3 py-1 font-display text-3xl uppercase text-black sm:text-5xl">
            404
          </span>
          <span className="font-tech text-[11px] uppercase tracking-wider text-[#FBF9F5]/60">
            {tt("notFound.headerNote", "No route matched the requested path")}
          </span>
        </div>
      </PageHeader>

      <PageSection variant="cream">
        <Reveal direction="up">
          <div className="mx-auto max-w-3xl border-4 border-black bg-[#F2EDE2] p-6 shadow-[6px_6px_0_0_#000]    sm:p-8">
            <div className="flex flex-col gap-4 sm:flex-row sm:items-start">
              <span className="inline-flex h-12 w-12 shrink-0 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000]  ">
                <ArrowLeft className="h-6 w-6 rotate-45" strokeWidth={2.5} aria-hidden />
              </span>
              <div>
                <h2 className="glitch-hover font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
                  {tt("notFound.bodyTitle", "Nothing to see here")}
                </h2>
                <p className="mt-3 font-tech text-sm leading-relaxed text-black/80 ">
                  {tt("notFound.bodyText", "The URL you hit didn't match any page. Try one of the suggested routes below — or head back home and start over.")}
                </p>
              </div>
            </div>

            <div className="mt-6 flex flex-wrap gap-3 border-t-4 border-black pt-6 ">
              <button
                onClick={() => router.push("/")}
                className="press-neon btn-neon-glow inline-flex items-center gap-2 border-4 border-black bg-[#FF5500] px-5 py-3 font-tech text-xs font-bold uppercase tracking-wider text-black shadow-[4px_4px_0_0_#000] transition-colors hover:bg-black hover:text-[#FF5500]  "
              >
                <Home className="h-4 w-4" strokeWidth={2.5} aria-hidden />
                {tt("notFound.backToHome", "Back to Home")}
              </button>
              <button
                onClick={() => router.back()}
                className="press inline-flex items-center gap-2 border-4 border-dashed border-black/40 px-5 py-3 font-tech text-[11px] font-bold uppercase tracking-wider text-black/60 transition-colors hover:border-[#FF5500] hover:text-[#FF5500]  "
              >
                <ArrowLeft className="h-3.5 w-3.5" strokeWidth={2.5} aria-hidden />
                {tt("notFound.goBack", "Go back")}
              </button>
            </div>
          </div>
        </Reveal>

        {/* Suggested links grid */}
        <Reveal direction="up" delay={80}>
          <div className="mt-10">
            <div className="mb-6 flex items-center gap-3">
              <span className="inline-flex h-2 w-2 animate-flicker bg-[#FF5500]" aria-hidden />
              <span className="font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black/60 ">
                {tt("notFound.suggestedRoutes", "Suggested routes")}
              </span>
            </div>
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-5">
              {SUGGESTED.map((link, i) => {
                const Icon = link.icon;
                return (
                  <Reveal key={link.href} direction="up" delay={i * 60} as="article">
                    <a
                      href={link.href}
                      className="press group flex h-full flex-col gap-3 border-4 border-black bg-[#FBF9F5] p-4 transition-transform hover:translate-x-[-2px] hover:translate-y-[-2px] hover:shadow-[5px_5px_0_0_#FF5500]  "
                    >
                      <span className="inline-flex h-10 w-10 items-center justify-center border-4 border-black bg-[#FF5500] text-black transition-transform group-hover:rotate-[-6deg] ">
                        <Icon className="h-5 w-5" strokeWidth={2.5} aria-hidden />
                      </span>
                      <h3 className="glitch-hover font-display text-lg uppercase leading-none text-black ">
                        {tt(link.labelKey, link.labelFallback)}
                      </h3>
                      <p className="font-tech text-[10px] uppercase tracking-wider text-black/60 ">
                        {tt(link.descriptionKey, link.descriptionFallback)}
                      </p>
                      <span className="mt-auto inline-flex items-center gap-1 font-tech text-[9px] font-bold uppercase tracking-wider text-[#FF5500]">
                        {tt("notFound.open", "Open")}
                        <ArrowRight className="h-3 w-3 transition-transform group-hover:translate-x-1" strokeWidth={2.5} aria-hidden />
                      </span>
                    </a>
                  </Reveal>
                );
              })}
            </div>
          </div>
        </Reveal>

        <Reveal direction="up" delay={120}>
          <div className="mt-10 border-4 border-dashed border-black/30 p-5 text-center ">
            <p className="font-tech text-xs leading-relaxed text-black/60 ">
              {tt("notFound.note", "Think this is a mistake? The URL you tried should exist? File an issue on GitHub and tell us what you expected to find.")}
            </p>
          </div>
        </Reveal>
      </PageSection>
    </div>
  );
}
