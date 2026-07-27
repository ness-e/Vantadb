"use client";

import { useEffect, useRef, useState } from "react";

/**
 * useCountUp — animates a number from 0 to `target` when the element enters the viewport.
 * Parses numeric portion of strings like "1.2ms", "5,400", "100%", "0".
 * Respects prefers-reduced-motion (shows final value immediately).
 */
export function useCountUp<T extends HTMLElement = HTMLSpanElement>(
  target: number,
  options?: { duration?: number; decimals?: number; threshold?: number }
) {
  const ref = useRef<T | null>(null);
  const [value, setValue] = useState(0);
  const [started, setStarted] = useState(false);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    const prefersReduced = window.matchMedia(
      "(prefers-reduced-motion: reduce)"
    ).matches;

    if (prefersReduced) {
      Promise.resolve().then(() => {
        setValue(target);
        setStarted(true);
      });
      return;
    }

    // Check if element is already visible on mount (for dynamically-rendered elements)
    const rect = el.getBoundingClientRect();
    const alreadyVisible =
      rect.top < window.innerHeight &&
      rect.bottom > 0 &&
      rect.width > 0;

    if (alreadyVisible && !started) {
      Promise.resolve().then(() => setStarted(true));
      return;
    }

    const obs = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting && !started) {
            setStarted(true);
            obs.unobserve(entry.target);
          }
        });
      },
      { threshold: options?.threshold ?? 0.5 }
    );

    obs.observe(el);
    return () => obs.disconnect();
  }, [target, started, options?.threshold]);

  useEffect(() => {
    if (!started) return;

    const prefersReduced = window.matchMedia(
      "(prefers-reduced-motion: reduce)"
    ).matches;
    if (prefersReduced) {
      Promise.resolve().then(() => setValue(target));
      return;
    }

    const duration = options?.duration ?? 1200;
    const startTime = performance.now();
    let raf = 0;

    const tick = (now: number) => {
      const elapsed = now - startTime;
      const t = Math.min(elapsed / duration, 1);
      // easeOutCubic for a snappy manga feel
      const eased = 1 - Math.pow(1 - t, 3);
      setValue(target * eased);
      if (t < 1) {
        raf = requestAnimationFrame(tick);
      } else {
        setValue(target);
      }
    };

    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [started, target, options?.duration]);

  return { ref, value };
}

/**
 * Parses a stat string like "1.2ms", "5,400", "100%", "0" into { number, suffix }.
 */
export function parseStat(raw: string): { number: number; suffix: string; decimals: number } {
  const match = raw.match(/^([\d.,]+)(.*)$/);
  if (!match) return { number: 0, suffix: raw, decimals: 0 };
  const numStr = match[1].replace(/,/g, "");
  const num = parseFloat(numStr);
  const decimals = numStr.includes(".") ? numStr.split(".")[1].length : 0;
  return { number: num, suffix: match[2] ?? "", decimals };
}

/**
 * CountUpStat — renders a stat value that counts up when scrolled into view.
 * Pass the raw string (e.g. "1.2ms", "5,400", "100%") and it handles parsing + animation.
 */
export function CountUpStat({
  value,
  className,
}: {
  value: string;
  className?: string;
}) {
  const { number, suffix, decimals } = parseStat(value);
  const { ref, value: current } = useCountUp(number, { decimals });

  const display =
    decimals > 0
      ? current.toFixed(decimals)
      : Math.round(current).toLocaleString("en-US");

  return (
    <span ref={ref} className={className}>
      {display}
      {suffix}
    </span>
  );
}
