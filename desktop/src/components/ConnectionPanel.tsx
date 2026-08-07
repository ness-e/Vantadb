import { FormEvent, useState } from "react";
import { ConnectionInfo, HealthReport } from "../vanta";

interface Props {
  connections: ConnectionInfo[];
  activeId: string | null;
  health: HealthReport | null;
  healthStatus: "ok" | "warn" | "err" | "idle";
  busy: boolean;
  onConnectNative: (path: string) => Promise<string | null>;
  onDisconnect: (id: string) => Promise<void>;
  onActivate: (id: string) => Promise<void>;
  onProbeHealth: () => Promise<void>;
}

export default function ConnectionPanel({
  connections,
  activeId,
  health,
  healthStatus,
  busy,
  onConnectNative,
  onDisconnect,
  onActivate,
  onProbeHealth,
}: Props) {
  const [path, setPath] = useState("vantadb-local");

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    const id = await onConnectNative(path);
    if (id) setPath("vantadb-local-" + new Date().getTime().toString(36));
  }

  return (
    <section className="panel">
      <div className="panel-head">
        <h2>Connections</h2>
        <button
          type="button"
          className="health-badge"
          data-status={healthStatus}
          onClick={onProbeHealth}
          title="Re-run health probe"
        >
          {healthStatus === "idle"
            ? "—"
            : health
              ? `${health.backend} · ${health.latency_ms}ms`
              : "check"}
        </button>
      </div>

      <form className="row" onSubmit={handleSubmit}>
        <input
          value={path}
          onChange={(e) => setPath(e.target.value)}
          placeholder="Database path"
          aria-label="Database path"
        />
        <button type="submit" disabled={busy}>
          {busy ? "Connecting…" : "Connect native"}
        </button>
      </form>

      <ul className="conn-list">
        {connections.length === 0 && (
          <li className="muted">No connections yet. Connect a native backend.</li>
        )}
        {connections.map((c) => (
          <li key={c.id} className={c.id === activeId ? "active" : ""}>
            <button type="button" className="conn-name" onClick={() => onActivate(c.id)}>
              <span className={`dot ${c.status}`} />
              {c.name}
              {c.id === activeId && <em className="tag">active</em>}
            </button>
            <span className="muted">{c.via}{c.description ? ` · ${c.description}` : ""}</span>
            <button type="button" className="ghost" onClick={() => onDisconnect(c.id)}>
              disconnect
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}