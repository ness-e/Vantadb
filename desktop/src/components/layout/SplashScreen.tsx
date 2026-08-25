import { useCallback, useEffect, useRef, useState } from "react";
import { MarkStudio } from "../mark/mark-studio";

/**
 * SplashScreen (FIND-23) — cold-start intro using the Mark mascot.
 *
 * Choreography (design-motion-principles: rare moment → expressive motion,
 * one easing language, transform/opacity only):
 *   0ms     mark pops in (overshoot bezier)
 *   250ms   title rises
 *   500ms   subtitle rises + hint fades in late
 *   2600ms  auto-dismiss begins → 400ms fade+scale-out exit
 * Total ≈3s. Click skips straight to exit. prefers-reduced-motion kills all
 * entrance animation (media query in index.css); timer still applies.
 */

const EXIT_MS = 400;
const HOLD_MS = 2600;

export function SplashScreen({ onDismiss }: { onDismiss: () => void }) {
  const [closing, setClosing] = useState(false);
  const dismissedRef = useRef(false);
  const rootRef = useRef<HTMLDivElement>(null);

  const beginExit = useCallback(() => {
    if (dismissedRef.current) return;
    dismissedRef.current = true;
    setClosing(true);
    window.setTimeout(onDismiss, EXIT_MS);
  }, [onDismiss]);

  useEffect(() => {
    const t = window.setTimeout(beginExit, HOLD_MS);
    return () => window.clearTimeout(t);
  }, [beginExit]);

  // UX-15: splash saltable por teclado — foco en el root al montar y
  // Enter/Espacio disparan la salida (antes solo click con mouse).
  useEffect(() => {
    rootRef.current?.focus();
  }, []);

  return (
    // FIND-23: covers the whole window once per cold start; click (o Enter)
    // skips. UX-15: role=button + tabIndex para usuarios de teclado.
    <div
      ref={rootRef}
      role="button"
      tabIndex={0}
      aria-label="Saltar la introducción"
      onClick={beginExit}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          beginExit();
        }
      }}
      className={`splash-root fixed inset-0 z-50 flex cursor-pointer flex-col items-center justify-center gap-6 bg-[var(--background)] ${
        closing ? "splash-closing" : ""
      }`}
      style={{ transitionDuration: `${EXIT_MS}ms` }}
    >
      <div className="splash-mark w-[min(320px,60vw)]">
        <MarkStudio status="idle" />
      </div>
      <div className="splash-title text-center">
        <h1 className="font-[family-name:var(--font-anton)] text-4xl uppercase tracking-wide text-[var(--foreground)]">
          VantaDB Studio
        </h1>
        <p className="splash-sub mt-2 font-[family-name:var(--font-space-mono)] text-xs uppercase tracking-widest text-[var(--muted-foreground)]">
          persistent memory · hybrid retrieval
        </p>
      </div>
      <p className="splash-hint absolute bottom-8 font-[family-name:var(--font-space-mono)] text-[10px] uppercase tracking-widest text-[var(--muted-foreground)] opacity-60">
        click o Enter para entrar
      </p>
    </div>
  );
}
