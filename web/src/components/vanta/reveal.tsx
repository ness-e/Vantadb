"use client";

import { type ReactNode } from "react";
import { useReveal } from "@/hooks/use-reveal";
import { cn } from "@/lib/utils";

type RevealDirection = "up" | "down" | "left" | "right" | "scale" | "fade";

const DIRECTION_CLASSES: Record<RevealDirection, { hidden: string; shown: string }> = {
  up: {
    hidden: "translate-y-6 opacity-0",
    shown: "translate-y-0 opacity-100",
  },
  down: {
    hidden: "-translate-y-6 opacity-0",
    shown: "translate-y-0 opacity-100",
  },
  left: {
    hidden: "translate-x-8 opacity-0",
    shown: "translate-x-0 opacity-100",
  },
  right: {
    hidden: "-translate-x-8 opacity-0",
    shown: "translate-x-0 opacity-100",
  },
  scale: {
    hidden: "scale-90 opacity-0",
    shown: "scale-100 opacity-100",
  },
  fade: {
    hidden: "opacity-0",
    shown: "opacity-100",
  },
};

export function Reveal({
  children,
  direction = "up",
  delay = 0,
  duration = 600,
  className,
  as: Tag = "div",
  threshold,
  rootMargin,
  once,
}: {
  children: ReactNode;
  direction?: RevealDirection;
  delay?: number;
  duration?: number;
  className?: string;
  as?: "div" | "section" | "article" | "li" | "span";
  threshold?: number;
  rootMargin?: string;
  once?: boolean;
}) {
  const { ref, visible } = useReveal<HTMLDivElement>({ threshold, rootMargin, once });
  const dirs = DIRECTION_CLASSES[direction];

  return (
    <Tag
      ref={ref as never}
      className={cn(
        "transition-[transform,opacity] ease-[cubic-bezier(0.2,0.8,0.2,1)] will-change-transform",
        visible ? dirs.shown : dirs.hidden,
        className
      )}
      style={{
        transitionDelay: `${delay}ms`,
        transitionDuration: `${duration}ms`,
      }}
    >
      {children}
    </Tag>
  );
}
