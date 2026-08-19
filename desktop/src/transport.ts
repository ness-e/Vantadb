// Pluggable IPC transport for the console (WEB-00).
//
// Every `vanta_*` command wrapper in vanta.ts goes through this transport
// instead of calling `invoke` directly, so the same React components can run
// against Tauri IPC today and against the embedded HTTP server (WEB-04) —
// or WASM (Fase 4) — without being rewritten.
import { invoke } from "@tauri-apps/api/core";
import { getHttpMapping } from "./vanta-http-map.ts";

export interface VantaTransport {
  call<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
}

/** Tauri v2 IPC backend. `invoke` accepts `InvokeArgs` which is a superset of
 * `Record<string, unknown>` (core.d.ts:105), so the narrowed `call` signature
 * is assignment-compatible for every command wrapper in vanta.ts. */
export class TauriBackend implements VantaTransport {
  call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    return invoke<T>(cmd, args);
  }
}

/** HTTP backend (WEB-04): fetch against the embedded server's `/api/v2/*`
 * endpoints. Command resolution + wire adaptation live in vanta-http-map.ts;
 * this class is a thin fetch engine that maps errors to the standard
 * `{success:false, error|data}` server shape (query errors use `data`). */
export class HttpBackend implements VantaTransport {
  // Plain field + explicit assignment: parameter properties are generated
  // code that Node's TS strip-only mode (node --test) cannot parse.
  private readonly base: string;
  constructor(base: string) {
    this.base = base;
  }

  async call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    const mapping = getHttpMapping(cmd);
    const query = mapping.query?.(args ?? {});
    const body = mapping.body?.(args ?? {});
    const qs = query
      ? "?" +
        new URLSearchParams(
          Object.fromEntries(
            Object.entries(query)
              .filter(([, v]) => v !== undefined && v !== null && v !== "")
              .map(([k, v]) => [k, String(v)]),
          ),
        ).toString()
      : "";
    const res = await fetch(this.base + mapping.path(args ?? {}) + qs, {
      method: mapping.method,
      headers: body !== undefined ? { "content-type": "application/json" } : undefined,
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });
    if (!res.ok) throw await httpError(res, cmd);
    const data: unknown = await res.json().catch(() => null);
    return (mapping.transform ? mapping.transform(data) : data) as T;
  }
}

/** Build a descriptive error from a non-2xx response body (`{success, error|data}`). */
async function httpError(res: Response, cmd: string): Promise<Error> {
  let detail = "";
  try {
    const j = (await res.json()) as { error?: unknown; data?: unknown };
    detail = typeof j.error === "string" ? j.error : typeof j.data === "string" ? j.data : "";
  } catch {
    // Non-JSON error body — fall through to the status line.
  }
  const msg = detail
    ? `${cmd}: ${detail}`
    : `${cmd}: HTTP ${res.status} ${res.statusText}`;
  return new Error(`[vanta] ${msg}`);
}

/** Pick the backend for the current environment. Tauri v2 exposes
 * `window.__TAURI_INTERNALS__`; in a plain browser it is absent, so the app
 * talks to the embedded server over REST (`VITE_VANTA_API_BASE` overrides the
 * origin, e.g. for a remote dev server). */
export function getTransport(): VantaTransport {
  const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
  if (inTauri) return new TauriBackend();
  const env = (import.meta as { env?: Record<string, string | undefined> }).env;
  return new HttpBackend(env?.VITE_VANTA_API_BASE ?? "");
}

/** Module-level transport — the environment is fixed at load time. */
export const transport: VantaTransport = getTransport();
