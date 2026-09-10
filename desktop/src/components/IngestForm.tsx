import { FormEvent, useEffect, useRef, useState } from "react";
import {
  embedCapabilities,
  embedText,
  EmbedCapabilities,
  EmbeddingResult,
  get,
  ingest,
  IngestItem,
  vantaErrorMessage,
} from "../vanta";
import { embedPrefs } from "../store/embed-prefs";
import { tp, tt, type DesktopLang } from "../i18n";
import { connectionPrefs } from "../store/connections";

interface Props {
  onDone: (ids: string[]) => void;
  runError: (msg: string) => void;
  /** UX-17: remonta el grid tras un ingest manual (patrón `key={gridKey}`
   * de WorkspaceShell, igual que el batch delete y los imports). */
  onRefresh?: () => void;
  lang?: DesktopLang;
}

const LABEL =
  "font-tech text-[10px] uppercase tracking-widest text-muted-foreground";
const INPUT =
  "border-2 border-foreground bg-background px-2.5 py-1.5";

export default function IngestForm({ onDone, runError, onRefresh, lang = connectionPrefs.get().lang ?? "es" }: Props) {
  const [id, setId] = useState("");
  const [text, setText] = useState("");
  const [namespace, setNamespace] = useState("");
  const [busy, setBusy] = useState(false);
  // UX-04: error anclado inline (no solo al toast global del shell).
  const [error, setError] = useState<string | null>(null);
  // UX-04: confirmación inline de sobrescritura — patrón DeleteButton de
  // DataExplorer, no window.confirm nativo (05 anti-patrón).
  const [confirming, setConfirming] = useState(false);
  const pendingRef = useRef<IngestItem | null>(null);

  // DESKTOP-EMBED-01: local ONNX embeddings surfaced to the UI.
  const [caps, setCaps] = useState<EmbedCapabilities | null>(null);
  const [embedding, setEmbedding] = useState<EmbeddingResult | null>(null);
  const [embeddingBusy, setEmbeddingBusy] = useState(false);

  useEffect(() => {
    let cancelled = false;
    // Probe capabilities on mount. Falls back silently if the backend is not
    // Tauri (browser / WASM build).
    embedCapabilities()
      .then((c) => {
        if (!cancelled) setCaps(c);
      })
      .catch(() => {
        if (!cancelled) setCaps(null);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  async function doIngest(item: IngestItem) {
    setBusy(true);
    try {
      const ids = await ingest([item]);
      onDone(ids);
      onRefresh?.();
      setId("");
      setText("");
      setEmbedding(null);
      setError(null);
    } catch (err) {
      setError(vantaErrorMessage(err));
      runError(vantaErrorMessage(err));
    } finally {
      setBusy(false);
      setConfirming(false);
    }
  }

  async function handleGenerateEmbedding() {
    if (!text.trim()) {
      setError(tt(lang, "ingest.textFirstErr", "Escribí texto antes de generar el embedding"));
      return;
    }
    setEmbeddingBusy(true);
    setError(null);
    try {
      // DESKTOP-EMBED-01 follow-up: el modelo se lee de embedPrefs (persiste en
      // localStorage; configurable en Settings.tsx). Default = manifest default.
      const { model } = embedPrefs.get();
      const result = await embedText(text, model);
      setEmbedding(result);
    } catch (err) {
      setError(vantaErrorMessage(err));
    } finally {
      setEmbeddingBusy(false);
    }
  }

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);
    const item: IngestItem = {
      id: id || undefined,
      text,
      namespace: namespace || undefined,
      embedding: embedding?.vector,
    };
    // Sobrescribir es destructivo → confirmación explícita (P6). `get` lanza
    // NotFound si la key no existe → sin confirmación, crear nuevo.
    if (item.id) {
      try {
        const existing = await get(item.id, item.namespace);
        if (existing) {
          pendingRef.current = item;
          setConfirming(true);
          return;
        }
      } catch {
        // key inexistente (NotFound) → crear sin confirmación
      }
    }
    await doIngest(item);
  }

  return (
    // UX-11: id para que el empty state del grid navegue aquí (scrollIntoView).
    <section id="ingest-form" className="border-[3px] border-foreground bg-card p-4 shadow-ink">
      <h2 className="m-0 font-tech text-xs uppercase tracking-widest">{tt(lang, "ingest.title", "Ingestar")}</h2>
      <form className="mt-3 flex flex-col gap-2" onSubmit={handleSubmit}>
        {/* UX-04: labels VISIBLES (WCAG 3.3.2) — placeholder solo no alcanza. */}
        <label className="flex flex-col gap-1">
          <span className={LABEL}>{tt(lang, "ingest.idLabel", "ID (opcional)")}</span>
          <input
            value={id}
            onChange={(e) => setId(e.target.value)}
            placeholder={tt(lang, "ingest.idPh", "el backend asigna uno")}
            className={INPUT}
          />
        </label>
        <label className="flex flex-col gap-1">
          <span className={LABEL}>{tt(lang, "ingest.textLabel", "Contenido de texto")}</span>
          <textarea
            value={text}
            onChange={(e) => {
              setText(e.target.value);
              // El texto cambió → invalidar cualquier embedding previo
              if (embedding) setEmbedding(null);
            }}
            placeholder={tt(lang, "ingest.textPh", "Texto a recordar")}
            rows={3}
            required
            className="resize-y border-2 border-foreground bg-background px-2.5 py-1.5"
          />
        </label>
        <label className="flex flex-col gap-1">
          <span className={LABEL}>{tt(lang, "ingest.nsLabel", "Namespace")}</span>
          <input
            value={namespace}
            onChange={(e) => setNamespace(e.target.value)}
            placeholder={tt(lang, "ingest.nsPh", "por omisión 'default'")}
            className={INPUT}
          />
        </label>
        {error && (
          <p role="alert" className="border-2 border-foreground bg-card px-2 py-1.5 font-tech text-[10px] text-destructive">
            {error}
          </p>
        )}
        {confirming && (
          <div role="alert" className="flex flex-wrap items-center gap-2 border-2 border-foreground bg-muted px-2 py-1.5">
            <span className="text-xs">{tp(lang, "ingest.dupMsg", "“{id}” ya existe — ¿sobrescribir?", { id: pendingRef.current?.id ?? "" })}</span>
            <button
              type="button"
              onClick={() => {
                const item = pendingRef.current;
                setConfirming(false);
                if (item) void doIngest(item);
              }}
              className="press border-2 border-foreground bg-neon px-2 py-1 text-[10px] font-bold text-background"
            >
              {tt(lang, "ingest.overwriteBtn", "SOBRESCRIBIR")}
            </button>
            <button
              type="button"
              onClick={() => setConfirming(false)}
              className="press border-2 border-foreground bg-background px-2 py-1 text-[10px]"
              aria-label={tt(lang, "ingest.cancelOverwriteAria", "Cancelar sobrescritura")}
            >
              {tt(lang, "ingest.cancelOverwriteBtn", "✕ CANCELAR")}
            </button>
          </div>
        )}
        <div className="flex flex-wrap items-center gap-2">
          <button
            type="submit"
            disabled={busy || confirming || !text.trim()}
            className="press cursor-pointer border-2 border-foreground bg-background px-2.5 py-1.5 text-sm disabled:cursor-default disabled:opacity-50"
          >
            {busy ? tt(lang, "ingest.savingBtn", "Guardando…") : tt(lang, "ingest.addBtn", "Agregar registro")}
          </button>
          {/* DESKTOP-EMBED-01: generación de vector local vía IPC.
              El botón sólo aparece cuando el shell Tauri expone el comando
              (caps.embed_local_compiled = true) — el fallback "Sin vector"
              sigue activo si la build se hizo sin `--features embed-local`. */}
          {caps?.embed_local_compiled && (
            <button
              type="button"
              onClick={() => void handleGenerateEmbedding()}
              disabled={embeddingBusy || !text.trim() || busy}
              className="press cursor-pointer border-2 border-foreground bg-background px-2.5 py-1.5 text-xs disabled:cursor-default disabled:opacity-50"
              aria-label={tt(lang, "ingest.genVectorAria", "Generar embedding local con ONNX")}
              title={tp(lang, "ingest.genVectorTitle", "Genera un vector de {d} dimensiones vía ort+tokenizers", { d: String(caps.default_model ?? "384") })}
            >
              {embeddingBusy ? tt(lang, "ingest.embedding", "Embebiendo…") : embedding ? tt(lang, "ingest.regen", "↻ Regenerar vector") : tt(lang, "ingest.genLocal", "Generar vector local")}
            </button>
          )}
        </div>
        {embedding && (
          <p
            className="m-0 border-2 border-foreground bg-card px-2 py-1.5 font-tech text-[10px]"
            data-testid="embedding-summary"
            data-source={embedding.source}
            data-dim={embedding.dim}
            data-model={embedding.model}
          >
            {tp(lang, "ingest.embedSummary", "{ok} vector {dim}d ({src}) — modelo {model}", { ok: embedding.source === "real" ? "✓" : "⚠", dim: String(embedding.dim), src: embedding.source === "real" ? "ONNX" : "dummy", model: embedding.model })}
          </p>
        )}
        {!caps?.embed_local_compiled && (
          <p className="m-0 text-xs opacity-60" title={tt(lang, "ingest.noEmbedTitle", "El binario del desktop se compiló sin --features embed-local")}>
            {tt(lang, "ingest.noEmbedLead", "Sin vector: el registro se guarda como texto. Para búsqueda semántica local,")}
            recompilá el desktop con <code>cargo tauri dev --features embed-local</code>
            {tt(lang, "ingest.noEmbedTail", "(o instalá un proveedor externo: Ollama u OpenAI).")}
          </p>
        )}
      </form>
    </section>
  );
}