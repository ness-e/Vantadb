// ProxyDashboard (DESKTOP-38): panel operativo del proxy local, consumiendo
// REST directo (`GET /snapshot` en vanta-proxy — proceso aparte, NO bridge
// nativo). Paneles: TurnReports (protocolo/modelo/status/duración), sesiones
// activas team→agent→task con TTL, cola write-back pendiente y rate-limit.
// Polling cada 5s mientras la superficie está montada; sin URL configurada
// muestra el formulario de configuración y NO polla.
import { FormEvent, ReactNode, useEffect, useState } from "react";

const LS_KEY = "vanta.proxy.url";
/** Evento disparado al guardar la URL para que el shell refresque su botón. */
export const PROXY_URL_EVENT = "vanta-proxy-url";

export function proxyUrl(): string {
  try {
    return localStorage.getItem(LS_KEY) ?? "";
  } catch {
    return "";
  }
}

function setProxyUrl(url: string): void {
  try {
    if (url) localStorage.setItem(LS_KEY, url);
    else localStorage.removeItem(LS_KEY);
    window.dispatchEvent(new Event(PROXY_URL_EVENT));
  } catch {
    // storage bloqueado → URL solo de sesión vía estado local
  }
}

interface TurnReportWire {
  timestamp_ms: number;
  space_id: string;
  protocol: string;
  model: string;
  status: number;
  duration_ms: number | string;
}

interface SessionWire {
  key: string;
  stage: "team" | "agent" | "task";
  updated_at_ms: number;
  expires_at_ms?: number;
}

interface SnapshotWire {
  turns: TurnReportWire[];
  sessions_active?: number;
  sessions: SessionWire[];
  writeback: { pending_labels: string[]; pending_count: number };
  rate_limit: { limit_per_minute: number; hits_total: number; degraded: boolean };
}

async function fetchSnapshot(base: string): Promise<SnapshotWire> {
  const res = await fetch(`${base.replace(/\/+$/, "")}/snapshot`);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return (await res.json()) as SnapshotWire;
}

/** TTL restante legible ("23m", "45s", expirado). */
export function ttlLabel(expiresAtMs: number | undefined): string {
  if (expiresAtMs === undefined) return "sin TTL";
  const s = Math.round((expiresAtMs - Date.now()) / 1000);
  if (s <= 0) return "expirado";
  if (s >= 60) return `${Math.floor(s / 60)}m`;
  return `${s}s`;
}

const stageGlyph = { team: "◉", agent: "◈", task: "◆" } as const;

function Panel({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section aria-label={title} className="border-[3px] border-foreground bg-card p-4">
      <h2 className="mb-3 mt-0 font-tech text-xs uppercase tracking-widest text-muted-foreground">
        {title}
      </h2>
      {children}
    </section>
  );
}

