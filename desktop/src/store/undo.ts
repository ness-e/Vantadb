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
// snapshot de la papelera.
//
// DESKTOP-30: la papelera (tombstones) persiste en storage inyectable con
// default `localStorage` — mismo patrón DESKTOP-23/26 (el WebView de Tauri lo
// conserva entre sesiones; no hace falta app_config_dir). El stack de undo NO
// se persiste: sus reverses referencian estado del backend de la sesión.
import { ingestBatch, listAll, remove, vantaPut, type MemoryRecord } from "../vanta";

const TRASH_KEY = "vanta.trash.v1";

function defaultStorage(): Storage | null {
  try {
    return typeof localStorage !== "undefined" ? localStorage : null;
  } catch {
    return null;
  }
}

export interface Tombstone {
  /** Snapshot completo del record al momento del borrado (VS-11 enriquece el
   * DTO: version/node_id/updated_at/expires/vector vienen en el record). */
  record: MemoryRecord;
  deletedAtMs: number;
}

/** Operación inversa que el caller del bridge debe ejecutar para deshacer.
 * `move` (DESKTOP-32): rename de namespace — `records` llevan el namespace
 * ORIGINAL y `toNs` es el destino; el undo re-copia al origen y borra las
 * copias del destino. */
type Reverse =
  | { kind: "put"; record: MemoryRecord }
  | { kind: "put-batch"; records: MemoryRecord[] }
  | { kind: "remove"; record: MemoryRecord }
  | { kind: "move"; records: MemoryRecord[]; toNs: string };

interface UndoEntry {
  /** Papelera exacta ANTES de la mutación — undo restaura este snapshot. */
  trashBefore: Tombstone[];
  reverse: Reverse;
}

/** Profundidad máxima del historial de undo (Ctrl+Z no es infinito). */
const MAX_HISTORY = 50;

export class UndoStore {
  private trash: Tombstone[] = [];
  private history: UndoEntry[] = [];
  private listeners = new Set<() => void>();
  /** Serializa las ops de backend (rapid Ctrl+Z / doble click no intercalan). */
  private queue: Promise<unknown> = Promise.resolve();

  constructor(private storage: Storage | null = defaultStorage()) {
    this.loadTrash();
  }

  private loadTrash(): void {
    if (!this.storage) return;
    try {
      const raw = this.storage.getItem(TRASH_KEY);
      if (!raw) return;
      const parsed = JSON.parse(raw) as unknown;
      if (!Array.isArray(parsed)) return;
      this.trash = parsed.filter((t): t is Tombstone => {
        if (!t || typeof t !== "object") return false;
        const c = t as Partial<Tombstone>;
        return (
          typeof c.deletedAtMs === "number" &&
          !!c.record &&
          typeof c.record === "object" &&
          typeof c.record.id === "string" &&
          typeof c.record.namespace === "string"
        );
      });
    } catch {
      this.trash = []; // storage corrupto → arrancar limpio
    }
  }

  private persistTrash(): void {
    if (!this.storage) return;
    try {
      this.storage.setItem(TRASH_KEY, JSON.stringify(this.trash));
    } catch {
      // quota/privacidad → papelera solo de sesión (no crashea la app)
    }
  }

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

  /** Toda mutación de la papelera pasa por acá → persiste en el mismo hook. */
  private notify(): void {
    this.persistTrash();
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
    return this.run(() => this.applySoftDelete(records));
  }

  /** Cuerpo compartido de softDeleteBatch/deleteNamespace — llamar DENTRO de
   * run() (anidar run() acá sería deadlock de la cola). */
  private async applySoftDelete(records: MemoryRecord[]): Promise<void> {
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
  }

  /** DESKTOP-32: borrar un namespace entero → cada registro va a la papelera y
   * UN Ctrl+Z restaura todo el lote (mismo mecanismo que softDeleteBatch).
   * Devuelve la cantidad de registros movidos (0 = namespace vacío, no-op). */
  async deleteNamespace(ns: string): Promise<number> {
    return this.run(async () => {
      const records = await listAll(ns);
      if (records.length === 0) return 0;
      await this.applySoftDelete(records);
      return records.length;
    });
  }

  /** DESKTOP-32: renombrar namespace — el core no tiene rename atómico, así que
   * es copiar todo al nuevo ns (ingestBatch preserva embedding/metadata/ttl) y
   * borrar el viejo. Un Ctrl+Z revierte: re-copia al ns origen y borra las
   * copias del destino. Devuelve la cantidad de registros movidos.
   * Si un put/remove falla a mitad, no se registra entry (mismo contrato que
   * softDeleteBatch: fallo atómico de backend no es simulable client-side). */
  async renameNamespace(from: string, to: string): Promise<number> {
    return this.run(async () => {
      const records = await listAll(from);
      if (records.length === 0) return 0;
      await ingestBatch(
        records.map((r) => ({
          id: r.id,
          namespace: to,
          text: r.text,
          embedding: r.vector ?? undefined,
          metadata: r.metadata,
        })),
      );
      for (const r of records) {
        await remove(r.id, from);
      }
      this.pushEntry({
        trashBefore: [...this.trash],
        reverse: { kind: "move", records, toNs: to },
      });
      this.notify();
      return records.length;
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
        } else if (entry.reverse.kind === "move") {
          // Rename inverso: restaurar en el ns ORIGINAL y borrar las copias
          // del destino (records[0].namespace = origen).
          const { records, toNs } = entry.reverse;
          for (const r of records) {
            await putRecord(r);
          }
          for (const r of records) {
            await remove(r.id, toNs);
          }
        } else {
          const r = entry.reverse.record;
          await remove(r.id, r.namespace);
        }
        this.trash = entry.trashBefore;
        this.notify();
        if (entry.reverse.kind === "put-batch") {
          return `deshecho · restaurados ${entry.reverse.records.length}`;
        }
        if (entry.reverse.kind === "move") {
          return `deshecho · "${entry.reverse.toNs}" vuelve a llamarse "${entry.reverse.records[0]?.namespace}"`;
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