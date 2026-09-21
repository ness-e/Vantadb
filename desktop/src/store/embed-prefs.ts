// Preferencias del modelo de embedding local (DESKTOP-EMBED-01 follow-up).
//
// Selección del modelo que `vanta_embed_text` usa por defecto. Persiste en
// localStorage (mismo mecanismo que `connectionPrefs.ts`) — el WebView de
// Tauri lo retiene entre sesiones sin necesidad de un comando IPC nuevo.
//
// Si `embed-local` está OFF en la build, el frontend sigue mostrando el
// dropdown pero el backend devuelve source="dummy" para cualquier eleccion.
// Es honesto con el contrato del IPC (`EmbeddingResult.source`).

const STORAGE_KEY = "vanta.embed.v1";

/** Catálogo curado — match exacto con `embeddings/manifest.json`. */
export const EMBED_MODELS = [
  { id: "multilingual-e5-small", label: "multilingual-e5-small (ES+EN 16+, 384d, MIT) — DEFAULT" },
  { id: "all-MiniLM-L6-v2", label: "all-MiniLM-L6-v2 (EN, 384d, Apache-2.0, ultra-ligero)" },
  { id: "bge-small-en-v1.5", label: "bge-small-en-v1.5 (EN, 384d, MIT, baseline)" },
  { id: "bge-base-en-v1.5", label: "bge-base-en-v1.5 (EN, 768d, MIT, balance)" },
  { id: "paraphrase-multilingual-MiniLM-L12-v2", label: "paraphrase-multilingual-MiniLM-L12-v2 (ES+EN 50+, 384d, Apache-2.0)" },
  { id: "jina-es-v2-base", label: "jina-es-v2-base (ES optimizado, 768d, Apache-2.0)" },
  { id: "distiluse-multilingual", label: "distiluse-multilingual (ES+EN 15+, 512d, Apache-2.0)" },
] as const;

export type EmbedModelId = (typeof EMBED_MODELS)[number]["id"];

export const DEFAULT_EMBED_MODEL: EmbedModelId = "multilingual-e5-small";

export interface EmbedPrefs {
  model: EmbedModelId;
}

function defaultStorage(): Storage | null {
  try {
    return typeof localStorage !== "undefined" ? localStorage : null;
  } catch {
    return null;
  }
}

function sanitize(raw: unknown): EmbedPrefs {
  if (!raw || typeof raw !== "object") return { model: DEFAULT_EMBED_MODEL };
  const o = raw as Record<string, unknown>;
  const id = typeof o.model === "string" ? (o.model as EmbedModelId) : DEFAULT_EMBED_MODEL;
  // Solo aceptar ids del catálogo curado (defense in depth — la UI no debería
  // permitir ids externos, pero si el storage está manipulado caemos al default).
  const known = EMBED_MODELS.find((m) => m.id === id);
  return { model: known ? known.id : DEFAULT_EMBED_MODEL };
}

export class EmbedPrefsStore {
  private prefs: EmbedPrefs = { model: DEFAULT_EMBED_MODEL };

  constructor(private storage: Storage | null = defaultStorage()) {
    this.load();
  }

  private load(): void {
    if (!this.storage) return;
    try {
      const raw = this.storage.getItem(STORAGE_KEY);
      if (!raw) return;
      this.prefs = sanitize(JSON.parse(raw));
    } catch {
      this.prefs = { model: DEFAULT_EMBED_MODEL };
    }
  }

  get(): EmbedPrefs {
    return { ...this.prefs };
  }

  set(patch: Partial<EmbedPrefs>): void {
    this.prefs = { ...this.prefs, ...patch };
    if (!this.storage) return;
    try {
      this.storage.setItem(STORAGE_KEY, JSON.stringify(this.prefs));
    } catch {
      // quota/privacidad → solo de sesion (no crashea la app)
    }
  }
}

export const embedPrefs = new EmbedPrefsStore();