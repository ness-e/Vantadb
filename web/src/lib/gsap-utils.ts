import { gsap } from "./gsap";

export const FADE_UP = { opacity: 0, y: 24 } as const;
export const FADE_UP_TO = {
  opacity: 1,
  y: 0,
  duration: 0.35,
  ease: "cubic-bezier(0.05, 0.95, 0.3, 1)",
} as const;

export function fadeUp(targets: gsap.TweenTarget, options?: { stagger?: number; delay?: number }) {
  return gsap.fromTo(targets, FADE_UP, {
    ...FADE_UP_TO,
    stagger: options?.stagger ?? 0.08,
    delay: options?.delay ?? 0,
  });
}

export function scrollTriggerConfig(
  trigger: Element | null,
  startPct = 75,
): gsap.plugins.ScrollTriggerInstanceVars {
  return {
    trigger,
    start: `top ${startPct}%`,
  };
}
