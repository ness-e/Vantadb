//! `ConnectionSelector` — pick a backend via ("native" embedded or "server"
//! over HTTP) and open a connection through the Tauri IPC bridge.
//!
//! All calls go through `invoke("vanta_connect", { target: {...} })` — the
//! Rust command routes to the right [`VantaConnection`] adapter and registers
//! it (active) in the `ConnectionManager`. No direct browser fetch, so the
//! Bearer auth and the loopback port never leave the Tauri WebView.

import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface ConnectionInfo {
  id: string;
  name: string;
  via: string;
  status: string;
  description?: string | null;
}

export type Via = "native" | "server";

export interface NativeConfig {
  path: string;
}

export interface ServerConfig {
  url: string;
  port: number;
  token: string;
}

interface Props {
  active: ConnectionInfo | null;
  onConnected: (info: ConnectionInfo) => void;
}

export default function ConnectionSelector({ active, onConnected }: Props) {
  const [via, setVia] = useState<Via>("server");
  const [server, setServer] = useState<ServerConfig>({
    url: "127.0.0.1",
    port: 8080,
    token: "",
  });
  const [path, setPath] = useState<string>("./vantadb-dev");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>("");

  // Default to loopback; only 127.0.0.1 is valid for the bundled server so the
  // HTTP port is never exposed beyond the local machine.
  const onUrlChange = (v: string) => setServer((s) => ({ ...s, url: v || "127.0.0.1" }));

  async function connect() {
    setBusy(true);
    setError("");
    try {
      const target =
        via === "server"
          ? {
              via: "server",
              config: {
                url: server.url,
                port: Number(server.port) || 8080,
                ...(server.token.trim() ? { token: server.token.trim() } : {}),
              },
            }
          : { via: "native", path };
      const info = await invoke<ConnectionInfo>("vanta_connect", { target });
      onConnected(info);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="card">
      <h2>Connection</h2>

      <div className="row" role="tablist" aria-label="Via">
        {(
          [
            ["server", "Server"],
            ["native", "Native"],
          ] as [Via, string][]
        ).map(([value, label]) => (
          <button
            key={value}
            type="button"
            className={via === value ? "seg active" : "seg"}
            aria-pressed={via === value}
            onClick={() => setVia(value)}
          >
            {label}
          </button>
        ))}
      </div>

      {via === "server" ? (
        <div className="fieldset">
          <label>
            URL
            <input value={server.url} onChange={(e) => onUrlChange(e.target.value)} placeholder="127.0.0.1" />
          </label>
          <label>
            Port
            <input
              type="number"
              value={server.port}
              onChange={(e) => setServer((s) => ({ ...s, port: Number(e.target.value) }))}
            />
          </label>
          <label>
            Token (optional)
            <input
              type="password"
              value={server.token}
              onChange={(e) => setServer((s) => ({ ...s, token: e.target.value }))}
              placeholder="VANTADB_API_KEY"
            />
          </label>
        </div>
      ) : (
        <div className="fieldset">
          <label>
            DB path
            <input value={path} onChange={(e) => setPath(e.target.value)} />
          </label>
        </div>
      )}

      <div className="row">
        <button type="button" disabled={busy} onClick={connect}>
          {busy ? "Connecting…" : "Connect"}
        </button>
        {active && <span className="badge ok">active: {active.name}</span>}
      </div>

      {error && <p className="err">{error}</p>}
    </section>
  );
}