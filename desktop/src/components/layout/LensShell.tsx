// LensShell (UX-01): encabezado estándar de lente del Studio — título stencil
// (icono opcional) + meta derecho con token `.label-tech` + subtítulo opcional.
// Reemplaza el header duplicado que cada lente repetía (Consolidate/Indices/
// Retrieval/Graph/Space/Memory — auditoría shell P2). Composable: NO envuelve
// children; cada lente lo coloca como primer hijo de su layout, así el layout
// full-height WebGL de Graph/Space queda intacto.
//
// Tokens: font-display/text-stencil para el título, `.label-tech` (11px Space
// Mono uppercase, tracking 0.1em) para el meta — unifica el patrón
// `font-tech text-[10px] uppercase tracking-widest` que se repetía a mano.
import type { ReactNode } from "react";

interface LensShellProps {
  /** Título stencil (p.ej. "CONSOLIDAR"). */
  title: string;
  /** Glifo/icono monocromo antes del título (identidad linocut, p.ej. "⇄"). */
  icon?: ReactNode;
  /** Etiqueta técnica derecha — token `.label-tech` + text-muted-foreground. */
  meta?: ReactNode;
  /** Línea de subtítulo bajo el título (p.ej. explicación de la lente). */
  subtitle?: ReactNode;
}

export default function LensShell({ title, icon, meta, subtitle }: LensShellProps) {
  return (
    <div className="flex flex-wrap items-center justify-between gap-2 border-b-4 border-foreground pb-2">
      <div className="flex items-center gap-2">
        {icon && <span className="text-neon">{icon}</span>}
        <h2 className="font-display text-2xl text-stencil">{title}</h2>
      </div>
      {meta && <span className="label-tech text-muted-foreground">{meta}</span>}
      {subtitle && (
        <p className="w-full font-tech text-[11px] text-muted-foreground">{subtitle}</p>
      )}
    </div>
  );
}