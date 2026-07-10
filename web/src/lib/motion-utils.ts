import { animate, type AnimationPlaybackControls } from "motion";

export const FADE_UP = { opacity: 0, y: 24 } as const;
export const FADE_UP_TO = {
  opacity: 1,
  y: 0,
  duration: 0.35,
  ease: [0.05, 0.95, 0.3, 1],
} as const;

export function fadeUp(
  target: Element | Element[] | NodeListOf<Element>,
  options?: { stagger?: number; delay?: number },
): AnimationPlaybackControls | AnimationPlaybackControls[] {
  const targets = target instanceof Element ? [target] : Array.from(target);
  return targets.map((el, i) =>
    animate(
      el,
      { opacity: [0, 1], y: [24, 0] },
      {
        duration: 0.35,
        delay: (options?.delay ?? 0) + i * (options?.stagger ?? 0.08),
        ease: [0.05, 0.95, 0.3, 1],
      },
    ),
  );
}
