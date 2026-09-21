"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { animate } from "animejs";
import type { BlinkState, MarkInteractionState } from "./types";

/**
 * useMarkInteraction — reusable hook for mark variants.
 * Handles:
 *  - Mouse tracking across the entire hero (rAF-throttled)
 *  - Smooth sphere/eye offset via Anime.js (butter-smooth interpolation)
 *  - Mouse distance → eye squint (farther mouse = narrower eyes)
 *  - Blink cycle on click via Anime.js (animates eye height directly on refs)
 *  - Graph node hover (mark looks at the node + node pulse animation)
 *
 * Returns state, handlers, and refs for the variant to wire up.
 */
export function useMarkInteraction(
  markWrapRef: React.RefObject<HTMLDivElement | null>,
  opts: {
    maxEyeOffset?: number;
    maxSphereOffset?: number;
    eyeSmoothMs?: number;
    sphereSmoothMs?: number;
    /** Distance (px) at which eyes are fully squinted (height = minEyeHeight) */
    squintDistance?: number;
    /** Eye height when fully open (mouse close) */
    maxEyeHeight?: number;
    /** Eye height when fully squinted (mouse far) */
    minEyeHeight?: number;
  } = {}
) {
  const {
    maxEyeOffset = 16,
    maxSphereOffset = 7,
    eyeSmoothMs = 180,
    sphereSmoothMs = 380,
    squintDistance = 600,
    maxEyeHeight = 10,
    minEyeHeight = 3,
  } = opts;

  const [state, setState] = useState<MarkInteractionState>({
    pupilOffset: { x: 0, y: 0 },
    sphereOffset: { x: 0, y: 0 },
    blink: "open",
    annoyed: false,
    hoveredNode: null,
    mouseInHero: false,
  });

  // Mouse distance from mark center (for squint effect)
  const [mouseDistance, setMouseDistance] = useState(0);

  // Mutable target offsets (updated by mousemove, applied by rAF)
  const target = useRef({
    eyeX: 0, eyeY: 0,
    sphereX: 0, sphereY: 0,
  });
  // Current displayed offsets (anime.js animates toward target)
  const current = useRef({
    eyeX: 0, eyeY: 0,
    sphereX: 0, sphereY: 0,
  });
  const rafId = useRef<number | null>(null);
  const lastRender = useRef(0);
  // Anime.js animation instances — keep refs so we can pause/cancel before starting new ones
  const eyeAnimRef = useRef<ReturnType<typeof animate> | null>(null);
  const sphereAnimRef = useRef<ReturnType<typeof animate> | null>(null);

  // Mouse tracking across the entire hero
  const handleMouseMove = useCallback((e: MouseEvent) => {
    const wrap = markWrapRef.current;
    if (!wrap) return;
    const rect = wrap.getBoundingClientRect();
    const cx = rect.left + rect.width / 2;
    const cy = rect.top + rect.height / 2;
    const dx = e.clientX - cx;
    const dy = e.clientY - cy;
    const dist = Math.sqrt(dx * dx + dy * dy);
    const angle = Math.atan2(dy, dx);

    const eyeNorm = Math.min(dist / 280, 1);
    const sphereNorm = Math.min(dist / 380, 1);

    // Offsets follow mouse direction (mouse right → mark right, mouse up → mark up).
    target.current = {
      eyeX: Math.cos(angle) * eyeNorm * maxEyeOffset,
      eyeY: Math.sin(angle) * eyeNorm * maxEyeOffset,
      sphereX: Math.cos(angle) * sphereNorm * maxSphereOffset,
      sphereY: Math.sin(angle) * sphereNorm * maxSphereOffset,
    };

    // Update mouse distance for squint effect
    setMouseDistance(dist);

    setState((s) => (s.mouseInHero ? s : { ...s, mouseInHero: true }));

    // rAF throttle — apply via Anime.js for smooth interpolation
    if (rafId.current === null) {
      rafId.current = requestAnimationFrame((ts) => {
        rafId.current = null;
        if (ts - lastRender.current < 16) return;
        lastRender.current = ts;

        // Cancel previous animations before starting new ones (prevents stacking)
        if (eyeAnimRef.current) eyeAnimRef.current.pause();
        if (sphereAnimRef.current) sphereAnimRef.current.pause();

        // Eyes: faster follow (180ms)
        eyeAnimRef.current = animate(current.current, {
          eyeX: target.current.eyeX,
          eyeY: target.current.eyeY,
          duration: eyeSmoothMs,
          ease: "outQuad",
          onUpdate: () => {
            setState((s) => ({
              ...s,
              pupilOffset: { x: current.current.eyeX, y: current.current.eyeY },
            }));
          },
        });

        // Sphere: slower, smoother follow (380ms) — separate animation
        sphereAnimRef.current = animate(current.current, {
          sphereX: target.current.sphereX,
          sphereY: target.current.sphereY,
          duration: sphereSmoothMs,
          ease: "outQuad",
          onUpdate: () => {
            setState((s) => ({
              ...s,
              sphereOffset: { x: current.current.sphereX, y: current.current.sphereY },
            }));
          },
        });
      });
    }
  }, [markWrapRef, maxEyeOffset, maxSphereOffset, eyeSmoothMs, sphereSmoothMs]);

  useEffect(() => {
    window.addEventListener("mousemove", handleMouseMove, { passive: true });
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      if (rafId.current !== null) cancelAnimationFrame(rafId.current);
      // Clean up anime.js instances on unmount
      if (eyeAnimRef.current) eyeAnimRef.current.pause();
      if (sphereAnimRef.current) sphereAnimRef.current.pause();
    };
  }, [handleMouseMove]);

  // Refs for the two eye rects — variant assigns these, hook animates them on blink
  const leftEyeRef = useRef<SVGRectElement | null>(null);
  const rightEyeRef = useRef<SVGRectElement | null>(null);

  // Compute current squint height based on mouse distance
  // Close mouse → maxEyeHeight, far mouse → minEyeHeight (squint)
  const squintNorm = Math.min(mouseDistance / squintDistance, 1);
  const squintHeight = maxEyeHeight - squintNorm * (maxEyeHeight - minEyeHeight);

  // Click → blink via anime.js (animates height AND y together on the DOM directly)
  // React does NOT change blink state → no re-render → anime.js has full control.
  // Mouse tracking continues (anime.js updates pupilOffset), but the blinking eye's
  // height/y are overridden by the blink animation until it completes.
  const cycle = useRef(0);
  // Pending blink timers — cleared on unmount
  const blinkTimersRef = useRef<ReturnType<typeof setTimeout>[]>([]);
  useEffect(() => {
    return () => {
      blinkTimersRef.current.forEach(clearTimeout);
      blinkTimersRef.current = [];
    };
  }, []);
  const handleClick = useCallback(() => {
    const states: BlinkState[] = ["left-closed", "right-closed", "both-closed"];
    const next = states[cycle.current % states.length];
    cycle.current += 1;
    setState((s) => ({ ...s, blink: next, annoyed: true }));

    // Which eye(s) to blink
    const eyes = next === "both-closed" ? [leftEyeRef, rightEyeRef]
      : next === "left-closed" ? [leftEyeRef]
      : [rightEyeRef];

    // Closed geometry: height=1.5, y centered in the original [45, 45+squintHeight] span
    const openY = 45;
    const closedY = 45 + (squintHeight - 1.5) / 2;

    eyes.forEach((eyeRef) => {
      if (!eyeRef.current) return;
      const el = eyeRef.current;

      // Phase 1: close — height shrinks + y shifts down (centered). Fast, 60ms.
      animate(el, {
        height: 1.5,
        y: closedY,
        duration: 60,
        ease: "inQuad",
        onComplete: () => {
          // Phase 2: hold 50ms then re-open — height grows + y back. Smooth, 120ms.
          blinkTimersRef.current.push(setTimeout(() => {
            animate(el, {
              height: squintHeight,
              y: openY,
              duration: 120,
              ease: "outQuad",
            });
          }, 50));
        },
      });
    });

    blinkTimersRef.current.push(setTimeout(() => setState((s) => ({ ...s, blink: "open", annoyed: false })), 280));
  }, [squintHeight]);

  // Node hover → mark looks at the node + Anime.js pulse on node
  const handleNodeHover = useCallback((idx: number | null) => {
    setState((s) => ({ ...s, hoveredNode: idx }));
    // Node pulse animation handled by variant via refs (see mark-classic)
  }, []);

  return {
    state,
    handleClick,
    handleNodeHover,
    // Expose for variant to wire up
    leftEyeRef,
    rightEyeRef,
    squintHeight,
    mouseDistance,
  };
}