export default function ProxyDashboard() {
  const [configured, setConfigured] = useState(!!proxyUrl());
  const [draft, setDraft] = useState("");
  const [snap, setSnap] = useState<SnapshotWire | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [polledAt, setPolledAt] = useState<number | null>(null);

  // Polling compartido-por-instancia: la surface solo está montada al abrirla.
  useEffect(() => {
    if (!configured) return;
    let alive = true;
    async function tick(): Promise<void> {
      try {
        const s = await fetchSnapshot(proxyUrl());
        if (!alive) return;
        setSnap(s);
        setError(null);
        setPolledAt(Date.now());
      } catch (e) {
        if (alive) setError(e instanceof Error ? e.message : String(e));
      }
    }
    void tick();
    const timer = window.setInterval(tick, 5000);
    return () => {
      alive = false;
      window.clearInterval(timer);
    };
  }, [configured]);

  function handleSave(e: FormEvent) {
    e.preventDefault();
    const clean = draft.trim().replace(/\/+$/, "");
    if (!clean) return;
    setProxyUrl(clean);
    setConfigured(true);
  }

  if (!configured) {
    return (
      <div className="mx-auto max-w-2xl p-6">
        <section className="press-lg border-4 border-foreground bg-card p-8">
          <div className="font-display text-2xl text-stencil">PROXY</div>
          <p className="mt-2 font-tech text-[11px] uppercase tracking-widest text-muted-foreground">
            configurá la URL base del proxy local (default :8096)
          </p>
          <form onSubmit={handleSave} className="mt-4 flex gap-2">
            <input
              value={draft}
              onChange={(e) => setDraft(e.target.value)}
              placeholder="http://127.0.0.1:8096"
              aria-label="URL base del proxy"
              className="min-w-0 flex-1 border-2 border-foreground bg-background px-3 py-1.5 text-sm placeholder:text-muted-foreground"
            />
            <button type="submit" className="press border-2 border-foreground bg-neon px-3 py-1.5 text-xs font-bold text-background">
              CONECTAR
            </button>
          </form>
        </section>
      </div>
    );
  }

  const turns = snap?.turns ?? [];
  const sessions = snap?.sessions ?? [];
  const wb = snap?.writeback;
  const rl = snap?.rate_limit;

  return (
    <div className="mx-auto max-w-6xl space-y-5 p-6">
      {/* TurnReports */}
      <Panel title={`Turn reports${snap ? ` · ${turns.length} recientes` : ""}`}>
        {error && <p className="text-sm text-muted-foreground">proxy no disponible: {error}</p>}
        {!error && turns.length === 0 && (
          <p className="text-sm text-muted-foreground">esperando el primer snapshot…</p>
        )}
        {turns.length > 0 && (
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b-2 border-foreground text-left font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
                <th className="py-1 pr-2">hora</th>
                <th className="py-1 pr-2">protocolo</th>
                <th className="py-1 pr-2">modelo</th>
                <th className="py-1 pr-2">status</th>
                <th className="py-1 pr-2">duración</th>
                <th className="py-1">space</th>
              </tr>
            </thead>
            <tbody>
              {[...turns].reverse().map((t, i) => (
                <tr key={`${t.timestamp_ms}-${i}`} className="border-b border-muted last:border-b-0">
                  <td className="py-1 pr-2">{new Date(t.timestamp_ms).toLocaleTimeString()}</td>
                  <td className="py-1 pr-2">{t.protocol}</td>
                  <td className="py-1 pr-2">{t.model}</td>
                  <td className="py-1 pr-2">
                    <span className={t.status < 400 ? "text-neon" : ""}>{t.status}</span>
                  </td>
                  <td className="py-1 pr-2">{t.duration_ms}ms</td>
                  <td className="truncate py-1">{t.space_id}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Panel>

      <div className="grid grid-cols-1 gap-5 lg:grid-cols-2">
        {/* Sesiones activas team→agent→task */}
        <Panel title={`Sesiones activas${snap ? ` · ${sessions.length}` : ""}`}>
          {sessions.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              sin sesiones — el estado aparece cuando llega tráfico con session header
            </p>
          ) : (
            <ul className="space-y-1 text-sm">
              {sessions.map((s) => (
                <li key={s.key} className="flex items-center gap-2 border-b border-muted pb-1 last:border-b-0">
                  <span className="text-neon" aria-hidden>{stageGlyph[s.stage]}</span>
                  <span className="truncate" title={s.key}>{s.key}</span>
                  <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
                    {s.stage}
                  </span>
                  <span className="ml-auto shrink-0 font-tech text-[10px]" title="TTL restante">
                    {ttlLabel(s.expires_at_ms)}
                  </span>
                </li>
              ))}
            </ul>
          )}
        </Panel>

        {/* Write-back pendiente */}
        <Panel title="Write-back pendiente">
          {wb ? (
            <>
              <p className="m-0 text-[1.4rem] font-bold leading-tight">{wb.pending_count}</p>
              <p className="mt-1 text-sm text-muted-foreground">escrituras L0 esperando flush</p>
              {wb.pending_labels.length > 0 && (
                <ul className="mt-2 space-y-1 text-sm">
                  {wb.pending_labels.map((l) => (
                    <li key={l} className="truncate" title={l}>· {l}</li>
                  ))}
                </ul>
              )}
            </>
          ) : (
            <p className="text-sm text-muted-foreground">—</p>
          )}
        </Panel>

        {/* Rate limit */}
        <Panel title="Rate limit">
          {rl ? (
            <dl className="m-0 grid grid-cols-3 gap-2 text-center">
              <div>
                <dt className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">límite</dt>
                <dd className="m-0 text-[1.4rem] font-bold leading-tight">{rl.limit_per_minute}/min</dd>
              </div>
              <div>
                <dt className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">hits 429</dt>
                <dd className="m-0 text-[1.4rem] font-bold leading-tight">{rl.hits_total}</dd>
              </div>
              <div>
                <dt className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">estado</dt>
                <dd className={`m-0 text-sm font-bold leading-tight ${rl.degraded ? "text-neon" : ""}`}>
                  {rl.degraded ? "degraded (fail-open)" : "ok"}
                </dd>
              </div>
            </dl>
          ) : (
            <p className="text-sm text-muted-foreground">—</p>
          )}
        </Panel>

        {/* Conexión */}
        <Panel title="Conexión">
          <p className="m-0 text-sm">
            <span className="text-muted-foreground">proxy:</span> {proxyUrl()}
          </p>
          {polledAt && (
            <p className="mt-1 font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
              último poll {new Date(polledAt).toLocaleTimeString()} · cada 5s
            </p>
          )}
          <button
            type="button"
            onClick={() => {
              setProxyUrl("");
              setDraft("");
              setSnap(null);
              setError(null);
              setPolledAt(null);
              setConfigured(false);
            }}
            className="press mt-2 border-2 border-foreground bg-background px-2 py-1 text-xs"
          >
            ✎ cambiar URL
          </button>
        </Panel>
      </div>
    </div>
  );
}
