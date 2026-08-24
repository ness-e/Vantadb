// DESKTOP-37: lente MEMORIA — sexta superficie del Studio. Visualiza lo que
// vanta-memory ya persiste (hoy invisible): escenas con heat (soft-delete
// visible), snapshot de persona con diff vs la última vista, skills versionadas
// por content-hash, y generation-log filtrable por capa L1/L2/L3.
// Read-only sobre el bridge de DESKTOP-36; click en genlog con anchor_id →
// Inspector vía get() real (los demás datos no son VantaMemoryRecord → detalle
// inline, evitando targets sintéticos para el vantaPut del Inspector).
import { useEffect, useMemo, useState } from "react";
import {
  GenlogEntry,
  GenerationLayer,
  get,
  memoryGenlogQuery,
  memoryPersonaGet,
  memorySceneList,
  memorySceneRead,
  memorySkillList,
  PersonaSnapshot,
  SceneBlock,
  SceneEntry,
  StoredSkillRecord,
  vantaErrorMessage,
  type MemoryRecord,
} from "../../vanta";

interface LensActions {
  onNotice: (msg: string) => void;
  onError: (msg: string) => void;
  onOpenRecord: (record: MemoryRecord, score: number | null) => void;
}

/** Diff por líneas old→new. ponytail: comparación por Sets (duplicados se
 * colapsan); LCS solo si el diff deja de ser legible en la práctica. */
export function lineDiff(
  oldText: string,
  newText: string,
): { added: string[]; removed: string[] } {
  const before = new Set(oldText.split("\n"));
  const after = new Set(newText.split("\n"));
  return {
    added: [...after].filter((l) => !before.has(l)),
    removed: [...before].filter((l) => !after.has(l)),
  };
}

function PanelEmpty({ msg }: { msg: string }) {
  return (
    <p className="border-2 border-dashed border-foreground bg-background p-4 font-tech text-[11px] uppercase tracking-widest text-muted-foreground">
      {msg}
    </p>
  );
}

function fmtMs(ms: number): string {
  return new Date(ms).toLocaleString();
}

