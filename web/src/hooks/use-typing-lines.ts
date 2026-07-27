"use client";

import { useEffect, useState, useRef } from "react";

/**
 * useTypingLines — reveals code lines one-by-one with a typewriter effect
 * when the element enters the viewport. Respects prefers-reduced-motion
 * (shows all lines immediately).
 *
 * Returns the index of the last visible line + whether typing is complete.
 */
export function useTypingLines(
  totalLines: number,
  options?: { threshold?: number; lineDelay?: number; charDelay?: number }
) {
  const [visibleLines, setVisibleLines] = useState(0);
  const [typing, setTyping] = useState(false);
  const ref = useRef<HTMLDivElement | null>(null);
  const startedRef = useRef(false);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    const prefersReduced = window.matchMedia(
      "(prefers-reduced-motion: reduce)"
    ).matches;

    if (prefersReduced) {
      Promise.resolve().then(() => {
        setVisibleLines(totalLines);
      });
      return;
    }

    const obs = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting && !startedRef.current) {
            startedRef.current = true;
            setTyping(true);
            obs.unobserve(entry.target);
          }
        });
      },
      { threshold: options?.threshold ?? 0.3 }
    );

    obs.observe(el);
    return () => obs.disconnect();
  }, [totalLines, options?.threshold]);

  useEffect(() => {
    if (!typing) return;
    if (visibleLines >= totalLines) {
      Promise.resolve().then(() => setTyping(false));
      return;
    }

    const delay = options?.lineDelay ?? 120;
    const id = setTimeout(() => {
      setVisibleLines((v) => v + 1);
    }, delay);
    return () => clearTimeout(id);
  }, [typing, visibleLines, totalLines, options?.lineDelay]);

  return { ref, visibleLines, done: visibleLines >= totalLines };
}
