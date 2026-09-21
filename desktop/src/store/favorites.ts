// Favoritos de namespaces/keys (VS-17): toggle (★) persistido en localStorage.
// Store vanilla con suscripción (mismo patrón que store/undo.ts) — sin deps
// nuevas. `Favorite.key === null` = favorito de namespace completo (★ en la
// sidebar); `key` presente = favorito de registro (★ en grid/inspector).
//
// El storage es inyectable con default `localStorage` (guardado con
// `typeof` para no romper en Node/self-check); si no hay storage disponible
// (sandbox, acceso denegado), el store funciona in-memory sin persistir.
export interface Favorite {
  namespace: string;
  key: string | null;
}

const STORAGE_KEY = "vanta.favorites.v1";

function defaultStorage(): Storage | null {
  try {
    return typeof localStorage !== "undefined" ? localStorage : null;
  } catch {
    return null;
  }
}

export class FavoritesStore {
  private favorites: Favorite[] = [];
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
        this.favorites = parsed.filter(
          (f): f is Favorite =>
            !!f &&
            typeof f === "object" &&
            typeof (f as Favorite).namespace === "string" &&
            ((f as Favorite).key === null || typeof (f as Favorite).key === "string"),
        );
      }
    } catch {
      this.favorites = []; // storage corrupto → arrancar limpio
    }
  }

  private persist(): void {
    if (!this.storage) return;
    try {
      this.storage.setItem(STORAGE_KEY, JSON.stringify(this.favorites));
    } catch {
      // quota/privacidad → favoritos solo de sesión (no crashea la app)
    }
  }

  private notify(): void {
    for (const fn of this.listeners) fn();
  }

  getFavorites(): Favorite[] {
    return [...this.favorites];
  }

  isFavorite(namespace: string, key: string | null): boolean {
    return this.favorites.some((f) => f.namespace === namespace && f.key === key);
  }

  /** Toggle: agrega al frente (newest-first) o quita. Devuelve true si quedó
   * favorito. key=null togglea SOLO el favorito de namespace; los favoritos de
   * key del mismo namespace son entradas independientes. */
  toggle(namespace: string, key: string | null): boolean {
    const i = this.favorites.findIndex((f) => f.namespace === namespace && f.key === key);
    if (i !== -1) {
      this.favorites.splice(i, 1);
    } else {
      this.favorites.unshift({ namespace, key });
    }
    this.persist();
    this.notify();
    return i === -1;
  }

  subscribe(fn: () => void): () => void {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  }
}

export const favoritesStore = new FavoritesStore();