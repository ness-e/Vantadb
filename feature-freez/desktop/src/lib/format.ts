// Shared formatting helpers (UX-15).
//
// Antes existían 3 formatters de bytes locales: indices-core.ts (decimal,
// el canónico testado en indices-core.test.ts), MetricsGrid.tsx (decimal
// idéntico) y KpiCards.tsx (binario MiB/KiB). Este es el ÚNICO fmtBytes —
// decimal, sin cambiar la semántica del sitio canónico. KpiCards migra de
// binario a decimal: esa unificación ES el cambio pedido por el ticket.

export function fmtBytes(n: number): string {
  if (n >= 1e9) return `${(n / 1e9).toFixed(2)} GB`;
  if (n >= 1e6) return `${(n / 1e6).toFixed(1)} MB`;
  if (n >= 1e3) return `${(n / 1e3).toFixed(0)} KB`;
  return `${n} B`;
}