"use client";

import { useEffect } from "react";

/**
 * App Router error boundary (Next.js 16).
 * Catches runtime errors in the route segment and renders a branded fallback.
 * `reset()` re-renders the segment; `error.digest` aids bug reports.
 */
export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    console.error(error);
  }, [error]);

  return (
    <main className="grid min-h-screen place-items-center bg-background px-6 text-foreground">
      <div className="max-w-lg border-4 border-black bg-[#FBF9F5] p-8 shadow-[6px_6px_0_0_#000]">
        <p className="font-tech text-xs uppercase tracking-wider text-[#FF5500]">
          500 · Error
        </p>
        <h1 className="mt-3 font-display text-4xl uppercase">Algo salió mal</h1>
        <p className="mt-4 text-sm text-black/60">
          Ocurrió un error inesperado al renderizar esta página. Podés intentar
          de nuevo o volver al inicio.
        </p>
        {error.digest ? (
          <p className="mt-4 font-tech text-[10px] uppercase tracking-wider text-black/40">
            digest: {error.digest}
          </p>
        ) : null}
        <div className="mt-6 flex flex-wrap gap-4">
          <button
            type="button"
            onClick={reset}
            className="border-4 border-black bg-[#FF5500] px-6 py-3 font-tech text-xs uppercase tracking-wider text-black shadow-[4px_4px_0_0_#000] transition-transform hover:-translate-y-0.5 active:translate-y-0 active:shadow-none"
          >
            Reintentar
          </button>
          <a
            href="/"
            className="border-4 border-black bg-[#FBF9F5] px-6 py-3 font-tech text-xs uppercase tracking-wider text-black shadow-[4px_4px_0_0_#000] transition-transform hover:-translate-y-0.5 active:translate-y-0 active:shadow-none"
          >
            Volver al inicio
          </a>
        </div>
      </div>
    </main>
  );
}
