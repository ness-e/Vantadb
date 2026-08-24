// Preferencias del workspace (DESKTOP-23): superficie activa (layout) y
// filtros compuestos persistidos para sobrevivir al reinicio de la app.
// Mismo mecanismo probado que favorites/search-history (VS-17): localStorage
// del WebView de Tauri ya persiste entre sesiones — no hace falta app_config_dir
// ni comandos Tauri (una segunda fuente de verdad sería más código, no más
// garantía). Storage inyectable con default `localStorage`; sin storage o con
// storage corrupto → defaults limpios, la app nunca crashea.
import type { RuleGroupType } from "react-querybuilder";

const STORAGE_KEY = "vanta.workspace.v1";

export interface WorkspacePrefs {
  surface?: string;
  showFilters?: boolean;
  ruleGroup?: RuleGroupType;
}

function defaultStorage(): Storage | null {
  try {
    return typeof localStorage !== "undefined" ? localStorage : null;
  } catch {
    return null;
  }
}

/** Sanitiza el objeto crudo del storage: solo campos conocidos y tipados.
 * ruleGroup exige `rules` array (forma mínima de RuleGroupType) para que un
 * JSON corrupto no rompa FiltersBuilder/evaluateQuery al hidratar. */
function sanitize(raw: unknown): WorkspacePrefs {
  if (!raw || typeof raw !== "object") return {};
  const o = raw as Record<string, unknown>;
  const prefs: WorkspacePrefs = {};
  if (typeof o.surface === "string") prefs.surface = o.surface;
  if (typeof o.showFilters === "boolean") prefs.showFilters = o.showFilters;
  if (
    !!o.ruleGroup &&
    typeof o.ruleGroup === "object" &&
    Array.isArray((o.ruleGroup as RuleGroupType).rules)
  ) {
    prefs.ruleGroup = o.ruleGroup as RuleGroupType;
  }
  return prefs;
}

export class WorkspacePrefsStore {
  private prefs: WorkspacePrefs = {};

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

  get(): WorkspacePrefs {
    return { ...this.prefs };
  }

  /** Merge parcial + persist write-through (el caller dueño del estado React
   * decide cuándo; no hay listeners porque el shell es el único consumidor). */
  set(patch: WorkspacePrefs): void {
    this.prefs = { ...this.prefs, ...patch };
    if (!this.storage) return;
    try {
      this.storage.setItem(STORAGE_KEY, JSON.stringify(this.prefs));
    } catch {
      // quota/privacidad → preferencias solo de sesión (no crashea la app)
    }
  }
}

export const workspacePrefs = new WorkspacePrefsStore();
