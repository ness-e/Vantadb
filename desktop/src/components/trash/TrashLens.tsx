// Papelera (VS-08, Fix 4): lens mínimo de tombstones de sesión. Lista los
// records soft-deleted (snapshot local) con Restore (vantaPut del snapshot) y
// Eliminar definitivo (descarta el tombstone — confirmación explícita P6).
// Se suscribe al store vanilla (no zustand aún — VS-09 lo instala en paralelo).
import { useEffect, useState } from "react";
import { vantaErrorMessage } from "../../vanta";
import { Tombstone, undoStore } from "../../store/undo";
import { tp, tt, type DesktopLang } from "../../i18n";
import { connectionPrefs } from "../../store/connections";

interface Props {
  onNotice: (msg: string) => void;
  onError: (msg: string) => void;
  lang?: DesktopLang;
}

function relTime(ms: number, now: number, lang: DesktopLang): string {
  const diff = Math.max(0, now - ms);
  const m = Math.floor(diff / 60_000);
  if (m < 1) return tt(lang, "trash.relNow", "now");
  if (m < 60) return tp(lang, "trash.relMin", "{m}m ago", { m: String(m) });
  const h = Math.floor(m / 60);
  if (h < 24) return tp(lang, "trash.relHour", "{h}h ago", { h: String(h) });
  return tp(lang, "trash.relDay", "{d}d ago", { d: String(Math.floor(h / 24)) });
}

export default function TrashLens({ onNotice, onError, lang = connectionPrefs.get().lang ?? "es" }: Props) {
  const [trash, setTrash] = useState<Tombstone[]>(undoStore.getTrash());
  const [busyKey, setBusyKey] = useState<string | null>(null);
  const [confirmPurge, setConfirmPurge] = useState<string | null>(null);

  useEffect(() => undoStore.subscribe(() => setTrash(undoStore.getTrash())), []);

  async function handleRestore(t: Tombstone) {
    const key = `${t.record.namespace}:${t.record.id}`;
    setBusyKey(key);
    try {
      await undoStore.restore(t);
      onNotice(tp(lang, "trash.restored", "restaurado {id}", { id: t.record.id }));
    } catch (err) {
      onError(vantaErrorMessage(err));
    } finally {
      setBusyKey(null);
    }
  }

  function handlePurge(t: Tombstone) {
    undoStore.purge(t);
    setConfirmPurge(null);
    onNotice(tp(lang, "trash.purged", "eliminado definitivamente {id}", { id: t.record.id }));
  }

  return (
    <section className="press-lg border-4 border-foreground bg-card" aria-label={tt(lang, "trash.aria", "Papelera")}>
      <div className="border-b-4 border-foreground p-4">
        <div className="flex flex-wrap items-baseline justify-between gap-2">
          <h2 className="font-display text-3xl text-stencil">PAPELERA</h2>
          <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
            {tp(lang, "trash.meta", "{n} tombstone(s) · Ctrl+Z deshace", { n: String(trash.length) })}
          </span>
        </div>
        <p className="mt-1 font-tech text-[11px] text-muted-foreground">
          {tt(lang, "trash.hint", "records eliminados en esta sesión — Restore vuelve a escribir el snapshot (vantaPut)")}
        </p>
      </div>

      {trash.length === 0 ? (
        <p className="p-8 text-center font-tech text-[11px] uppercase tracking-widest text-muted-foreground">
          {tt(lang, "trash.empty", "papelera vacía")}
        </p>
      ) : (
        <ul className="divide-y-4 divide-foreground">
          {trash.map((t) => {
            const key = `${t.record.namespace}:${t.record.id}`;
            const busy = busyKey === key;
            return (
              <li key={key} className="flex items-center gap-3 p-4">
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <code className="truncate font-tech text-sm">{t.record.id}</code>
                    <span className="shrink-0 border-2 border-foreground bg-background px-1.5 py-0.5 font-tech text-[10px]">
                      {t.record.namespace}
                    </span>
                    <span className="shrink-0 font-tech text-[10px] text-muted-foreground">
                      {relTime(t.deletedAtMs, Date.now(), lang)}
                    </span>
                    {t.record.version != null && (
                      <span className="shrink-0 border-2 border-neon px-1 font-tech text-[10px] text-neon">
                        v{t.record.version}
                      </span>
                    )}
                  </div>
                  <p className="mt-0.5 truncate text-[13px] opacity-70">{t.record.text}</p>
                </div>

                <button
                  type="button"
                  onClick={() => handleRestore(t)}
                  disabled={busy}
                  className="press shrink-0 border-2 border-foreground bg-background px-3 py-2 text-xs font-semibold"
                  title={tt(lang, "trash.restoreTitle", "Restaurar con vantaPut (Ctrl+Z lo vuelve a borrar)")}
                >
                  {busy ? "…" : tt(lang, "trash.restore", "↩ RESTORE")}
                </button>

                {confirmPurge === key ? (
                  <span className="flex shrink-0 items-center gap-2">
                    <button
                      type="button"
                      onClick={() => handlePurge(t)}
                      className="press border-2 border-foreground bg-neon px-3 py-2 text-xs font-bold text-background"
                    >
                      {tt(lang, "trash.purgeConfirm", "¿BORRAR?")}
                    </button>
                    <button
                      type="button"
                      onClick={() => setConfirmPurge(null)}
                      className="press flex h-8 w-8 items-center justify-center border-2 border-foreground text-xs"
                      aria-label={tt(lang, "trash.purgeCancelAria", "Cancelar eliminación definitiva")}
                    >
                      ✕
                    </button>
                  </span>
                ) : (
                  <button
                    type="button"
                    onClick={() => setConfirmPurge(key)}
                    className="press shrink-0 border-2 border-foreground bg-background px-3 py-2 text-xs"
                    title={tt(lang, "trash.purgeTitle", "Descartar el snapshot — no se puede restaurar (Ctrl+Z lo deshace)")}
                  >
                    {tt(lang, "trash.purge", "BORRAR DEF.")}
                  </button>
                )}
              </li>
            );
          })}
        </ul>
      )}
    </section>
  );
}