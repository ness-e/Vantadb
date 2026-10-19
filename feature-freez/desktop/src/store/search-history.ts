// Historial de búsquedas (VS-17): últimas N queries en localStorage,
// re-ejecutables desde la palette (grupo HISTORIAL). Dedup no-consecutivo:
// re-buscar una query la mueve al frente (patrón de historial estándar).
// Store vanilla con suscripción, sin deps nuevas.
const STORAGE_KEY = "vanta.search-history.v1";
const MAX_ENTRIES = 10;

function defaultStorage(): Storage | null {
  try {
    return typeof localStorage !== "undefined" ? localStorage : null;
  } catch {
    return null;
  }
}

export class SearchHistory {
  private entries: string[] = [];
  private listeners = new Set<() => void>();

  constructor(private storage: Storage | null = defaultStorage()) {
    this.load();
  }

  private load(): void {
    if (!this.storage) return;
    try {
      const raw = this.storage.getItem(STORAGE_KEY);
      if (!raw) return;
      const parsed = JSON.parse(raw) as unknown;
      if (Array.isArray(parsed)) {
        this.entries = parsed
          .filter((e): e is string => typeof e === "string")
          .slice(0, MAX_ENTRIES);
      }
    } catch {
      this.entries = [];
    }
  }

  private persist(): void {
    if (!this.storage) return;
    try {
      this.storage.setItem(STORAGE_KEY, JSON.stringify(this.entries));
    } catch {
      // sin persistencia → solo sesión
    }
  }

  private notify(): void {
    for (const fn of this.listeners) fn();
  }

  get(): string[] {
    return [...this.entries];
  }

  add(query: string): void {
    const q = query.trim();
    if (!q) return;
    this.entries = [q, ...this.entries.filter((e) => e !== q)].slice(0, MAX_ENTRIES);
    this.persist();
    this.notify();
  }

  clear(): void {
    if (this.entries.length === 0) return;
    this.entries = [];
    this.persist();
    this.notify();
  }

  subscribe(fn: () => void): () => void {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  }
}

export const searchHistory = new SearchHistory();