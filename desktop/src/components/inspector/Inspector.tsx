// Inspector (VS-06): orquestador master-detail del registro. Tabs General /
// Metadata / Vector / Payload. Nunca auto-guarda (P6): los edits viven en
// drafts locales; el footer muestra el diff de cambios y Guardar (vantaPut) /
// Revertir (descarta) como commit explícito. Al guardar, `onSaved` devuelve el
// MemoryRecord fresco (version bump) que refresca la selección del shell.
import { useEffect, useMemo, useState } from "react";
import type { MemoryRecord } from "../../vanta";
import { vantaErrorMessage, vantaPut } from "../../vanta";
import GeneralTab from "./GeneralTab";
import MetadataTab from "./MetadataTab";
import PayloadTab from "./PayloadTab";
import VectorTab from "./VectorTab";
// VS-14: tab HISTORIAL — versiones retenidas + diff + revert explícito (P6).
import HistorialTab from "./historial-tab";
import {
  fmtDuration,
  metadataToRows,
  rowsToMetadata,
  ttlFromRecord,
  ttlToMs,
  type MetaRow,
  type TtlDraft,
} from "./shared";
// VS-17: favorito (★) + copy-as (JSON / KEY / MD) — slice aditivo.
import { favoritesStore } from "../../store/favorites";
import { CopyButton } from "../copy/CopyButton";
import { recordToJson, recordToMarkdown } from "../copy/copy-as";

export type InspectorTab = "general" | "metadata" | "vector" | "payload" | "historial";

interface Props {
  record: MemoryRecord;
  /** Score de búsqueda; null en navegación por grid. */
  score: number | null;
  dark: boolean;
  onClose: () => void;
  /** Record actualizado tras `vantaPut` (nuevo version). */
  onSaved: (updated: MemoryRecord) => void;
  onError: (msg: string) => void;
}

const TABS: { id: InspectorTab; label: string }[] = [
  { id: "general", label: "GENERAL" },
  { id: "metadata", label: "METADATA" },
  { id: "vector", label: "VECTOR" },
  { id: "payload", label: "PAYLOAD" },
  { id: "historial", label: "HISTORIAL" },
];

