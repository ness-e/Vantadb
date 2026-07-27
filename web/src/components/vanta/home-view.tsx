"use client";

import type { View } from "./vanta-data";
// Secciones conservadas (arco narrativo profesional, 11 secciones)
import { Hero } from "./hero";
import { TrustBar } from "./trust-bar";
import { Features } from "./features";
import { CoreEngine } from "./core-engine";
import { CodeTerminal } from "./code-terminal";
import { LatencyComparator } from "./latency-comparator";
import { UseCases } from "./use-cases";
import { TutorialsSection } from "./tutorials-section";
import { FaqSection } from "./faq-section";
import { TrustSection } from "./trust-section";
import { CtaFinal } from "./cta-final";
// Estructura
import { InkDivider } from "./ink-divider";

/**
 * HomeView — 11 secciones con arco narrativo profesional (sin redundancias).
 *
 * CONSOLIDACIÓN (tras feedback de usuario "se repite mucha información"):
 *  - Eliminada MetricsBar §03 (4 de 5 stats duplicaban Hero)
 *  - Eliminada Architecture §07 (pipeline duplicaba CoreEngine)
 *  - Eliminada Ecosystem §10 standalone (tech stack ya cubierto por TrustSection)
 *  - CtaFinal §13: removidos stats recap (duplicaban Hero por 3ª vez)
 *  - TrustSection §★: removido CTA GitHub (duplicaba Hero + CtaFinal), reubicada
 *
 * Arco narrativo:
 *  01 Hook (Hero) → 02 Social proof (TrustBar) → 03 Qué hace (Features) →
 *  04 Cómo funciona (CoreEngine) → 05 Vélo en acción (CodeTerminal) →
 *  06 Prueba de performance (LatencyComparator) → 07 Para quién (UseCases) →
 *  08 Profundiza (Tutorials) → 09 Resuelve dudas (FAQ) →
 *  10 Credibilidad final (TrustSection) → 11 CTA Final (CtaFinal)
 *
 * Nota: MetricsBar, Architecture, Ecosystem, ChangelogSection se conservan como
 * archivos para reutilizar en rutas /architecture, /changelog etc. (Fase 1).
 */
export function HomeView({ onNavigate }: { onNavigate: (v: View) => void }) {
  return (
    <div className="animate-rise">
      {/* 01 — Hook: mascota + headline stats + install + CTAs */}
      <Hero onNavigate={onNavigate} />

      {/* 02 — Social proof strip: "se integra con tu stack de IA" */}
      <TrustBar />

      {/* 03 — Qué hace: 6 Core Capabilities */}
      <Features />
      <InkDivider />

      {/* 04 — Cómo funciona: pipeline del motor (ÚNICA visualización del pipeline) */}
      <CoreEngine />
      <InkDivider />

      {/* 05 — Vélo en acción: Quickstart Python con typing animation */}
      <CodeTerminal />
      <InkDivider />

      {/* 06 — Prueba de performance: Latency Explorer + benchmarks */}
      <LatencyComparator />
      <InkDivider />

      {/* 07 — Para quién: AI Agents, Local RAG, IDE Tooling */}
      <UseCases />
      <InkDivider />

      {/* 08 — Profundiza: 4 tutoriales interactivos + modal */}
      <TutorialsSection onNavigate={onNavigate} />
      <InkDivider />

      {/* 09 — Resuelve dudas: FAQ accordion */}
      <FaqSection />
      <InkDivider />

      {/* 10 — Credibilidad final: licencia, Rust, tech stack */}
      <TrustSection />
      <InkDivider />

      {/* 11 — CTA Final: Vector Nebula + CTAs (sin stats duplicados) */}
      <CtaFinal />
    </div>
  );
}
