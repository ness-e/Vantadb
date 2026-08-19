// Pluggable IPC transport for the console (WEB-00).
//
// Every `vanta_*` command wrapper in vanta.ts goes through this transport
// instead of calling `invoke` directly, so the same React components can run
// against Tauri IPC today and against the embedded HTTP server (WEB-04) —
// or WASM (Fase 4) — without being rewritten.
import { invoke } from "@tauri-apps/api/core";

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

/** HTTP backend stub — completed in WEB-04 with a fetch-based implementation
 * against the embedded server's `/api/v2/*` endpoints. */
export class HttpBackend implements VantaTransport {
  call<T>(_cmd: string, _args?: Record<string, unknown>): Promise<T> {
    return Promise.reject(
      new Error(`[vanta] HttpBackend not implemented yet (WEB-04): ${_cmd}`),
    );
  }
}

/** Pick the backend for the current environment. Tauri v2 exposes
 * `window.__TAURI_INTERNALS__`; in a plain browser it is absent. */
export function getTransport(): VantaTransport {
  const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
  return inTauri ? new TauriBackend() : new HttpBackend();
}

/** Module-level transport — the environment is fixed at load time. */
export const transport: VantaTransport = getTransport();