export default function Inspector({ record, score, dark, onClose, onSaved, onError }: Props) {
  const [tab, setTab] = useState<InspectorTab>("general");
  const [payloadText, setPayloadText] = useState(record.text);
  const [metaRows, setMetaRows] = useState<MetaRow[]>(() => metadataToRows(record.metadata ?? {}));
  const [ttl, setTtl] = useState<TtlDraft>(() => ttlFromRecord(record));
  const [saving, setSaving] = useState(false);
  const [savedFlash, setSavedFlash] = useState(false);

  // VS-17: re-render al cambiar favoritos (★ en header) — store notifica.
  const [, setFavTick] = useState(0);
  useEffect(() => favoritesStore.subscribe(() => setFavTick((t) => t + 1)), []);

  // Un record distinto (nuevo objeto tras save) reinicia los drafts desde el
  // record guardado. Un registro DIFERENTE remonta vía key del shell.
  useEffect(() => {
    setPayloadText(record.text);
    setMetaRows(metadataToRows(record.metadata ?? {}));
    setTtl(ttlFromRecord(record));
  }, [record]);

  const metaCheck = useMemo(() => rowsToMetadata(metaRows), [metaRows]);
  const payloadDirty = payloadText !== record.text;
  const metaDirty =
    metaCheck.error !== null ||
    JSON.stringify(metaCheck.meta) !== JSON.stringify(record.metadata ?? {});
  const ttlDirty = ttlToMs(ttl, Date.now()) !== (record.expires_at_ms ?? null);
  const dirty = payloadDirty || metaDirty || ttlDirty;

  const diffItems = useMemo(() => {
    const items: string[] = [];
    if (payloadDirty) items.push("payload (texto editado)");
    if (metaDirty) {
      const before = new Set(Object.keys(record.metadata ?? {}));
      const after = new Set(metaRows.map((r) => r.key.trim()).filter(Boolean));
      const added = [...after].filter((k) => !before.has(k));
      const removed = [...before].filter((k) => !after.has(k));
      const kept = [...after].filter((k) => before.has(k));
      const parts: string[] = [];
      if (added.length) parts.push(`+${added.length} key`);
      if (removed.length) parts.push(`-${removed.length} key`);
      if (kept.length) parts.push(`~${kept.length} valor${kept.length === 1 ? "" : "es"}`);
      items.push(`metadata (${parts.join(" ") || "sin cambios netos"})`);
    }
    if (ttlDirty) {
      const before = record.expires_at_ms
        ? `${fmtDuration(record.expires_at_ms - Date.now())} restante`
        : "sin expiración";
      items.push(`TTL (${before} → nueva)`);
    }
    return items;
  }, [payloadDirty, metaDirty, ttlDirty, record, metaRows]);

  async function handleSave() {
    if (metaCheck.error) {
      onError(metaCheck.error);
      return;
    }
    setSaving(true);
    try {
      const updated = await vantaPut({
        namespace: record.namespace,
        key: record.id,
        payload: payloadText,
        metadata: metaCheck.meta,
        expires_at_ms: ttlToMs(ttl, Date.now()) ?? undefined,
      });
      onSaved(updated);
      setSavedFlash(true);
      window.setTimeout(() => setSavedFlash(false), 2500);
    } catch (err) {
      onError(vantaErrorMessage(err));
    } finally {
      setSaving(false);
    }
  }

  function handleRevert() {
    setPayloadText(record.text);
    setMetaRows(metadataToRows(record.metadata ?? {}));
    setTtl(ttlFromRecord(record));
  }

  // VS-17: estado del favorito de este registro (★ header).
  const isFav = favoritesStore.isFavorite(record.namespace, record.id);

  return (
    <aside
      className="flex w-[400px] shrink-0 flex-col overflow-hidden border-l-4 border-foreground bg-card"
      aria-label="Inspector de registro"
    >
      {/* Header: key/ns mono */}
      <div className="flex items-center justify-between gap-2 border-b-4 border-foreground px-4 py-3">
        <div className="min-w-0">
          <div className="font-tech text-[10px] uppercase tracking-widest text-neon">Inspector</div>
          <div className="mt-0.5 flex items-center gap-2">
            <code className="max-w-[170px] truncate font-tech text-sm">{record.id}</code>
            <span className="shrink-0 border-2 border-foreground bg-background px-1.5 py-0.5 font-tech text-[10px]">
              {record.namespace}
            </span>
            {score != null && (
              <span className="shrink-0 font-tech text-[10px] text-neon">{score.toFixed(3)}</span>
            )}
          </div>
        </div>
        <button
          type="button"
          onClick={onClose}
          className="press flex h-6 w-6 shrink-0 items-center justify-center border-2 border-foreground text-xs"
          aria-label="Cerrar inspector"
        >
          ✕
        </button>
      </div>

      {/* VS-17: favorito + copy-as (registro JSON / key / payload markdown) —
          fila compacta aditiva; el feedback "copiado" vive en CopyButton. */}
      <div className="flex items-center gap-1 border-b-4 border-foreground bg-background px-3 py-1.5">
        <button
          type="button"
          onClick={() => favoritesStore.toggle(record.namespace, record.id)}
          aria-pressed={isFav}
          className={`press flex h-6 w-6 items-center justify-center border-2 border-foreground text-sm ${
            isFav ? "bg-neon text-background" : "bg-background"
          }`}
          title={isFav ? `Quitar ${record.id} de favoritos` : `Agregar ${record.id} a favoritos`}
          aria-label={isFav ? `Quitar ${record.id} de favoritos` : `Agregar ${record.id} a favoritos`}
        >
          ★
        </button>
        <CopyButton
          getText={() => recordToJson(record)}
          label="JSON"
          title="Copiar registro completo (JSON)"
          onError={onError}
          className="h-6 px-2"
        />
        <CopyButton
          getText={() => record.id}
          label="KEY"
          title="Copiar key"
          onError={onError}
          className="h-6 px-2"
        />
        <CopyButton
          getText={() => recordToMarkdown(record)}
          label="MD"
          title="Copiar payload (markdown)"
          onError={onError}
          className="h-6 px-2"
        />
        <span className="ml-auto font-tech text-[9px] uppercase tracking-widest text-muted-foreground">
          copiar
        </span>
      </div>

      {/* Tabs */}
      <div className="flex border-b-4 border-foreground bg-background">
        {TABS.map((t) => (
          <button
            key={t.id}
            type="button"
            onClick={() => setTab(t.id)}
            aria-selected={tab === t.id}
            className={`flex-1 border-r-2 border-foreground px-1 py-2 font-tech text-[10px] uppercase tracking-widest last:border-r-0 ${
              tab === t.id ? "bg-neon text-background" : "text-muted-foreground hover:text-foreground"
            }`}
          >
            {tab === t.id ? `◆ ${t.label}` : t.label}
          </button>
        ))}
      </div>

      {/* Body */}
      <div className="flex-1 overflow-y-auto scroll-manga p-4">
        {tab === "general" && <GeneralTab record={record} score={score} ttl={ttl} setTtl={setTtl} />}
        {tab === "metadata" && <MetadataTab rows={metaRows} setRows={setMetaRows} />}
        {tab === "vector" && <VectorTab record={record} />}
        {tab === "payload" && <PayloadTab text={payloadText} onChange={setPayloadText} dark={dark} />}
        {tab === "historial" && <HistorialTab record={record} onSaved={onSaved} onError={onError} />}
      </div>

      {/* Footer: commit explícito (P6) */}
      <div className="border-t-4 border-foreground p-3">
        {dirty ? (
          <>
            <div className="flex items-center justify-between">
              <span className="font-tech text-[10px] uppercase tracking-widest text-neon">
                Cambios sin guardar
              </span>
              {savedFlash && (
                <span className="font-tech text-[10px] text-neon">✓ guardado v{record.version}</span>
              )}
            </div>
            <details className="mt-2 border-2 border-dashed border-foreground bg-background p-2">
              <summary className="cursor-pointer font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
                ver diff ({diffItems.length})
              </summary>
              <ul className="mt-1 list-inside list-disc space-y-0.5 font-tech text-[10px]">
                {diffItems.map((d) => (
                  <li key={d}>{d}</li>
                ))}
              </ul>
            </details>
            <div className="mt-3 flex gap-2">
              <button
                type="button"
                onClick={handleRevert}
                disabled={saving}
                className="press flex-1 border-2 border-foreground bg-background px-3 py-2 text-xs font-semibold"
              >
                REVERTIR
              </button>
              <button
                type="button"
                onClick={handleSave}
                disabled={saving || metaCheck.error !== null}
                className="btn-neon-glow flex-1 border-2 border-foreground bg-neon px-3 py-2 text-xs font-bold text-background"
              >
                {saving ? "GUARDANDO…" : "GUARDAR"}
              </button>
            </div>
            {metaCheck.error && (
              <p className="mt-1 font-tech text-[10px] text-neon">{metaCheck.error}</p>
            )}
          </>
        ) : savedFlash ? (
          <div className="text-center font-tech text-[10px] uppercase tracking-widest text-neon">
            ✓ guardado v{record.version}
          </div>
        ) : (
          <p className="text-center font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
            commit explícito — editar y guardar
          </p>
        )}
      </div>
    </aside>
  );
}