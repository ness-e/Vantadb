// Processes & Connections panel (ADMIN-08).
//
// Renders live connections from the shared bridge with a per-entry
// shutdown action (manager.remove via `vanta_disconnect` — releases the
// backend, and for subprocess-backed adapters force-kills the sidecar via
// McpSpawn's Drop).
//
// Subprocesses: the desktop tracks no registry of running children yet —
// `McpSpawn` (src-tauri/src/connections/child_process.rs) is defined but
// never instantiated, so there is nothing to list or kill. The empty state
// below is the documented future extension point.
import { ConnectionInfo } from "../vanta";

interface Props {
  connections: ConnectionInfo[];
  activeId: string | null;
  onShutdown: (id: string) => Promise<void>;
  onActivate: (id: string) => Promise<void>;
}

export default function ProcessPanel({
  connections,
  activeId,
  onShutdown,
  onActivate,
}: Props) {
  return (
    <section className="panel" aria-label="Processes and connections">
      <div className="panel-head">
        <h2>Processes &amp; Connections</h2>
      </div>

      <h3 className="proc-sub">Connections</h3>
      <ul className="conn-list">
        {connections.length === 0 && (
          <li className="muted">No connections. Connect a backend to see it here.</li>
        )}
        {connections.map((c) => (
          <li key={c.id} className={c.id === activeId ? "active" : ""}>
            <button type="button" className="conn-name" onClick={() => onActivate(c.id)}>
              <span className={`dot ${c.status}`} />
              {c.name}
              {c.id === activeId && <em className="tag">active</em>}
            </button>
            <span className="muted">
              {c.via} · {c.id}
            </span>
            <button
              type="button"
              className="ghost"
              onClick={() => onShutdown(c.id)}
              title="Shutdown (disconnect) this connection"
            >
              shutdown
            </button>
          </li>
        ))}
      </ul>

      <h3 className="proc-sub">Subprocesses</h3>
      <p className="muted proc-empty">
        No subprocesses are tracked yet. Sidecar children are spawned via{" "}
        <code>McpSpawn</code> (<code>connections/child_process.rs</code>), but a live
        process registry is not wired into the app — listing running children is a
        documented future extension and will replace this placeholder.
      </p>
    </section>
  );
}
