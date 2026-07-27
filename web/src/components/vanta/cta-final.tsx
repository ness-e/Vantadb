"use client";

import { useState } from "react";
import { Github, BookOpen, Copy, Check, Zap, ArrowUpRight, Terminal } from "lucide-react";
import { VANTA, PRODUCT } from "./vanta-data";
import { Reveal } from "./reveal";
import { MarkCta, type CtaButton } from "./mark/mark-cta";
import { useLanguage } from "@/lib/language-provider";
import { copyToClipboard } from "./copy-utils";
import { toast } from "./toast";
import { cn } from "@/lib/utils";

/**
 * CtaFinal — closing CTA with interactive mark on top + action buttons below.
 * Mark reacts to button hover (eyes/sphere look toward the button) and
 * has UNIQUE click reactions per button (different from hero blink).
 * Style: black bg, scanlines, grid-tech, neon accents. Matches home aesthetic.
 */
export function CtaFinal() {
  const { t } = useLanguage();
  const [copied, setCopied] = useState(false);
  const [activeButton, setActiveButton] = useState<CtaButton | null>(null);
  const [clickButton, setClickButton] = useState<{ button: CtaButton; timestamp: number } | null>(null);
  const [redirecting, setRedirecting] = useState<string | null>(null);

  // Delayed redirect — lets the mark animation play before leaving
  const delayedRedirect = (url: string, button: CtaButton) => {
    setClickButton({ button, timestamp: Date.now() });
    setRedirecting(button);
    setTimeout(() => {
      window.open(url, "_blank", "noopener,noreferrer");
      setRedirecting(null);
    }, 1800);
  };

  const tt = (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };

  const copyInstall = async () => {
    const ok = await copyToClipboard(PRODUCT.distribution[0].cmd);
    if (ok) {
      setCopied(true);
      toast.copy(PRODUCT.distribution[0].cmd);
      setClickButton({ button: "install", timestamp: Date.now() });
      setTimeout(() => setCopied(false), 1600);
    }
  };

  return (
    <section
      aria-label={tt("ctaFinal.ariaLabel", "Comienza con VantaDB hoy")}
      className="scanlines relative overflow-hidden border-b-4 border-[#FF5500] bg-black text-[#FBF9F5]"
    >
      {/* Grid-tech overlay */}
      <div
        className="pointer-events-none absolute inset-0 opacity-[0.04]"
        aria-hidden
        style={{
          backgroundImage:
            "linear-gradient(to right, rgba(251,249,245,0.5) 1px, transparent 1px), linear-gradient(to bottom, rgba(251,249,245,0.5) 1px, transparent 1px)",
          backgroundSize: "32px 32px",
        }}
      />

      {/* Radial neon glow behind mark */}
      <div
        className="pointer-events-none absolute inset-0"
        aria-hidden
        style={{
          background:
            "radial-gradient(ellipse 40% 40% at 50% 25%, rgba(255,85,0,0.20) 0%, transparent 70%)",
        }}
      />

      <div className="relative mx-auto max-w-4xl px-4 py-16 sm:px-6 sm:py-20">
        {/* Interactive mark — top center */}
        <Reveal direction="up">
          <div className="mb-8 flex justify-center">
            <MarkCta activeButton={activeButton} clickButton={clickButton} />
          </div>
        </Reveal>

        {/* Title */}
        <Reveal direction="up" delay={60}>
          <h2 className="glitch-hover text-center font-display text-stencil uppercase leading-[0.82] text-[#FBF9F5]">
            <span className="block text-[12vw] sm:text-[8vw] lg:text-[6vw]">
              {tt("ctaFinal.titleLine1", "Start building")}
            </span>
            <span className="block text-[12vw] sm:text-[8vw] lg:text-[6vw]">
              <span className="text-outline-neon glow-neon">{tt("ctaFinal.titleLine2", "today")}</span>
            </span>
          </h2>
        </Reveal>

        {/* Subtitle */}
        <Reveal direction="up" delay={120}>
          <p className="mx-auto mt-4 max-w-lg text-center font-tech text-sm leading-relaxed text-[#FBF9F5]/70 sm:text-base">
            {tt(
              "ctaFinal.subtitle",
              "Una instalación. Cero daemons. Motor Rust in-process con WAL crash-safe y retrieval híbrido nativo."
            )}
          </p>
        </Reveal>

        {/* CTA buttons — redesigned, brutalist style with hover effects */}
        <Reveal direction="up" delay={180}>
          <div className="mt-8 flex flex-col items-stretch justify-center gap-3 sm:flex-row sm:items-center">
            {/* Install button — primary, neon, with terminal-style command */}
            <button
              type="button"
              onClick={copyInstall}
              onMouseEnter={() => setActiveButton("install")}
              onMouseLeave={() => setActiveButton(null)}
              className={cn(
                "group relative flex items-center justify-center gap-2 border-4 px-5 py-3 font-tech text-xs font-bold uppercase tracking-wider transition-all",
                activeButton === "install"
                  ? "border-[#FF5500] bg-[#FF5500] text-black shadow-[6px_6px_0_0_#FF5500]"
                  : "border-[#FBF9F5] bg-[#FF5500] text-black shadow-[6px_6px_0_0_#000] hover:translate-x-[2px] hover:translate-y-[2px] hover:shadow-[4px_4px_0_0_#000]"
              )}
              aria-label={tt("ctaFinal.getStartedAria", "Copiar comando de instalación")}
            >
              <Terminal className="h-4 w-4" strokeWidth={2.5} />
              <span>{tt("ctaFinal.getStarted", "Get Started")}</span>
              <span className="border-2 border-black bg-black px-1.5 py-0.5 font-tech text-[9px] text-[#FF5500]">
                $ pip install
              </span>
              {copied ? (
                <Check className="h-3.5 w-3.5" strokeWidth={3} />
              ) : (
                <Copy className="h-3.5 w-3.5 opacity-60 transition-opacity group-hover:opacity-100" />
              )}
            </button>

            {/* Docs button — secondary, outline crema, delayed redirect */}
            <button
              type="button"
              onMouseEnter={() => setActiveButton("docs")}
              onMouseLeave={() => setActiveButton(null)}
              onClick={() => delayedRedirect(VANTA.quickstart, "docs")}
              disabled={redirecting !== null}
              className={cn(
                "group flex items-center justify-center gap-2 border-4 px-5 py-3 font-tech text-xs font-bold uppercase tracking-wider transition-all",
                activeButton === "docs" || redirecting === "docs"
                  ? "border-[#FF5500] bg-[#1A1A1A] text-[#FF5500] shadow-[6px_6px_0_0_#FF5500]"
                  : "border-[#FBF9F5] bg-[#0a0a0a] text-[#FBF9F5] shadow-[6px_6px_0_0_#000] hover:translate-x-[2px] hover:translate-y-[2px] hover:shadow-[4px_4px_0_0_#000]",
                redirecting !== null && "cursor-wait opacity-60"
              )}
            >
              <BookOpen className="h-4 w-4" strokeWidth={2.5} />
              {redirecting === "docs" ? "Opening..." : tt("ctaFinal.readDocs", "Read Docs")}
              {redirecting !== "docs" && <ArrowUpRight className="h-3.5 w-3.5" strokeWidth={2.5} />}
            </button>

            {/* GitHub button — tertiary, ghost/transparent, delayed redirect */}
            <button
              type="button"
              onMouseEnter={() => setActiveButton("github")}
              onMouseLeave={() => setActiveButton(null)}
              onClick={() => delayedRedirect(VANTA.repo, "github")}
              disabled={redirecting !== null}
              className={cn(
                "group flex items-center justify-center gap-2 border-4 px-5 py-3 font-tech text-xs font-bold uppercase tracking-wider transition-all",
                activeButton === "github" || redirecting === "github"
                  ? "border-[#FF5500] bg-[#1A1A1A] text-[#FF5500] shadow-[6px_6px_0_0_#FF5500]"
                  : "border-[#FBF9F5]/30 bg-transparent text-[#FBF9F5]/60 shadow-[6px_6px_0_0_#000] hover:translate-x-[2px] hover:translate-y-[2px] hover:border-[#FBF9F5]/60 hover:text-[#FBF9F5] hover:shadow-[4px_4px_0_0_#000]",
                redirecting !== null && "cursor-wait opacity-60"
              )}
            >
              <Github className="h-4 w-4" strokeWidth={2.5} />
              {redirecting === "github" ? "Opening..." : "GitHub"}
              {redirecting !== "github" && <ArrowUpRight className="h-3.5 w-3.5" strokeWidth={2.5} />}
            </button>
          </div>
        </Reveal>

        {/* Footnote — centralized product data */}
        <Reveal direction="up" delay={240}>
          <p className="mt-8 text-center font-tech text-[10px] uppercase tracking-[0.2em] text-[#FBF9F5]/40">
            {PRODUCT.versions.license} · Rust {PRODUCT.versions.rust} · Python {PRODUCT.versions.python} · Windows / macOS / Linux
          </p>
        </Reveal>
      </div>
    </section>
  );
}
