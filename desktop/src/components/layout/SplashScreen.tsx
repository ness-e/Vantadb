import { useEffect } from "react";
import { MarkStudio } from "../mark/mark-studio";

/**
 * SplashScreen (FIND-23) — cold-start intro using the Mark mascot.
 * Pure CSS animation (no new deps); auto-dismisses after ~1.8s or on click.
 * `prefers-reduced-motion` disables the entrance animation via media query
 * in index.css (.splash-* rules).
 */

export function SplashScreen({ onDismiss }: { onDismiss: () => void }) {
  useEffect(() => {
    const t = setTimeout(onDismiss, 1800);
    return () => clearTimeout(t);
  }, [onDismiss]);

  return (
    // FIND-23: covers the whole window once per cold start; click skips.
    <div
      onClick={onDismiss}
      className="splash-root fixed inset-0 z-50 flex cursor-pointer flex-col items-center justify-center gap-6 bg-[var(--background)]"
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
        click para entrar
      </p>
    </div>
  );
}
