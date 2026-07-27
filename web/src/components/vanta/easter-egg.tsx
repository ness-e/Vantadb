"use client";

import { useEffect, useState, useCallback } from "react";
import { Sparkles, X } from "lucide-react";

/**
 * EasterEgg — typing "vanta" in sequence triggers a special overlay
 * with the mascot and a neon celebration animation.
 * Listens for keydown events globally; resets on incorrect keys.
 */
export function EasterEgg() {
  const [active, setActive] = useState(false);
  const [progress, setProgress] = useState(0);
  const target = "vanta";

  const handleKey = useCallback(
    (e: KeyboardEvent) => {
      // Skip if user is typing in an input/textarea
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      ) {
        return;
      }

      const key = e.key.toLowerCase();
      if (key === target[progress]) {
        const next = progress + 1;
        setProgress(next);
        if (next === target.length) {
          setActive(true);
          setProgress(0);
        }
      } else if (key === target[0]) {
        setProgress(1);
      } else {
        setProgress(0);
      }
    },
    [progress]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [handleKey]);

  // Auto-close after 4 seconds
  useEffect(() => {
    if (!active) return;
    const timer = setTimeout(() => setActive(false), 4000);
    return () => clearTimeout(timer);
  }, [active]);

  if (!active) return null;

  return (
    <div
      className="fixed inset-0 z-[80] flex items-center justify-center p-4"
      role="dialog"
      aria-modal="true"
      aria-label="VantaDB easter egg"
    >
      <div
        className="absolute inset-0 bg-black/70 backdrop-blur-sm"
        onClick={() => setActive(false)}
      />

      <div className="relative animate-stamp">
        {/* Neon glow frame */}
        <div className="absolute -inset-3 animate-pulse-ring bg-[#FF5500]/30" />

        <div className="relative border-4 border-[#FF5500] bg-[#FBF9F5] p-8 shadow-[12px_12px_0_0_#FF5500,12px_12px_0_4px_#000]">
          {/* Mascot */}
          <img
            src="/assets/mascota_gato.png"
            alt="VantaDB shadow cat"
            className="mx-auto h-40 w-40 animate-flicker object-cover mix-blend-multiply "
          />

          {/* SFX text */}
          <div className="mt-4 text-center">
            <p className="font-display text-4xl uppercase text-[#FF5500] text-stencil">
              VANTA!
            </p>
            <p className="mt-2 flex items-center justify-center gap-1.5 font-tech text-xs font-bold uppercase tracking-[0.2em] text-black">
              <Sparkles className="h-3.5 w-3.5 text-[#FF5500]" strokeWidth={2.5} />
              you found the shadow cat
              <Sparkles className="h-3.5 w-3.5 text-[#FF5500]" strokeWidth={2.5} />
            </p>
            <p className="mt-1 font-tech text-[10px] uppercase tracking-wider text-black/50">
              1.2ms · 100% recall · zero network
            </p>
          </div>

          {/* Close button */}
          <button
            onClick={() => setActive(false)}
            className="absolute -right-3 -top-3 inline-flex h-8 w-8 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000]  "
            aria-label="Cerrar"
          >
            <X className="h-4 w-4" strokeWidth={3} />
          </button>
        </div>
      </div>
    </div>
  );
}
