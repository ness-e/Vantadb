import { FormEvent, useState } from "react";
import { get, ingest, IngestItem, vantaErrorMessage } from "../vanta";

interface Props {
  onDone: (ids: string[]) => void;
  runError: (msg: string) => void;
}

export default function IngestForm({ onDone, runError }: Props) {
  const [id, setId] = useState("");
  const [text, setText] = useState("");
  const [namespace, setNamespace] = useState("");
  const [busy, setBusy] = useState(false);

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    const item: IngestItem = { id: id || undefined, text, namespace: namespace || undefined };
    // VS-08 (Fix 4): sobrescribir es destructivo → confirmación explícita (P6).
    // `get` lanza NotFound si la key no existe → sin confirmación, crear nuevo.
    if (item.id) {
      try {
        const existing = await get(item.id, item.namespace);
        if (existing && !window.confirm(`"${item.id}" ya existe — ¿sobrescribir?`)) return;
      } catch {
        // key inexistente (NotFound) → crear sin confirmación
      }
    }
    setBusy(true);
    try {
      const ids = await ingest([item]);
      onDone(ids);
      setId("");
      setText("");
    } catch (err) {
      runError(vantaErrorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="border-[3px] border-foreground bg-card p-4 shadow-ink">
      <h2 className="m-0 font-tech text-xs uppercase tracking-widest">Ingestar</h2>
      <form className="mt-3 flex flex-col gap-2" onSubmit={handleSubmit}>
        <input
          value={id}
          onChange={(e) => setId(e.target.value)}
          placeholder="ID (opcional — el backend asigna uno)"
          aria-label="ID de registro"
          className="border-2 border-foreground bg-background px-2.5 py-1.5"
        />
        <textarea
          value={text}
          onChange={(e) => setText(e.target.value)}
          placeholder="Contenido de texto"
          rows={3}
          aria-label="Contenido de texto"
          required
          className="resize-y border-2 border-foreground bg-background px-2.5 py-1.5"
        />
        <input
          value={namespace}
          onChange={(e) => setNamespace(e.target.value)}
          placeholder="Namespace (por omisión 'default')"
          aria-label="Namespace"
          className="border-2 border-foreground bg-background px-2.5 py-1.5"
        />
        <button
          type="submit"
          disabled={busy || !text.trim()}
          className="press cursor-pointer self-start border-2 border-foreground bg-background px-2.5 py-1.5 text-sm disabled:cursor-default disabled:opacity-50"
        >
          {busy ? "Guardando…" : "Agregar registro"}
        </button>
        {/* DESKTOP-39 (Caso B): el core no genera embeddings localmente — ver
            src/llm.rs (EmbeddingProvider → Ollama/OpenAI, feature remote-inference).
            Documentar el límite en vez de fingir un botón "generar vector". */}
        <p className="m-0 text-xs opacity-60" title="Los vectores requieren un proveedor externo configurado por variables de entorno">
          Sin vector: el registro se guarda como texto. Para búsqueda semántica,
          generá el embedding con un proveedor externo (Ollama u OpenAI —
          VANTA_EMBEDDING_PROVIDER, VANTA_LLM_URL o VANTA_OPENAI_API_KEY).
        </p>
      </form>
    </section>
  );
}