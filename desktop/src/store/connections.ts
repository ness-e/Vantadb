// Perfiles de conexión + defaults de búsqueda + idioma (DESKTOP-31).
// Mismo mecanismo probado que preferences/favorites (DESKTOP-23): localStorage
// del WebView de Tauri ya persiste entre sesiones — no hace falta app_config_dir
// ni comandos Tauri (el transporte con Bearer ya vive en Rust: ServerClientConfig).
// Storage inyectable con default `localStorage`; storage corrupto → defaults.
const STORAGE_KEY = "vanta.connections.v1";

export type ProfileKind = "native" | "server";

/** Perfil guardado: nativo (path) o server remoto (URL + puerto + Bearer). */
export interface ConnectionProfile {
  id: string;
  name: string;
  kind: ProfileKind;
  /** Solo kind="native": ruta del backend embebido. */
  path?: string;
  /** Solo kind="server": base URL sin puerto (ej. http://127.0.0.1). */
  url?: string;
  port?: number;
  /** Bearer token para `vanta-cli server` con auth. */
  token?: string;
}

export type SearchMode = "hybrid" | "vector";
export type Lang = "es" | "en";

export interface ConnectionsPrefs {
  profiles?: ConnectionProfile[];
  activeProfileId?: string | null;
  topK?: number;
  mode?: SearchMode;
  lang?: Lang;
}

function defaultStorage(): Storage | null {
  try {
    return typeof localStorage !== "undefined" ? localStorage : null;
  } catch {
    return null;
  }
}

function sanitizeProfile(raw: unknown): ConnectionProfile | null {
  if (!raw || typeof raw !== "object") return null;
  const o = raw as Record<string, unknown>;
  if (typeof o.id !== "string" || typeof o.name !== "string") return null;
  if (o.kind !== "native" && o.kind !== "server") return null;
  const p: ConnectionProfile = { id: o.id, name: o.name, kind: o.kind };
  if (typeof o.path === "string") p.path = o.path;
  if (typeof o.url === "string") p.url = o.url;
  if (typeof o.port === "number") p.port = o.port;
  if (typeof o.token === "string") p.token = o.token;
  return p;
}

function sanitize(raw: unknown): ConnectionsPrefs {
  if (!raw || typeof raw !== "object") return {};
  const o = raw as Record<string, unknown>;
  const prefs: ConnectionsPrefs = {};
  if (Array.isArray(o.profiles)) {
    prefs.profiles = o.profiles.map(sanitizeProfile).filter((p): p is ConnectionProfile => p !== null);
  }
  if (typeof o.activeProfileId === "string" || o.activeProfileId === null) {
    prefs.activeProfileId = o.activeProfileId as string | null;
  }
  if (typeof o.topK === "number" && Number.isFinite(o.topK) && o.topK > 0) prefs.topK = o.topK;
  if (o.mode === "hybrid" || o.mode === "vector") prefs.mode = o.mode;
  if (o.lang === "es" || o.lang === "en") prefs.lang = o.lang;
  return prefs;
}

export class ConnectionPrefsStore {
  private prefs: ConnectionsPrefs = {};

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
      this.prefs = {}; // storage corrupto → arrancar limpio
    }
  }

  get(): ConnectionsPrefs {
    return { ...this.prefs };
  }

  set(patch: ConnectionsPrefs): void {
    this.prefs = { ...this.prefs, ...patch };
    if (!this.storage) return;
    try {
      this.storage.setItem(STORAGE_KEY, JSON.stringify(this.prefs));
    } catch {
      // quota/privacidad → solo de sesión (no crashea la app)
    }
  }

  /** Upsert por id (id nuevo si falta). Devuelve el perfil guardado. */
  upsertProfile(p: Omit<ConnectionProfile, "id"> & { id?: string }): ConnectionProfile {
    const full: ConnectionProfile = { ...p, id: p.id ?? crypto.randomUUID() };
    const rest = (this.prefs.profiles ?? []).filter((x) => x.id !== full.id);
    this.set({ profiles: [...rest, full] });
    return full;
  }

  removeProfile(id: string): void {
    const rest = (this.prefs.profiles ?? []).filter((p) => p.id !== id);
    this.set({
      profiles: rest,
      activeProfileId: this.prefs.activeProfileId === id ? null : this.prefs.activeProfileId,
    });
  }
}

export const connectionPrefs = new ConnectionPrefsStore();

/** Id legible del objetivo de un perfil (dropdowns, listas). */
export function profileTarget(p: ConnectionProfile): string {
  return p.kind === "native"
    ? (p.path ?? "")
    : `${p.url ?? ""}${p.port ? `:${p.port}` : ""}`;
}
