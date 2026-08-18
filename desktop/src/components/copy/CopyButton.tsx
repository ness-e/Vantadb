// CopyButton (VS-17): botón copy-as con feedback "copiado" (~1.5s).
// `getText` se evalúa en el click — el record puede refrescarse sin re-render
// del botón. Reutilizado en grid (JSON) e inspector (JSON / KEY / MD).
import { useState } from "react";
import { vantaErrorMessage } from "../../vanta";
import { copyText } from "./copy-as";

export function CopyButton({
  getText,
  label,
  title,
  onError,
  className,
}: {
  getText: () => string;
  /** Contenido del botón (ícono o texto corto). */
  label: string;
  /** Tooltip + aria-label (a11y — el feedback visual no es el único canal). */
  title: string;
  onError: (msg: string) => void;
  className?: string;
}) {
  const [copied, setCopied] = useState(false);

  async function handleCopy() {
    try {
      await copyText(getText());
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    } catch (err) {
      onError(vantaErrorMessage(err));
    }
  }

  return (
    <button
      type="button"
      onClick={handleCopy}
      title={title}
      aria-label={title}
      className={`press flex items-center justify-center border-2 border-foreground text-[10px] ${
        copied ? "bg-neon text-background" : "bg-background"
      } ${className ?? ""}`}
    >
      {copied ? "✓ copiado" : label}
    </button>
  );
}