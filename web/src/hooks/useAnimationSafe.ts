import { useGSAP, gsap } from "../lib/gsap";
import type { RefObject } from "react";

type AnimationCallback = (mm: gsap.MatchMedia) => void;

export function useAnimationSafe(
  callback: AnimationCallback,
  scope?: RefObject<HTMLElement | null>,
): void {
  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        callback(mm);
      });
    },
    scope ? { scope } : undefined,
  );
}
