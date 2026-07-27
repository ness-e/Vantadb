"use client";

import { useEffect, useState, useRef } from "react";

/**
 * useParallax — tracks mouse position (and device orientation on mobile)
 * and returns normalized offset values (-1 to 1) for X and Y axes.
 * Respects prefers-reduced-motion (returns 0,0).
 *
 * Use the returned offsets to apply transforms like:
 *   transform: translate(${x * intensity}px, ${y * intensity}px)
 */
export function useParallax(intensity = 1) {
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const frameRef = useRef<number>(0);

  useEffect(() => {
    const prefersReduced = window.matchMedia(
      "(prefers-reduced-motion: reduce)"
    ).matches;

    if (prefersReduced) return;

    const onMove = (e: MouseEvent) => {
      if (frameRef.current) cancelAnimationFrame(frameRef.current);
      frameRef.current = requestAnimationFrame(() => {
        // Normalize to -1..1 from viewport center
        const x = (e.clientX / window.innerWidth - 0.5) * 2;
        const y = (e.clientY / window.innerHeight - 0.5) * 2;
        setOffset({ x: x * intensity, y: y * intensity });
      });
    };

    window.addEventListener("mousemove", onMove, { passive: true });
    return () => {
      window.removeEventListener("mousemove", onMove);
      if (frameRef.current) cancelAnimationFrame(frameRef.current);
    };
  }, [intensity]);

  return offset;
}