// --- ESCENAS -------------------------------------------------------------------
function ScenesPanel({ sessionKey, onError }: { sessionKey: string } & Pick<LensActions, "onError">) {
  const [scenes, setScenes] = useState<SceneEntry[] | null>(null);
  const [detail, setDetail] = useState<SceneBlock | null>(null);

  useEffect(() => {
    let alive = true;
    setScenes(null);
    setDetail(null);
    memorySceneList(sessionKey)
      .then((s) => alive && setScenes(s))
      .catch((err) => alive && onError(vantaErrorMessage(err)));
    return () => {
      alive = false;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sessionKey]);

  async function openScene(name: string) {
    try {
      // El índice lista también soft-deleted; read falla (404) para esos →
      // el catch los marca como borrados visiblemente.
      setDetail(await memorySceneRead(sessionKey, name));
    } catch {
      setDetail({
        scene_name: name,
        meta: { created: "", updated: "", summary: "", heat: 0 },
        content: "",
        deleted: true,
      });
    }
  }

  const maxHeat = useMemo(
    () => Math.max(1, ...(scenes ?? []).map((s) => s.heat)),
    [scenes],
  );

  if (!scenes) return <PanelEmpty msg="cargando escenas…" />;
  if (scenes.length === 0) return <PanelEmpty msg={`sin escenas en "${sessionKey}"`} />;

  return (
    <div className="space-y-3">
      <ol className="space-y-2">
        {scenes.map((s) => (
          <li key={s.filename}>
            <button
              type="button"
              onClick={() => openScene(s.filename)}
              className="press block w-full border-2 border-foreground bg-background p-3 text-left"
            >
              <div className="flex items-baseline justify-between gap-2">
                <code className="truncate font-tech text-sm">{s.filename}</code>
                <span className="shrink-0 border-2 border-foreground bg-neon px-1.5 py-0.5 font-tech text-[10px] text-background">
                  heat {s.heat.toFixed(1)}
                </span>
              </div>
              <div className="mt-1 h-1.5 w-full border border-foreground bg-card">
                <div className="h-full bg-neon" style={{ width: `${(s.heat / maxHeat) * 100}%` }} />
              </div>
              <p className="mt-1 truncate font-tech text-[10px] text-muted-foreground">{s.summary}</p>
            </button>
          </li>
        ))}
      </ol>
      {detail && (
        <section className="border-4 border-foreground bg-card p-4" aria-label="Detalle de escena">
          <div className="flex items-center justify-between gap-2">
            <code className="font-tech text-sm">{detail.scene_name}</code>
            <button type="button" onClick={() => setDetail(null)} className="press border-2 border-foreground px-2 py-0.5 font-tech text-[10px]" aria-label="Cerrar detalle">
              ✕
            </button>
          </div>
          {detail.deleted ? (
            <p className="mt-2 font-tech text-[11px] uppercase tracking-widest text-neon">
              ⌫ soft-deleted — bloque no accesible
            </p>
          ) : (
            <>
              <div className="mt-1 flex gap-3 font-tech text-[10px] text-muted-foreground">
                <span>creada {detail.meta.created}</span>
                <span>act. {detail.meta.updated}</span>
                <span>heat {detail.meta.heat.toFixed(1)}</span>
              </div>
              <pre className="mt-2 max-h-72 overflow-auto whitespace-pre-wrap border-2 border-foreground bg-background p-3 font-tech text-xs">{detail.content}</pre>
            </>
          )}
        </section>
      )}
    </div>
  );
}

// --- PERSONA -------------------------------------------------------------------
function PersonaPanel({ sessionKey }: { sessionKey: string }) {
  const [snap, setSnap] = useState<PersonaSnapshot | null | undefined>(undefined);
  const [diff, setDiff] = useState<{ added: string[]; removed: string[] } | null>(null);

  useEffect(() => {
    let alive = true;
    setSnap(undefined);
    setDiff(null);
    memoryPersonaGet(sessionKey)
      .then((p) => {
        if (!alive || !p) {
          if (alive) setSnap(p);
          return;
        }
        const storeKey = `vanta-persona-last:${sessionKey}`;
        const prev = localStorage.getItem(storeKey);
        if (prev !== null && prev !== p.content) setDiff(lineDiff(prev, p.content));
        localStorage.setItem(storeKey, p.content);
        setSnap(p);
      })
      .catch(() => {});
    return () => {
      alive = false;
    };
  }, [sessionKey]);

  if (snap === undefined) return <PanelEmpty msg="cargando persona…" />;
  if (snap === null) return <PanelEmpty msg={`sin snapshot de persona en "${sessionKey}" — generá uno con L3`} />;

  return (
    <div className="space-y-3">
      <div className="flex gap-3 font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
        <span className="border-2 border-foreground bg-neon px-1.5 py-0.5 text-background">{snap.mode}</span>
        <span>generada {snap.generated_at}</span>
      </div>
      <pre className="max-h-96 overflow-auto whitespace-pre-wrap border-2 border-foreground bg-background p-3 font-tech text-xs">{snap.content}</pre>
      {diff && (
        <details open className="border-2 border-dashed border-foreground bg-background p-3">
          <summary className="cursor-pointer font-tech text-[10px] uppercase tracking-widest text-neon">
            diff vs última snapshot vista (+{diff.added.length} / −{diff.removed.length})
          </summary>
          <ul className="mt-2 space-y-0.5 font-tech text-xs">
            {diff.removed.map((l) => (
              <li key={`-${l}`} className="text-muted-foreground line-through">− {l}</li>
            ))}
            {diff.added.map((l) => (
              <li key={`+${l}`} className="text-neon">+ {l}</li>
            ))}
          </ul>
        </details>
      )}
    </div>
  );
}

// --- SKILLS --------------------------------------------------------------------
interface SkillGroup {
  name: string;
  /** versiones asc por updated_at_ms (timeline). */
  versions: StoredSkillRecord[];
}

function SkillsPanel({ onError }: Pick<LensActions, "onError">) {
  const [groups, setGroups] = useState<SkillGroup[] | null>(null);
  const [viewing, setViewing] = useState<{ name: string; hash: number; content: string } | null>(null);

  useEffect(() => {
    let alive = true;
    setGroups(null);
    setViewing(null);
    memorySkillList()
      .then((skills) => {
        if (!alive) return;
        const byName = new Map<string, StoredSkillRecord[]>();
        for (const s of skills) {
          const arr = byName.get(s.name) ?? [];
          arr.push(s);
          byName.set(s.name, arr);
        }
        setGroups(
          [...byName.entries()].map(([name, versions]) => ({
            name,
            versions: versions.sort((a, b) => a.updated_at_ms - b.updated_at_ms),
          })),
        );
      })
      .catch((err) => alive && onError(vantaErrorMessage(err)));
    return () => {
      alive = false;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function shortHash(hash: number): string {
    return (hash >>> 0).toString(16).padStart(8, "0").slice(-8);
  }

  if (!groups) return <PanelEmpty msg="cargando skills…" />;
  if (groups.length === 0) return <PanelEmpty msg="sin skills extraídas aún" />;

  return (
    <div className="space-y-3">
      {groups.map((g) => (
        <section key={g.name} className="border-2 border-foreground bg-background p-3">
          <div className="flex items-baseline justify-between gap-2">
            <code className="font-tech text-sm">{g.name}</code>
            <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
              {g.versions.length === 1 ? "1 versión" : `${g.versions.length} versiones`}
            </span>
          </div>
          <p className="mt-0.5 truncate font-tech text-[10px] text-muted-foreground">{g.versions[g.versions.length - 1].description}</p>
          {/* timeline por content-hash: una fila por versión, asc */}
          <ol className="mt-2 space-y-1 border-l-2 border-foreground pl-3">
            {g.versions.map((v) => (
              <li key={`${v.name}:${v.content_hash}`}>
                <button
                  type="button"
                  onClick={() => setViewing({ name: v.name, hash: v.content_hash, content: v.content })}
                  className="press flex w-full items-center gap-2 border-2 border-foreground bg-card px-2 py-1 text-left font-tech text-[10px]"
                >
                  <span className="bg-neon px-1 text-background">{shortHash(v.content_hash)}</span>
                  <span className="text-muted-foreground">{fmtMs(v.updated_at_ms)}</span>
                </button>
              </li>
            ))}
          </ol>
        </section>
      ))}
      {viewing && (
        <section className="border-4 border-foreground bg-card p-4" aria-label="Contenido de skill">
          <div className="flex items-center justify-between gap-2">
            <code className="font-tech text-sm">
              {viewing.name}
              <span className="ml-2 bg-neon px-1 font-tech text-[10px] text-background">
                {(viewing.hash >>> 0).toString(16).padStart(8, "0").slice(-8)}
              </span>
            </code>
            <button type="button" onClick={() => setViewing(null)} className="press border-2 border-foreground px-2 py-0.5 font-tech text-[10px]" aria-label="Cerrar skill">
              ✕
            </button>
          </div>
          <pre className="mt-2 max-h-80 overflow-auto whitespace-pre-wrap border-2 border-foreground bg-background p-3 font-tech text-xs">{viewing.content}</pre>
        </section>
      )}
    </div>
  );
}

// --- GENLOG --------------------------------------------------------------------
const LAYER_FILTERS: { id: GenerationLayer | "todos"; label: string }[] = [
  { id: "todos", label: "TODOS" },
  { id: "l1", label: "L1" },
  { id: "l2", label: "L2" },
  { id: "l3", label: "L3" },
];

function GenlogPanel({ sessionKey, onOpenRecord, onError }: LensActions & { sessionKey: string }) {
  const [layer, setLayer] = useState<GenerationLayer | "todos">("todos");
  const [entries, setEntries] = useState<GenlogEntry[] | null>(null);

  useEffect(() => {
    let alive = true;
    setEntries(null);
    memoryGenlogQuery(sessionKey, layer === "todos" ? undefined : layer, 200)
      .then((e) => alive && setEntries(e))
      .catch((err) => alive && onError(vantaErrorMessage(err)));
    return () => {
      alive = false;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sessionKey, layer]);

  async function openAnchor(anchorId?: string) {
    // Solo los entries con anchor apuntan a un VantaMemoryRecord real →
    // Inspector vía get(); el resto es provenance sin record detrás.
    if (!anchorId) return;
    try {
      onOpenRecord(await get(anchorId), null);
    } catch (err) {
      onError(vantaErrorMessage(err));
    }
  }

  return (
    <div className="space-y-3">
      <div className="flex gap-1" role="group" aria-label="Filtrar por capa">
        {LAYER_FILTERS.map((f) => (
          <button
            key={f.id}
            type="button"
            onClick={() => setLayer(f.id)}
            aria-pressed={layer === f.id}
            className={`press border-2 border-foreground px-2 py-1 font-tech text-[10px] uppercase tracking-widest ${
              layer === f.id ? "bg-neon text-background" : "bg-background"
            }`}
          >
            {f.label}
          </button>
        ))}
      </div>
      {!entries ? (
        <PanelEmpty msg="cargando generation log…" />
      ) : entries.length === 0 ? (
        <PanelEmpty msg={`sin entradas ${layer === "todos" ? "" : layer.toUpperCase() + " "}en "${sessionKey}"`} />
      ) : (
        <ol className="space-y-1 border-l-2 border-foreground pl-3">
          {entries.map((e, i) => (
            <li key={`${e.ts_ms}:${i}`}>
              <button
                type="button"
                onClick={() => openAnchor(e.anchor_id)}
                disabled={!e.anchor_id}
                title={e.anchor_id ? `Abrir record ${e.anchor_id}` : "entrada sin record anclado"}
                className={`press flex w-full items-center gap-2 border-2 border-foreground px-2 py-1 text-left font-tech text-[10px] ${
                  e.anchor_id ? "bg-background" : "cursor-default bg-card opacity-70"
                }`}
              >
                <span className="w-6 shrink-0 bg-neon px-1 text-center text-background">{e.layer.toUpperCase()}</span>
                <span className="shrink-0">{e.status === "succeeded" ? "✓" : "✗"}</span>
                <span className="shrink-0 text-muted-foreground">{fmtMs(e.ts_ms)}</span>
                {e.status === "failed" && e.error && (
                  <span className="truncate text-neon" title={e.error}>{e.error}</span>
                )}
                {e.anchor_id && <code className="ml-auto truncate text-muted-foreground">{e.anchor_id}</code>}
              </button>
            </li>
          ))}
        </ol>
      )}
    </div>
  );
}

// --- SHELL DE LA LENTE -----------------------------------------------------------
type MemTab = "escenas" | "persona" | "skills" | "genlog";

const TABS: { id: MemTab; label: string }[] = [
  { id: "escenas", label: "ESCENAS" },
  { id: "persona", label: "PERSONA" },
  { id: "skills", label: "SKILLS" },
  { id: "genlog", label: "GENLOG" },
];

export default function MemoryLens({
  active,
  sessionKey: initialSession,
  ...actions
}: LensActions & { active: boolean; sessionKey?: string }) {
  const [session, setSession] = useState(initialSession ?? "");
  const [sessionInput, setSessionInput] = useState(initialSession ?? "");
  const [tab, setTab] = useState<MemTab>("escenas");

  if (!active) {
    return <PanelEmpty msg="sin backend activo — conectá uno (o sembrá datos con vanta-seed) para explorar memoria" />;
  }

  return (
    <div className="space-y-4">
      {/* Selector de sesión (contexto de todas las consultas) */}
      <form
        onSubmit={(e) => {
          e.preventDefault();
          setSession(sessionInput.trim());
        }}
        className="flex items-center gap-2 border-2 border-foreground bg-background p-3"
      >
        <label htmlFor="mem-session" className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
          sesión
        </label>
        <input
          id="mem-session"
          value={sessionInput}
          onChange={(e) => setSessionInput(e.target.value)}
          placeholder="user-1"
          className="min-w-0 flex-1 border-2 border-foreground bg-card px-2 py-1 font-tech text-sm outline-none focus:bg-neon focus:text-background"
        />
        <button type="submit" className="btn-neon-glow press border-2 border-foreground bg-neon px-3 py-1 font-tech text-[10px] font-bold uppercase tracking-widest text-background" disabled={!sessionInput.trim()}>
          CARGAR
        </button>
      </form>

      {!session ? (
        <PanelEmpty msg="indicá una session_key (la misma que usó vanta-seed) y CARGAR" />
      ) : (
        <>
          <nav className="flex border-2 border-foreground bg-background" aria-label="Secciones de memoria">
            {TABS.map((t) => (
              <button
                key={t.id}
                type="button"
                onClick={() => setTab(t.id)}
                aria-selected={tab === t.id}
                role="tab"
                className={`flex-1 border-r-2 border-foreground px-1 py-2 font-tech text-[10px] uppercase tracking-widest last:border-r-0 ${
                  tab === t.id ? "bg-neon text-background" : "text-muted-foreground hover:text-foreground"
                }`}
              >
                {tab === t.id ? `◆ ${t.label}` : t.label}
              </button>
            ))}
          </nav>

          {tab === "escenas" && <ScenesPanel sessionKey={session} onError={actions.onError} />}
          {tab === "persona" && <PersonaPanel sessionKey={session} />}
          {tab === "skills" && <SkillsPanel onError={actions.onError} />}
          {tab === "genlog" && <GenlogPanel sessionKey={session} {...actions} />}
        </>
      )}
    </div>
  );
}
