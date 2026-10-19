// E2E-VISUAL — helpers compartidos de los specs (UX-19 + DAUD-01).
// Sin imports de src/** (cero acoplamiento): seed vía HTTP API del server
// embedded, igual que scripts/selfcheck-web-e2e.ts.
import { mkdirSync } from "node:fs";
import { join } from "node:path";

/** Base URL de la app (default: web build embedded servido por e2e/serve.mjs). */
export const APP_BASE =
  process.env.E2E_BASE_URL ??
  `http://127.0.0.1:${process.env.E2E_PORT ?? 8091}/dashboard/`;

/** Base de la API REST (origin de APP_BASE — same-origin en embedded). */
export const API_BASE = new URL("/", APP_BASE).toString().replace(/\/$/, "");

/** Screenshots evidencia DAUD-01 (light/dark) — regenerables, gitignore. */
export const SCREENSHOTS_DIR = join("e2e", "screenshots");

/** Seed idempotente por key (upsert) — mismo wire que selfcheck-web-e2e. */
export async function seedRecords(
  records: {
    namespace: string;
    key: string;
    payload: string;
    metadata?: Record<string, unknown>;
    vector?: number[] | null;
    sparse_vector?: Record<string, number> | null;
    ttl_ms?: number | null;
  }[],
): Promise<void> {
  const res = await fetch(`${API_BASE}/api/v2/records/batch`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(records),
  });
  if (!res.ok) {
    throw new Error(`seed failed: ${res.status} ${await res.text()}`);
  }
}

export function ensureScreenshotsDir(): void {
  mkdirSync(SCREENSHOTS_DIR, { recursive: true });
}

/** Tokens FIX-D1 (App.css/index.css) — mismos valores en light y dark. */
export const CREAM = "rgb(251, 249, 245)"; // --background light (#FBF9F5)
export const VANTA_BLACK = "rgb(10, 10, 10)"; // --background dark (#0a0a0a)