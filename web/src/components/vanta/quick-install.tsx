"use client";

import { useState } from "react";
import { Copy, Check, Terminal } from "lucide-react";
import { copyToClipboard } from "./copy-utils";
import { toast } from "./toast";

const INSTALL_CMD = "pip install vantadb-py";

interface QuickInstallProps {
  id?: string;
  variant?: "hero" | "bar" | "docs";
  className?: string;
}

/**
 * QuickInstall — bloque instalación copiable arriba del fold.
 * Contrato WEB-06: comando visible sin scroll en 1440×900 + copy button funcional.
 * No recrea ruta /quickstart — solo anclas #quickstart / #install.
 * ponytail: naive single-command bar, tabs/additional managers when 2+ requested.
 */
export function QuickInstall({ id = "quickstart", variant = "bar", className }: QuickInstallProps) {
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    const ok = await copyToClipboard(INSTALL_CMD);
    if (ok) {
      setCopied(true);
      toast.copy(INSTALL_CMD);
      setTimeout(() => setCopied(false), 1600);
    } else {
      toast.error("No se pudo copiar");
    }
  };

  if (variant === "hero") {
    return (
      <div
        id={id}
        data-testid="quick-install"
        role="region"
        aria-label="Instalación rápida"
        className={className}
      >
        <button
          onClick={copy}
          aria-label="Copiar comando pip install vantadb-py"
          title="Copiar comando de instalación"
          data-testid="quick-install-copy"
          className="group inline-flex items-center gap-3 border-4 border-black bg-black px-4 py-3 font-tech text-sm font-bold text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500,6px_6px_0_2px_#000] transition-all hover:translate-x-[2px] hover:translate-y-[2px] hover:shadow-[4px_4px_0_0_#FF5500,4px_4px_0_2px_#000] active:translate-x-[6px] active:translate-y-[6px] active:shadow-none"
        >
          <span className="text-[#FF5500]">$</span>
          <span>{INSTALL_CMD}</span>
          {copied ? (
            <Check className="h-4 w-4 shrink-0 text-[#FF5500]" aria-hidden />
          ) : (
            <Copy className="h-4 w-4 shrink-0 opacity-60 transition-opacity group-hover:opacity-100" aria-hidden />
          )}
        </button>
      </div>
    );
  }

  // bar / docs variant — terminal strip
  return (
    <section
      id={id}
      data-testid="quick-install"
      role="region"
      aria-label="Instalación rápida"
      className={className}
    >
      <div className="flex flex-wrap items-center gap-3 border-4 border-black bg-black px-4 py-3 shadow-[6px_6px_0_0_#000] sm:px-5">
        <span className="inline-flex items-center gap-2 font-tech text-xs font-bold uppercase tracking-wider text-[#FBF9F5]/70">
          <Terminal className="h-4 w-4 text-[#FF5500]" strokeWidth={2.5} aria-hidden />
          <span className="hidden sm:inline">Quickstart</span>
        </span>
        <code className="flex-1 break-all font-mono text-sm font-bold text-[#FBF9F5] sm:text-[15px]">
          <span className="text-[#FF5500]">$</span> {INSTALL_CMD}
        </code>
        <button
          onClick={copy}
          aria-label="Copiar comando pip install vantadb-py"
          title="Copiar"
          data-testid="quick-install-copy"
          className="inline-flex h-11 min-h-11 min-w-11 shrink-0 items-center justify-center gap-1.5 border-2 border-[#FBF9F5]/20 bg-[#FBF9F5]/10 px-3 font-tech text-xs font-bold uppercase tracking-wider text-[#FBF9F5] transition-colors hover:bg-[#FF5500] hover:text-black hover:border-black active:translate-y-px"
        >
          {copied ? <Check className="h-4 w-4 text-[#FF5500]" strokeWidth={3} aria-hidden /> : <Copy className="h-4 w-4" strokeWidth={2.5} aria-hidden />}
          <span className="hidden sm:inline">{copied ? "Copied" : "Copy"}</span>
        </button>
      </div>
      {variant === "docs" && (
        <p className="mt-2 font-tech text-[11px] text-black/60">
          Distribución <code className="bg-black px-1 py-0.5 text-[#FF5500]">vantadb-py</code> · import{" "}
          <code className="bg-black px-1 py-0.5 text-[#FBF9F5]">import vantadb</code>
        </p>
      )}
    </section>
  );
}

// Re-export constant for tests / docs
export const QUICK_INSTALL_CMD = INSTALL_CMD;
