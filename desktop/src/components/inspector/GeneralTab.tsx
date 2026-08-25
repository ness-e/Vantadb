// General tab (VS-06): identidad del registro (key/ns/node_id mono), timestamps
// created/updated con relativo, version, y TTL editable con countdown en vivo.
import { useEffect, useState } from "react";
import type { MemoryRecord } from "../../vanta";
import { fmtDateTime, fmtDuration, fmtRelative, ttlToMs, type TtlDraft } from "./shared";
import { TriangleAlert } from "lucide-react";

interface Props {
  record: MemoryRecord;
  score: number | null;
  ttl: TtlDraft;
  setTtl: (d: TtlDraft) => void;
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex gap-2 border-b-2 border-foreground py-2 last:border-b-0">
      <span className="w-24 shrink-0 font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
        {label}
      </span>
      <div className="min-w-0 flex-1 break-words">{children}</div>
    </div>
  );
}

export default function GeneralTab({ record, score, ttl, setTtl }: Props) {
  // Countdown TTL en vivo (1s; el grid usa 30s, acá el inspector es el foco).
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(t);
  }, []);

  const [editingTtl, setEditingTtl] = useState(false);
  const expiresMs = ttlToMs(ttl, now);
  const total =
    expiresMs != null ? expiresMs - (record.updated_at_ms ?? record.created_at_ms ?? expiresMs) : 0;
  const remain = expiresMs != null ? expiresMs - now : 0;
  const frac = total > 0 ? Math.min(1, Math.max(0, remain / total)) : 1;
  // VS-18/P15: TTL nunca es solo-color — estado = ícono + texto + patrón/fill.
  const expired = remain <= 0;
  const expiring = !expired && frac < 0.2;
  const barFill = expired ? "bg-muted-foreground" : expiring ? "stripes-neon" : "bg-foreground";

  return (
    <div>
      <div className="border-2 border-foreground bg-background p-3">
        <Row label="key">
          <code className="text-sm">{record.id}</code>
        </Row>
        <Row label="namespace">
          <span className="border-2 border-foreground bg-card px-2 py-0.5 font-tech text-[11px]">
            {record.namespace}
          </span>
        </Row>
        {record.node_id && (
          <Row label="node_id">
            <code className="break-all font-tech text-[11px] text-muted-foreground">{record.node_id}</code>
          </Row>
        )}
        {score != null && (
          <Row label="score">
            <span className="font-tech text-[11px] text-neon">{score.toFixed(3)}</span>
          </Row>
        )}
        <Row label="version">
          <span className="border-2 border-foreground bg-card px-1.5 py-0.5 font-tech text-[10px]">
            v{record.version ?? "—"}
          </span>
        </Row>
        <Row label="created">
          <span className="text-sm">
            {record.created_at_ms != null ? fmtDateTime(record.created_at_ms) : "—"}
            {record.created_at_ms != null && (
              <span className="ml-1 font-tech text-[10px] text-muted-foreground">
                ({fmtRelative(record.created_at_ms, now)})
              </span>
            )}
          </span>
        </Row>
        <Row label="updated">
          <span className="text-sm">
            {record.updated_at_ms != null ? fmtDateTime(record.updated_at_ms) : "—"}
            {record.updated_at_ms != null && (
              <span className="ml-1 font-tech text-[10px] text-neon">
                ({fmtRelative(record.updated_at_ms, now)})
              </span>
            )}
          </span>
        </Row>
      </div>

      {/* TTL editable — draft vive en Inspector; Guardar/Revertir en el footer */}
      <div className="mt-3 border-2 border-foreground bg-background p-3">
        <div className="flex items-center justify-between">
          <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
            TTL
          </span>
          <button
            type="button"
            onClick={() => setEditingTtl((v) => !v)}
            className="press border-2 border-foreground bg-card px-2 py-0.5 font-tech text-[10px]"
          >
            {editingTtl ? "cerrar" : "editar"}
          </button>
        </div>

        {!editingTtl ? (
          expiresMs == null ? (
            <p className="mt-2 font-tech text-[11px] text-muted-foreground">sin expiración</p>
          ) : (
            <div className="mt-2">
              <div className="flex items-center justify-between font-tech text-[10px]">
                <span className={expired ? "font-bold text-foreground" : "text-foreground"}>
                  {expired ? (
                    "✕ EXPIRED"
                  ) : expiring ? (
                    <>
                      <TriangleAlert className="mr-0.5 inline h-3 w-3 align-[-2px]" strokeWidth={2.5} aria-hidden="true" />
                      {fmtDuration(remain)} left
                    </>
                  ) : (
                    `● ${fmtDuration(remain)} left`
                  )}
                </span>
                <span className="text-muted-foreground">{fmtDateTime(expiresMs)}</span>
              </div>
              <div className="mt-1 h-2 w-full border-2 border-foreground bg-card">
                <div
                  className={`block h-full ${barFill}`}
                  style={{ width: `${expired ? 0 : Math.round(frac * 100)}%` }}
                />
              </div>
            </div>
          )
        ) : (
          <div className="mt-2 space-y-2 font-tech text-[11px]">
            <label className="flex items-center gap-2">
              <input
                type="radio"
                checked={ttl.mode === "never"}
                onChange={() => setTtl({ mode: "never", relMinutes: 0, absLocal: "" })}
              />
              nunca expira
            </label>
            <label className="flex items-center gap-2">
              <input
                type="radio"
                checked={ttl.mode === "relative"}
                onChange={() => setTtl({ ...ttl, mode: "relative" })}
              />
              expira en
              <input
                type="number"
                min={1}
                value={ttl.relMinutes}
                onChange={(e) =>
                  setTtl({ ...ttl, mode: "relative", relMinutes: Number(e.target.value) })
                }
                className="w-20 border-2 border-foreground bg-card px-1 py-0.5 font-tech text-[11px]"
                aria-label="Minutos hasta expiración"
              />
              min
            </label>
            <label className="flex items-center gap-2">
              <input
                type="radio"
                checked={ttl.mode === "absolute"}
                onChange={() => setTtl({ ...ttl, mode: "absolute" })}
              />
              hasta
              <input
                type="datetime-local"
                value={ttl.absLocal}
                onChange={(e) => setTtl({ ...ttl, mode: "absolute", absLocal: e.target.value })}
                className="border-2 border-foreground bg-card px-1 py-0.5 font-tech text-[11px]"
                aria-label="Fecha y hora de expiración"
              />
            </label>
            <p className="font-tech text-[10px] text-muted-foreground">
              commit explícito: usá GUARDAR / REVERTIR en el pie del inspector
            </p>
          </div>
        )}
      </div>
    </div>
  );
}