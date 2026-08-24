"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { X, ChevronLeft, ChevronRight, Copy, Check, Clock, GraduationCap } from "lucide-react";
import { TUTORIALS, VANTA } from "./vanta-data";
import { copyToClipboard } from "./copy-utils";
import { toast } from "./toast";
import { useFocusTrap } from "@/hooks/use-focus-trap";
import { useLanguage } from "@/lib/language-provider";
import { cn } from "@/lib/utils";

type Tutorial = (typeof TUTORIALS)[number];

const LEVEL_STYLES: Record<string, string> = {
  Beginner: "bg-[#a3d9a5] text-black border-black",
  Intermediate: "bg-[#ffd479] text-black border-black",
  Advanced: "bg-[#FF5500] text-black border-black",
};

export function TutorialModal({
  tutorial,
  onClose,
}: {
  tutorial: Tutorial | null;
  onClose: () => void;
}) {
  const [stepIdx, setStepIdx] = useState(0);
  const modalRef = useRef<HTMLDivElement>(null);
  const { tt } = useLanguage();
  useFocusTrap(modalRef, tutorial !== null);

  // Reset step when tutorial changes
  useEffect(() => {
    Promise.resolve().then(() => setStepIdx(0));
  }, [tutorial]);

  // Keyboard navigation
  const handleKey = useCallback(
    (e: KeyboardEvent) => {
      if (!tutorial) return;
      if (e.key === "Escape") onClose();
      else if (e.key === "ArrowRight")
        setStepIdx((i) => Math.min(i + 1, tutorial.steps.length - 1));
      else if (e.key === "ArrowLeft") setStepIdx((i) => Math.max(i - 1, 0));
    },
    [tutorial, onClose]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [handleKey]);

  // Lock body scroll when open
  useEffect(() => {
    if (tutorial) {
      document.body.style.overflow = "hidden";
      return () => {
        document.body.style.overflow = "";
      };
    }
  }, [tutorial]);

  if (!tutorial) return null;

  const step = tutorial.steps[stepIdx];
  const isLast = stepIdx === tutorial.steps.length - 1;

  return (
    <div
      ref={modalRef}
      tabIndex={-1}
      className="fixed inset-0 z-[75] flex items-center justify-center p-4 outline-none"
      role="dialog"
      aria-modal="true"
      aria-label={`Tutorial: ${tutorial.title}`}
    >
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-[2px]"
        onClick={onClose}
      />

      <div className="relative flex max-h-[88vh] w-full max-w-2xl flex-col border-4 border-black bg-[#FBF9F5] shadow-[8px_8px_0_0_#000]   ">
        {/* Header */}
        <div className="flex items-center justify-between border-b-4 border-black bg-black px-5 py-3 ">
          <div className="flex items-center gap-3">
            <span className="font-display text-2xl text-[#FF5500]">{tutorial.num}</span>
            <div>
              <h2 className="font-display text-lg uppercase leading-none text-[#FBF9F5]">
                {tutorial.title}
              </h2>
              <div className="mt-1 flex items-center gap-2">
                <span
                  className={cn(
                    "border-2 px-1.5 py-0.5 font-tech text-[8px] font-bold uppercase tracking-wider",
                    LEVEL_STYLES[tutorial.level] ?? LEVEL_STYLES.Beginner
                  )}
                >
                  {tutorial.level}
                </span>
                <span className="flex items-center gap-1 font-tech text-[9px] uppercase tracking-wider text-[#FBF9F5]/60">
                  <Clock className="h-2.5 w-2.5" strokeWidth={2.5} />
                  {tutorial.duration}
                </span>
              </div>
            </div>
          </div>
          <button
            onClick={onClose}
            className="inline-flex h-8 w-8 items-center justify-center border-2 border-[#FBF9F5]/40 bg-[#1A1A1A] text-[#FBF9F5] transition-colors hover:bg-[#FF5500] hover:text-black"
            aria-label={tt("common.close", "Cerrar")}
          >
            <X className="h-4 w-4" strokeWidth={3} />
          </button>
        </div>

        {/* Step progress bar */}
        <div className="flex gap-1 border-b-2 border-black/10 px-5 py-2 ">
          {tutorial.steps.map((s, i) => (
            <button
              key={i}
              onClick={() => setStepIdx(i)}
              className={cn(
                // R-FE-5: visual 6px alto, hit-area extendida a ~26px vía pseudo-elemento
                "relative h-1.5 flex-1 transition-colors before:absolute before:inset-y-[-10px] before:left-0 before:right-0 before:content-['']",
                i === stepIdx
                  ? "bg-[#FF5500]"
                  : i < stepIdx
                    ? "bg-black "
                    : "bg-black/15 "
              )}
              aria-label={`Paso ${i + 1}: ${s.title}`}
              title={`Paso ${i + 1}: ${s.title}`}
            />
          ))}
        </div>

        {/* Body */}
        <div className="scroll-manga flex-1 overflow-y-auto px-5 py-4">
          <div className="flex items-center gap-2">
            <span className="inline-flex h-7 w-7 items-center justify-center border-2 border-black bg-[#FF5500] font-display text-sm text-black ">
              {stepIdx + 1}
            </span>
            <h3 className="font-display text-lg uppercase leading-none text-black ">
              {step.title}
            </h3>
          </div>

          <p className="mt-3 font-tech text-sm leading-relaxed text-black/80 ">
            {step.body}
          </p>

          {/* Code block with copy */}
          <div className="group/code relative mt-4">
            <StepCodeBlock code={step.code} />
          </div>

          {/* Tags */}
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
        </div>

        {/* Footer / navigation */}
        <div className="flex items-center justify-between border-t-4 border-black bg-[#F2EDE2] px-5 py-3  ">
          <span className="font-tech text-[10px] uppercase tracking-wider text-black/70 ">
            Paso {stepIdx + 1} de {tutorial.steps.length}
          </span>
          <div className="flex items-center gap-2">
            <button
              onClick={() => setStepIdx((i) => Math.max(i - 1, 0))}
              disabled={stepIdx === 0}
              className="press inline-flex items-center gap-1 border-4 border-black bg-[#FBF9F5] px-3 py-2 font-tech text-xs font-bold uppercase tracking-wider text-black disabled:opacity-30   "
              aria-label="Paso anterior"
            >
              <ChevronLeft className="h-3.5 w-3.5" strokeWidth={2.5} />
              Anterior
            </button>
            {isLast ? (
              <a
                href={VANTA.quickstart}
                target="_blank"
                rel="noopener noreferrer"
                onClick={onClose}
                className="press-neon inline-flex items-center gap-1 border-4 border-black bg-[#FF5500] px-3 py-2 font-tech text-xs font-bold uppercase tracking-wider text-black"
              >
                <GraduationCap className="h-3.5 w-3.5" strokeWidth={2.5} />
                Ver en GitHub
              </a>
            ) : (
              <button
                onClick={() => setStepIdx((i) => Math.min(i + 1, tutorial.steps.length - 1))}
                className="press-neon inline-flex items-center gap-1 border-4 border-black bg-[#FF5500] px-3 py-2 font-tech text-xs font-bold uppercase tracking-wider text-black"
                aria-label="Siguiente paso"
              >
                Siguiente
                <ChevronRight className="h-3.5 w-3.5" strokeWidth={2.5} />
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function StepCodeBlock({ code }: { code: string }) {
  const [copied, setCopied] = useState(false);
  const [typed, setTyped] = useState("");
  const [typing, setTyping] = useState(true);

  // Typing animation: reveal code char-by-char when step changes
  useEffect(() => {
    Promise.resolve().then(() => {
      setTyped("");
      setTyping(true);
    });
    const prefersReduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (prefersReduced) {
      Promise.resolve().then(() => {
        setTyped(code);
        setTyping(false);
      });
      return;
    }
    let i = 0;
    const interval = setInterval(() => {
      i += 2; // 2 chars per tick for snappy feel
      if (i >= code.length) {
        Promise.resolve().then(() => {
          setTyped(code);
          setTyping(false);
        });
        clearInterval(interval);
      } else {
        setTyped(code.slice(0, i));
      }
    }, 20);
    return () => clearInterval(interval);
  }, [code]);

  const copy = async () => {
    const ok = await copyToClipboard(code);
    if (ok) {
      setCopied(true);
      toast.copy("Código del paso copiado");
      setTimeout(() => setCopied(false), 1600);
    }
  };

  return (
    <>
      <button
        onClick={copy}
        className="absolute right-2 top-2 z-10 inline-flex h-7 w-7 items-center justify-center border-2 border-[#FBF9F5]/30 bg-[#FBF9F5]/10 text-[#FBF9F5] opacity-0 transition-all hover:bg-[#FF5500] hover:text-black focus-visible:opacity-100 group-hover/code:opacity-100"
        aria-label="Copiar código"
        title="Copiar"
      >
        {copied ? (
          <Check className="h-3.5 w-3.5 text-[#FF5500]" strokeWidth={3} />
        ) : (
          <Copy className="h-3.5 w-3.5" strokeWidth={2.5} />
        )}
      </button>
      <pre className="scroll-manga overflow-x-auto border-2 border-black bg-black p-3 pr-10 font-tech text-[11px] leading-relaxed text-[#FBF9F5] ">
        {typed}
        {typing && <span className="animate-blink">▋</span>}
      </pre>
    </>
  );
}
