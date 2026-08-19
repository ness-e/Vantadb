// Undo + papelera (VS-08, Fix 4 — P8 recuperación de errores, Norman).
//
// Store VANILLA con suscripción (zustand lo instala VS-09 en paralelo; si el
// día de mañana se migra, la API pública es la misma: getTrash/subscribe/
// softDelete/restore/purge/undo).
//
// Modelo: cada mutación destructiva guarda un ENTRY con (a) un snapshot de la
// papelera ANTES de la mutación (`trashBefore`) y (b) la operación inversa
// (`reverse`) con el snapshot COMPLETO del record afectado. Ctrl+Z hace pop del
// último entry, ejecuta el reverse vía bridge (vantaPut/remove) y restaura el
// snapshot de la papelera. Los tombstones viven en memoria de sesión — no se
// persisten (la persistencia de papelera es territorio de Fase 1).
import { remove, vantaPut, type MemoryRecord } from "../vanta";

export interface Tombstone {
  /** Snapshot completo del record al momento del borrado (VS-11 enriquece el
   * DTO: version/node_id/updated_at/expires/vector vienen en el record). */
  record: MemoryRecord;
  deletedAtMs: number;
}

/** Operación inversa que el caller del bridge debe ejecutar para deshacer. */
type Reverse =
  | { kind: "put"; record: MemoryRecord }
  | { kind: "put-batch"; records: MemoryRecord[] }
  | { kind: "remove"; record: MemoryRecord };

interface UndoEntry {
  /** Papelera exacta ANTES de la mutación — undo restaura este snapshot. */
  trashBefore: Tombstone[];
  reverse: Reverse;
}

/** Profundidad máxima del historial de undo (Ctrl+Z no es infinito). */
const MAX_HISTORY = 50;

class UndoStore {
  private trash: Tombstone[] = [];
  private history: UndoEntry[] = [];
  private listeners = new Set<() => void>();
  /** Serializa las ops de backend (rapid Ctrl+Z / doble click no intercalan). */
  private queue: Promise<unknown> = Promise.resolve();

  getTrash(): Tombstone[] {
    return this.trash;
  }

  canUndo(): boolean {
    return this.history.length > 0;
  }

  subscribe(fn: () => void): () => void {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  }

  private notify(): void {
    for (const fn of this.listeners) fn();
  }

  private run<T>(op: () => Promise<T>): Promise<T> {
    const next = this.queue.then(op, op);
    this.queue = next.then(
      () => undefined,
      () => undefined,
    );
    return next;
  }

  private pushEntry(entry: UndoEntry): void {
    this.history.push(entry);
    if (this.history.length > MAX_HISTORY) this.history.shift();
  }

  /** Soft-delete: borra en backend y guarda un tombstone local (snapshot).
   * El backend se toca PRIMERO: si falla, no queda ningún estado fantasma. */
  async softDelete(record: MemoryRecord): Promise<void> {
    return this.run(async () => {
      await remove(record.id, record.namespace);
      this.pushEntry({
        trashBefore: [...this.trash],
        reverse: { kind: "put", record },
      });
      this.trash = [{ record, deletedAtMs: Date.now() }, ...this.trash];
      this.notify();
    });
  }

  /** Soft-delete de un lote (ESPACIO-02/OP-02): borra cada record por key y
   * guarda UN entry de undo con el snapshot completo del lote — un solo Ctrl+Z
   * restaura todos (mismo mecanismo VS-08 que softDelete). El backend se toca
   * PRIMERO: si un remove falla a mitad, ningún tombstone queda registrado y
   * el error propaga; los ya borrados quedaron borrados (fallo atómico de
   * backend no es simulable client-side — Ctrl+Z no puede deshacer algo que
   * no registró snapshot). */
  async softDeleteBatch(records: MemoryRecord[]): Promise<void> {
    return this.run(async () => {
      for (const r of records) {
        await remove(r.id, r.namespace);
      }
      this.pushEntry({
        trashBefore: [...this.trash],
        reverse: { kind: "put-batch", records },
      });
      this.trash = [
        ...records.map((record) => ({ record, deletedAtMs: Date.now() })),
        ...this.trash,
      ];
      this.notify();
    });
  }

  /** Restore: vantaPut con el snapshot del tombstone; la entrada queda
   * deshacible (Ctrl+Z la vuelve a borrar). */
  async restore(tombstone: Tombstone): Promise<void> {
    return this.run(async () => {
      const r = tombstone.record;
      await vantaPut({
        namespace: r.namespace,
        key: r.id,
        payload: r.text,
        metadata: r.metadata ?? undefined,
        expires_at_ms: r.expires_at_ms ?? undefined,
      });
      this.pushEntry({
        trashBefore: [...this.trash],
        reverse: { kind: "remove", record: r },
      });
      this.trash = this.trash.filter((t) => t !== tombstone);
      this.notify();
    });
  }

  /** Eliminación definitiva: descarta el tombstone (el record ya no existe en
   * backend — no hay op de backend). Ctrl+Z todavía la deshace (restaura). */
  purge(tombstone: Tombstone): void {
    this.pushEntry({
      trashBefore: [...this.trash],
      reverse: { kind: "put", record: tombstone.record },
    });
    this.trash = this.trash.filter((t) => t !== tombstone);
    this.notify();
  }

  /** Ctrl+Z: pop del último entry, ejecuta el reverse y restaura el snapshot
   * de la papelera. Devuelve una etiqueta legible para el notice. En fallo de
   * backend re-pushea el entry (sigue deshacible) y relanza — la papelera no
   * se toca hasta que el reverse tuvo éxito.
   * El pop corre DENTRO de la cola: un Ctrl+Z inmediato tras un softDelete en
   * vuelo deshace ESE delete (su entry ya se pusheó cuando su op corrió). */
  async undo(): Promise<string> {
    return this.run(async () => {
      const entry = this.history.pop();
      if (!entry) return "nada que deshacer";
      const putRecord = async (r: MemoryRecord) => {
        await vantaPut({
          namespace: r.namespace,
          key: r.id,
          payload: r.text,
          metadata: r.metadata ?? undefined,
          expires_at_ms: r.expires_at_ms ?? undefined,
        });
      };
      try {
        if (entry.reverse.kind === "put-batch") {
          for (const r of entry.reverse.records) {
            await putRecord(r);
          }
        } else if (entry.reverse.kind === "put") {
          await putRecord(entry.reverse.record);
        } else {
          const r = entry.reverse.record;
          await remove(r.id, r.namespace);
        }
        this.trash = entry.trashBefore;
        this.notify();
        if (entry.reverse.kind === "put-batch") {
          return `deshecho · restaurados ${entry.reverse.records.length}`;
        }
        const r = entry.reverse.record;
        return entry.reverse.kind === "put"
          ? `deshecho · restaurado ${r.id}`
          : `deshecho · eliminado ${r.id}`;
      } catch (err) {
        this.history.push(entry);
        this.notify();
        throw err;
      }
    });
  }
}

export const undoStore = new UndoStore();