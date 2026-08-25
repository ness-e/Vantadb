// DESKTOP-33: confirmación destructiva en 2 pasos (mismo patrón que
// NamespaceDialog): paso 1 = aviso + elección papelera/permanente,
// paso 2 = tipear el objetivo exacto (`ns/id` del único target, o CONFIRMAR
// en lote) para habilitar el botón. Papelera es el default (undo vía Ctrl+Z).
// UX-09: unificado al patrón inline de la app — se renderiza en flujo (caja),
// sin overlay fixed (antes era el 3er lenguaje de confirmación: modal).
import { FormEvent, useEffect, useRef, useState } from "react";
import type { PairRecord } from "./consolidate-core";

const BTN_CANCEL =
  "press border-2 border-foreground bg-background px-3 py-1.5 font-tech text-[11px] uppercase tracking-widest";
const BTN_OK =
  "press border-2 border-foreground bg-neon px-3 py-1.5 font-tech text-[11px] uppercase tracking-widest font-bold text-background disabled:opacity-40";

export default function ConfirmDiscard({
  targets,
  busy,
  onClose,
  onConfirm,
}: {
  targets: PairRecord[];
  busy: boolean;
  onClose: () => void;
  onConfirm: (mode: "trash" | "purge") => Promise<void>;
}) {
  const [step, setStep] = useState<1 | 2>(1);
  const [value, setValue] = useState("");
  const [mode, setMode] = useState<"trash" | "purge">("trash");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
    inputRef.current?.select();
  }, [step]);

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const expected =
    targets.length === 1 ? `${targets[0].namespace}/${targets[0].id}` : "CONFIRMAR";
  const mismatch = step === 2 && value.trim() !== expected;

  async function submit(e: FormEvent) {
    e.preventDefault();
    if (busy || mismatch) return;
    if (step === 1) {
      setStep(2);
      return;
    }
    await onConfirm(mode);
  }

  return (
    // UX-09: inline en flujo (sin overlay fixed) — misma caja y lenguaje visual
    // que el resto de la app; el contenedor de ConsolidateLens lo coloca al pie.
    <div className="mt-4">
      <form
        role="region"
        aria-label="Confirmar eliminación"
        onSubmit={submit}
        className="w-full border-4 border-foreground bg-card p-5 shadow-[4px_4px_0_0_#000]"
      >
        <div className="font-display text-2xl text-stencil">
          Eliminar {targets.length === 1 ? "registro" : `${targets.length} registros`}?
        </div>

        {step === 1 ? (
          <>
            <ul className="mt-2 max-h-32 space-y-0.5 overflow-y-auto border-2 border-foreground bg-background p-2 font-tech text-[10px]">
              {targets.map((t) => (
                <li key={t.id}>
                  {t.namespace}/{t.id}
                </li>
              ))}
            </ul>
            <div className="mt-3 space-y-1 text-xs">
              <label className="flex items-center gap-2">
                <input
                  type="radio"
                  name="discard-mode"
                  checked={mode === "trash"}
                  onChange={() => setMode("trash")}
                />
                Mover a papelera (recuperable con Ctrl+Z o desde PAPELERA)
              </label>
              <label className="flex items-center gap-2">
                <input
                  type="radio"
                  name="discard-mode"
                  checked={mode === "purge"}
                  onChange={() => setMode("purge")}
                />
                Eliminar permanente (sin undo)
              </label>
            </div>
          </>
        ) : (
          <>
            <p className="mt-2 text-sm">
              Escribí <span className="font-bold">{expected}</span> para confirmar.
            </p>
            <input
              ref={inputRef}
              value={value}
              onChange={(e) => setValue(e.target.value)}
              placeholder={expected}
              aria-label="Confirmar eliminación"
              className={`mt-3 w-full border-2 px-3 py-1.5 text-sm ${
                mismatch && value !== "" ? "border-red-500 bg-background" : "border-foreground bg-background"
              }`}
            />
            {mismatch && value !== "" && (
              <p className="mt-1 font-tech text-[10px] uppercase tracking-widest text-red-500">
                no coincide — esta acción es destructiva
              </p>
            )}
          </>
        )}

        <div className="mt-4 flex justify-end gap-2">
          <button type="button" onClick={onClose} className={BTN_CANCEL}>
            cancelar
          </button>
          <button type="submit" disabled={busy || mismatch} className={BTN_OK}>
            {step === 1 ? "continuar" : mode === "trash" ? "a papelera" : "eliminar"}
          </button>
        </div>
      </form>
    </div>
  );
}
