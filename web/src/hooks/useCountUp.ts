import { useEffect, useRef, useState } from "react";

interface CountUpOptions {
  from?: number;
  duration?: number;
  easing?: (t: number) => number;
  disabled?: boolean;
}

function easeOutCubic(t: number): number {
  return 1 - Math.pow(1 - t, 3);
}

export function useCountUp(
  end: number,
  triggerRef: React.RefObject<Element | null>,
  options: CountUpOptions = {},
): number {
  const { from = 0, duration = 1.2, easing = easeOutCubic, disabled = false } = options;
  const [value, setValue] = useState(disabled ? end : from);
  const rafRef = useRef<number>(0);
  const startTimeRef = useRef<number>(0);

  useEffect(() => {
    if (disabled) {
      setValue(end);
      return;
    }

    const el = triggerRef.current;
    if (!el) return;

    let observer: IntersectionObserver | null = null;

    const startAnimation = () => {
      startTimeRef.current = performance.now();
      const animate = (now: number) => {
        const elapsed = (now - startTimeRef.current) / 1000;
        const t = Math.min(elapsed / duration, 1);
        const current = from + (end - from) * easing(t);
        setValue(current);
        if (t < 1) {
          rafRef.current = requestAnimationFrame(animate);
        } else {
          setValue(end);
        }
      };
      rafRef.current = requestAnimationFrame(animate);
    };

    if ("IntersectionObserver" in window) {
      observer = new IntersectionObserver(
        (entries) => {
          if (entries[0]?.isIntersecting) {
            startAnimation();
            observer?.disconnect();
          }
        },
        { threshold: 0.3 },
      );
      observer.observe(el);
    } else {
      startAnimation();
    }

    return () => {
      cancelAnimationFrame(rafRef.current);
      observer?.disconnect();
    };
  }, [end, from, duration, easing, disabled, triggerRef]);

  return value;
}
