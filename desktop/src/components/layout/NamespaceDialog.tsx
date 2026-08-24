// DESKTOP-32: modal único de CRUD de namespaces desde la sidebar.
// Tres modos sobre el mismo form: crear (input vacío), renombrar (input
// precargado + validación de colisión) y borrar (confirmación en 2 pasos:
// aviso → escribir el nombre exacto para habilitar el botón).
import { FormEvent, useEffect, useRef, useState } from "react";

export type NsDialog =
  | { mode: "create" }
  | { mode: "rename"; name: string }
  | { mode: "delete"; name: string };

interface Props {
  dialog: NsDialog;
  /** Namespaces ya existentes — bloquea crear/renombrar a uno duplicado. */
  existing: string[];
  onClose: () => void;
  onCreate: (name: string) => Promise<void>;
  onRename: (from: string, to: string) => Promise<void>;
  onDelete: (name: string) => Promise<void>;
}

const BTN_CANCEL =
  "press border-2 border-foreground bg-background px-3 py-1.5 font-tech text-[11px] uppercase tracking-widest";
const BTN_OK =
  "press border-2 border-foreground bg-neon px-3 py-1.5 font-tech text-[11px] uppercase tracking-widest font-bold text-background disabled:opacity-40";

export default function NamespaceDialog({
  dialog,
  existing,
  onClose,
  onCreate,
  onRename,
  onDelete,
}: Props) {
  const destructive = dialog.mode === "delete";
  // Borrar es 2 pasos: 1 = aviso, 2 = tipear el nombre exacto (input vacío:
  // el gate real es tipearlo, no apretar un botón ya habilitado).
  const [step, setStep] = useState<1 | 2>(1);
  const [value, setValue] = useState(dialog.mode === "create" || dialog.mode === "delete" ? "" : dialog.name);
  const [busy, setBusy] = useState(false);
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

  const name = value.trim();
  const taken = existing.includes(name);
  const invalid =
    !name ||
    (dialog.mode === "create" && taken) ||
    (dialog.mode === "rename" && (taken || name === dialog.name));
  // Paso 2 de borrado: el botón queda deshabilitado hasta coincidencia exacta.
  const mismatch = destructive && step === 2 && name !== dialog.name;

  const title =
    dialog.mode === "create"
      ? "Nuevo namespace"
      : dialog.mode === "rename"
        ? `Renombrar "${dialog.name}"`
        : `Borrar "${dialog.name}"`;

  async function submit(e: FormEvent) {
    e.preventDefault();
    if (busy || invalid || mismatch) return;
    if (destructive) {
      if (step === 1) {
        setStep(2);
        return;
      }
      setBusy(true);
      await onDelete(dialog.name);
      setBusy(false);
      onClose();
      return;
    }
    setBusy(true);
    if (dialog.mode === "create") {
      await onCreate(name);
    } else {
      await onRename(dialog.name, name);
    }
    setBusy(false);
    onClose();
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-background/80 p-4"
      onClick={onClose}
    >
      <form
        role="dialog"
        aria-modal="true"
        aria-label={title}
        onSubmit={submit}
        onClick={(e) => e.stopPropagation()}
        className="w-full max-w-sm border-4 border-foreground bg-card p-5 shadow-[4px_4px_0_0_#000]"
      >
        <div className="font-display text-2xl text-stencil">{title}</div>

        {destructive && step === 1 ? (
          <p className="mt-2 text-sm">
            Se moverán todos sus registros a la papelera. Podés recuperarlos con
            Ctrl+Z o desde la superficie PAPELERA.
          </p>
        ) : (
          <>
            <input
              ref={inputRef}
              value={value}
              onChange={(e) => setValue(e.target.value)}
              placeholder={
                destructive
                  ? `Escribí "${dialog.name}" para confirmar`
                  : "Nombre del namespace"
              }
              aria-label={destructive ? "Confirmar nombre del namespace" : "Nombre del namespace"}
              className={`mt-3 w-full border-2 px-3 py-1.5 text-sm ${
                mismatch || (name !== "" && invalid)
                  ? "border-red-500 bg-background"
                  : "border-foreground bg-background"
              }`}
            />
            {name !== "" && taken && !destructive && (
              <p className="mt-1 font-tech text-[10px] uppercase tracking-widest text-red-500">
                ya existe un namespace con ese nombre
              </p>
            )}
            {mismatch && (
              <p className="mt-1 font-tech text-[10px] uppercase tracking-widest text-red-500">
                el nombre no coincide — esta acción es destructiva
              </p>
            )}
          </>
        )}

        <div className="mt-4 flex justify-end gap-2">
          <button type="button" onClick={onClose} className={BTN_CANCEL}>
            cancelar
          </button>
          <button type="submit" disabled={invalid || mismatch || busy} className={BTN_OK}>
            {destructive ? (step === 1 ? "continuar" : "borrar") : dialog.mode === "create" ? "crear" : "renombrar"}
          </button>
        </div>
      </form>
    </div>
  );
}
