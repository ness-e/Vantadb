"use client";

import { type ReactNode } from "react";
import { Reveal } from "./reveal";
import { cn } from "@/lib/utils";

/**
 * PageHeader — consistent hero header for Tier 2/3 route pages.
 * Manga/brutalist style: black panel, neon accent, rigid shadow.
 *
 * Usage:
 * <PageHeader
 *   badge="§ENGINE"
 *   title="Core Engine"
 *   subtitle="How the Rust engine..."
 *   tag="Rust 1.94+ · PyO3"
 * />
 */
export function PageHeader({
  badge,
  title,
  subtitle,
  tag,
  children,
}: {
  badge?: string;
  title: string;
  subtitle?: string;
  tag?: string;
  children?: ReactNode;
}) {
  return (
    <section className="relative border-b-4 border-black bg-[#FBF9F5]  ">
      <div className="pointer-events-none absolute inset-0 grid-tech opacity-40" aria-hidden />
      <div
        className="pointer-events-none absolute -right-20 -top-20 h-72 w-72 halftone halftone-fade opacity-20 "
        aria-hidden
      />
      <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6 sm:py-20">
        <Reveal direction="up">
          <div className="flex flex-col gap-3 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row sm:items-end sm:justify-between">
            <div>
              {badge && (
                <span className="inline-flex items-center gap-2 border-2 border-[#FF5500] bg-[#FF5500] px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black">
                  <span className="h-1.5 w-1.5 bg-black" />
                  {badge}
                </span>
              )}
              <h1 className="glitch-hover mt-3 font-display text-4xl uppercase leading-[0.85] sm:text-6xl lg:text-7xl">
                {title}
              </h1>
              {subtitle && (
                <p className="mt-3 max-w-2xl font-tech text-sm text-[#FBF9F5]/70 sm:text-base">
                  {subtitle}
                </p>
              )}
            </div>
            {tag && (
              <span className="flex items-center gap-2 font-tech text-[10px] uppercase tracking-wider text-[#FF5500]">
                <span className="h-2 w-2 animate-flicker bg-[#FF5500]" />
                {tag}
              </span>
            )}
          </div>
        </Reveal>
        {children && (
          <Reveal direction="up" delay={60}>
            <div className="mt-6">{children}</div>
          </Reveal>
        )}
      </div>
    </section>
  );
}

/**
 * PageSection — consistent content wrapper for route pages.
 * Alternates background for rhythm.
 */
export function PageSection({
  children,
  className,
  variant = "cream",
}: {
  children: ReactNode;
  className?: string;
  variant?: "cream" | "paper" | "ink";
}) {
  const bg =
    variant === "paper"
      ? "bg-[#F2EDE2] "
      : variant === "ink"
        ? "bg-black text-[#FBF9F5] "
        : "bg-[#FBF9F5] ";
  return (
    <section className={cn("relative border-b-4 border-black ", bg, className)}>
      <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6">{children}</div>
    </section>
  );
}
