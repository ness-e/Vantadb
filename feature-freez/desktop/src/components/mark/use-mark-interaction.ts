import { useCallback, useEffect, useRef, useState } from "react";

/**
 * Mark interaction — port of web/src/components/vanta/mark/use-mark-interaction.ts
 * WITHOUT Anime.js (VS-02 spec). Differences are declared, not silent:
 *  - follow: rAF exponential lerp  current += (target-current)*(1-exp(-dt/τ))
 *    τ ≈ 60ms eyes / 130ms sphere (spec), instead of Anime.js outQuad tween.
 *  - blink: WAAPI (Element.animate) over the y/height geometry properties
 *    instead of Anime.js. Verified animatable: SVG2 geometry properties
 *    (https://www.w3.org/TR/SVG2/geometry.html#YProperty "Animatable: yes",
 *    https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/y).
 *  - prefers-reduced-motion: no follow, no blink animation, no SMIL (SMIL is
 *    conditionally rendered by the variant). The web relies only on a CSS
 *    kill-switch which SMIL ignores — this port is stricter.
 */

export type BlinkState = "open" | "left-closed" | "right-closed" | "both-closed";

export interface MarkInteractionState {
  pupilOffset: { x: number; y: number };
  sphereOffset: { x: number; y: number };
  blink: BlinkState;
  annoyed: boolean;
  hoveredNode: number | null;
  mouseInHero: boolean;
}

/** inQuad / outQuad standard cubic-bezier approximations. */
const EASE_IN_QUAD = "cubic-bezier(0.55, 0.085, 0.68, 0.53)";
const EASE_OUT_QUAD = "cubic-bezier(0.25, 0.46, 0.45, 0.94)";
/** Time constants for the exponential follow (spec: 60ms eyes / 130ms sphere). */
const EYE_TAU_S = 0.06;
const SPHERE_TAU_S = 0.13;
/** Converged when every offset delta is below this (stops the rAF loop). */
const CONVERGED = 0.05;

