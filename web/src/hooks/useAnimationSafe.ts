import type { RefObject } from "react";
import { useEffect, useRef } from "react";
import { inView } from "motion";
import { useReducedMotion } from "./useReducedMotion";

type AnimationCallback = () => void;

export function useAnimationSafe(
  callback: AnimationCallback,
  scope?: RefObject<HTMLElement | null>,
): void {
  const calledRef = useRef(false);
  const reducedMotion = useReducedMotion();

  useEffect(() => {
    const el = scope?.current;
    if (!el) {
      callback();
      return;
    }

    if (reducedMotion) return;

    const cleanup = inView(
      el,
      () => {
        if (calledRef.current) return;
        calledRef.current = true;
        callback();
      },
      { amount: 0.2 },
    );

    return () => {
      cleanup?.();
      calledRef.current = false;
    };
  }, [callback, scope, reducedMotion]);
}