export function usePrefersReducedMotion(): boolean {
  const [reduced, setReduced] = useState(
    () => typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches
  );
  useEffect(() => {
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    const onChange = () => setReduced(mq.matches);
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, []);
  return reduced;
}

export function useMarkInteraction(
  markWrapRef: React.RefObject<HTMLDivElement | null>,
  opts: {
    maxEyeOffset?: number;
    maxSphereOffset?: number;
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
    squintDistance = 600,
    maxEyeHeight = 10,
    minEyeHeight = 3,
  } = opts;

  const reduced = usePrefersReducedMotion();

  const [state, setState] = useState<MarkInteractionState>({
    pupilOffset: { x: 0, y: 0 },
    sphereOffset: { x: 0, y: 0 },
    blink: "open",
    annoyed: false,
    hoveredNode: null,
    mouseInHero: false,
  });
  const [mouseDistance, setMouseDistance] = useState(0);

  // Mutable follow state (updated by mousemove, applied by the rAF lerp).
  const target = useRef({ eyeX: 0, eyeY: 0, sphereX: 0, sphereY: 0 });
  const current = useRef({ eyeX: 0, eyeY: 0, sphereX: 0, sphereY: 0 });
  const rafId = useRef<number | null>(null);

  // Blink resources (WAAPI animations + timers) for cleanup on unmount.
  const blinkResources = useRef<{ anims: Animation[]; timers: number[] }>({ anims: [], timers: [] });
  useEffect(
    () => () => {
      if (rafId.current !== null) cancelAnimationFrame(rafId.current);
      blinkResources.current.anims.forEach((a) => a.cancel());
      blinkResources.current.timers.forEach((t) => window.clearTimeout(t));
    },
    []
  );

  const startFollow = useCallback(() => {
    if (reduced || rafId.current !== null) return;
    let last = performance.now();
    const tick = (ts: number) => {
      // Clamp dt so a background tab doesn't cause a huge jump on resume.
      const dt = Math.min((ts - last) / 1000, 0.05);
      last = ts;
      const c = current.current;
      const t = target.current;
      const eyeK = 1 - Math.exp(-dt / EYE_TAU_S);
      const sphereK = 1 - Math.exp(-dt / SPHERE_TAU_S);
      c.eyeX += (t.eyeX - c.eyeX) * eyeK;
      c.eyeY += (t.eyeY - c.eyeY) * eyeK;
      c.sphereX += (t.sphereX - c.sphereX) * sphereK;
      c.sphereY += (t.sphereY - c.sphereY) * sphereK;
      const done =
        Math.abs(t.eyeX - c.eyeX) < CONVERGED &&
        Math.abs(t.eyeY - c.eyeY) < CONVERGED &&
        Math.abs(t.sphereX - c.sphereX) < CONVERGED &&
        Math.abs(t.sphereY - c.sphereY) < CONVERGED;
      rafId.current = done ? null : requestAnimationFrame(tick);
      setState((s) => ({
        ...s,
        pupilOffset: { x: c.eyeX, y: c.eyeY },
        sphereOffset: { x: c.sphereX, y: c.sphereY },
      }));
    };
    rafId.current = requestAnimationFrame(tick);
  }, [reduced]);

  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
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
      target.current = {
        eyeX: Math.cos(angle) * eyeNorm * maxEyeOffset,
        eyeY: Math.sin(angle) * eyeNorm * maxEyeOffset,
        sphereX: Math.cos(angle) * sphereNorm * maxSphereOffset,
        sphereY: Math.sin(angle) * sphereNorm * maxSphereOffset,
      };
      setMouseDistance(dist);
      setState((s) => (s.mouseInHero ? s : { ...s, mouseInHero: true }));
      startFollow();
    },
    [markWrapRef, maxEyeOffset, maxSphereOffset, startFollow]
  );

  useEffect(() => {
    window.addEventListener("mousemove", handleMouseMove, { passive: true });
    return () => window.removeEventListener("mousemove", handleMouseMove);
  }, [handleMouseMove]);

  const leftEyeRef = useRef<SVGRectElement | null>(null);
  const rightEyeRef = useRef<SVGRectElement | null>(null);

  // Squint: close mouse → maxEyeHeight, far mouse → minEyeHeight.
  const squintNorm = Math.min(mouseDistance / squintDistance, 1);
  const squintHeight = maxEyeHeight - squintNorm * (maxEyeHeight - minEyeHeight);

  /**
   * Blink one eye via WAAPI: close 60ms inQuad → hold 50ms → open 120ms outQuad.
   * While blink !== "open" React omits the eye's y/height attributes (parity
   * with the web) so the animation owns them; on open-finish we write the final
   * y/height back as attributes, covering the 230→280ms window before React
   * restores them (otherwise the eye flashes to height 0).
   */
  const blinkEye = useCallback(
    (el: SVGRectElement) => {
      const openY = 45;
      const closedY = 45 + (squintHeight - 1.5) / 2;
      // Read the actual DOM geometry (React hasn't re-rendered yet at click).
      const fromY = Number(el.getAttribute("y") ?? openY);
      const fromH = Number(el.getAttribute("height") ?? squintHeight);
      const close = el.animate(
        [{ y: fromY, height: fromH }, { y: closedY, height: 1.5 }],
        { duration: 60, easing: EASE_IN_QUAD, fill: "both" }
      );
      blinkResources.current.anims.push(close);
      close.onfinish = () => {
        const timer = window.setTimeout(() => {
          const open = el.animate(
            [{ y: closedY, height: 1.5 }, { y: openY, height: squintHeight }],
            { duration: 120, easing: EASE_OUT_QUAD, fill: "both" }
          );
          blinkResources.current.anims.push(open);
          open.onfinish = () => {
            el.setAttribute("y", String(openY));
            el.setAttribute("height", String(squintHeight));
          };
        }, 50);
        blinkResources.current.timers.push(timer);
      };
    },
    [squintHeight]
  );

  const cycle = useRef(0);
  const handleClick = useCallback(() => {
    if (reduced) {
      // Reduced motion: no blink animation, no state change that would make
      // React omit the eye attributes. Keep the hint/annoyed flip only.
      setState((s) => ({ ...s, annoyed: true }));
      const timer = window.setTimeout(() => setState((s) => ({ ...s, annoyed: false })), 280);
      blinkResources.current.timers.push(timer);
      return;
    }
    const states: BlinkState[] = ["left-closed", "right-closed", "both-closed"];
    const next = states[cycle.current % states.length];
    cycle.current += 1;
    setState((s) => ({ ...s, blink: next, annoyed: true }));

    const eyes =
      next === "both-closed" ? [leftEyeRef, rightEyeRef] : next === "left-closed" ? [leftEyeRef] : [rightEyeRef];
    eyes.forEach((ref) => {
      if (ref.current) blinkEye(ref.current);
    });

    const timer = window.setTimeout(() => setState((s) => ({ ...s, blink: "open", annoyed: false })), 280);
    blinkResources.current.timers.push(timer);
  }, [blinkEye, reduced]);

  const handleNodeHover = useCallback((idx: number | null) => {
    setState((s) => ({ ...s, hoveredNode: idx }));
  }, []);

  return {
    state,
    handleClick,
    handleNodeHover,
    leftEyeRef,
    rightEyeRef,
    squintHeight,
    mouseDistance,
  };
}