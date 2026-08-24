// DESKTOP-31: superficie AJUSTES — perfiles de conexión multi-perfil
// (path nativo / URL server con Bearer token), defaults de búsqueda e idioma.
// Persistencia vía connectionPrefs (localStorage inyectable, patrón DESKTOP-23);
// la conexión real pasa por actions.connectNativePath/connectServerCfg del hook
// useConnectionState — sin comandos Tauri nuevos (el transporte Bearer ya vive
// en Rust: ServerClientConfig.token).
import { FormEvent, useState } from "react";
import { connectionPrefs, ConnectionProfile, profileTarget } from "../store/connections";

interface Props {
  /** WEB-05: en build embebido no hay multi-conexión → ocultar perfiles. */
  embedded?: boolean;
  busy?: boolean;
  onConnectNative: (path: string) => Promise<string | null>;
  onConnectServer: (url: string, port: number, token: string) => Promise<string | null>;
  onNotice: (msg: string) => void;
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="border-4 border-foreground bg-card p-5 shadow-[6px_6px_0_0_#000] dark:shadow-[6px_6px_0_0_#FBF9F5]">
      <h2 className="m-0 font-tech text-xs uppercase tracking-widest text-neon">{title}</h2>
      <div className="mt-3">{children}</div>
    </section>
  );
}

const inputCls = "w-full border-2 border-foreground bg-background px-2.5 py-1.5 text-sm";

export default function Settings({ embedded = false, busy = false, onConnectNative, onConnectServer, onNotice }: Props) {
  // Estado local hidratado del store una vez al montar; cada mutación es
  // write-through (el store persiste y este estado espeja para el render).
  const [prefs, setPrefs] = useState(connectionPrefs.get());
  const sync = () => setPrefs(connectionPrefs.get());

  // Formulario de nuevo perfil.
  const [name, setName] = useState("");
  const [kind, setKind] = useState<ConnectionProfile["kind"]>("server");
  const [path, setPath] = useState("vantadb-local");
  const [url, setUrl] = useState("http://127.0.0.1");
  const [port, setPort] = useState(8080);
  const [token, setToken] = useState("");

  async function handleAddProfile(e: FormEvent) {
    e.preventDefault();
    if (!name.trim()) return;
    connectionPrefs.upsertProfile(
      kind === "native"
        ? { name: name.trim(), kind, path }
        : { name: name.trim(), kind, url, port, token: token || undefined },
    );
    setName("");
    setToken("");
    sync();
    onNotice(`Perfil "${name.trim()}" guardado.`);
  }

  async function connectProfile(p: ConnectionProfile) {
    const id =
      p.kind === "native"
        ? await onConnectNative(p.path ?? "")
        : await onConnectServer(p.url ?? "", p.port ?? 8080, p.token ?? "");
    if (id) {
      connectionPrefs.set({ activeProfileId: p.id });
      sync();
      onNotice(`Conectado vía perfil "${p.name}".`);
    }
  }

  function removeProfile(p: ConnectionProfile) {
    connectionPrefs.removeProfile(p.id);
    sync();
    onNotice(`Perfil "${p.name}" eliminado.`);
  }

  const profiles = prefs.profiles ?? [];

  return (
    <div className="mx-auto max-w-3xl space-y-5 p-6">
      {/* ===== (1+2) PERFILES DE CONEXIÓN + AUTH BEARER ===== */}
      {!embedded && (
        <Section title="Conexiones guardadas">
          {profiles.length === 0 ? (
            <p className="font-tech text-[11px] text-muted-foreground">
              Sin perfiles — guardá uno abajo para reconectar con un clic.
            </p>
          ) : (
            <ul className="m-0 list-none space-y-1 p-0">
              {profiles.map((p) => {
                const active = prefs.activeProfileId === p.id;
                return (
                  <li key={p.id} className={`flex items-center gap-2 border-t-2 border-foreground py-2 ${active ? "bg-neon/10" : ""}`}>
                    <span className={`inline-block h-2 w-2 shrink-0 rounded-full border border-foreground ${active ? "bg-neon" : "bg-paper"}`} />
                    <button type="button" onClick={() => void connectProfile(p)} className="cursor-pointer border-none bg-transparent text-left" disabled={busy}>
                      <span className="font-semibold">{p.name}</span>
                      <span className="ml-2 font-tech text-[10px] uppercase tracking-widest text-muted-foreground">{p.kind}</span>
                      <span className="ml-2 truncate font-tech text-[10px] text-muted-foreground">{profileTarget(p)}</span>
                    </button>
                    <button type="button" onClick={() => void connectProfile(p)} disabled={busy} className="press ml-auto shrink-0 border-2 border-foreground bg-background px-2 py-0.5 text-[10px] font-semibold disabled:opacity-50">
                      conectar
                    </button>
                    <button type="button" onClick={() => removeProfile(p)} aria-label={`Eliminar ${p.name}`} className="press flex h-7 w-7 shrink-0 items-center justify-center border-2 border-foreground bg-background text-[10px]">
                      ✕
                    </button>
                  </li>
                );
              })}
            </ul>
          )}

          <form onSubmit={handleAddProfile} className="mt-4 space-y-2 border-t-2 border-dashed border-muted-foreground pt-4">
            <div className="flex gap-2">
              <input value={name} onChange={(e) => setName(e.target.value)} placeholder="Nombre del perfil" aria-label="Nombre del perfil" className={inputCls} />
              <select value={kind} onChange={(e) => setKind(e.target.value as ConnectionProfile["kind"])} aria-label="Tipo de conexión" className={inputCls}>
                <option value="server">Server remoto</option>
                <option value="native">Nativo (path)</option>
              </select>
            </div>
            {kind === "native" ? (
              <input value={path} onChange={(e) => setPath(e.target.value)} placeholder="Ruta de base de datos" aria-label="Ruta nativa" className={inputCls} />
            ) : (
              <>
                <div className="flex gap-2">
                  <input value={url} onChange={(e) => setUrl(e.target.value)} placeholder="http://host" aria-label="URL del servidor" className={`${inputCls} min-w-0 flex-1`} />
                  <input type="number" value={port} onChange={(e) => setPort(Number(e.target.value) || 0)} placeholder="Puerto" aria-label="Puerto" className={`${inputCls} w-24`} />
                </div>
                <input type="password" value={token} onChange={(e) => setToken(e.target.value)} placeholder="Bearer token (opcional)" aria-label="Bearer token" autoComplete="off" className={inputCls} />
              </>
            )}
            <button type="submit" className="press border-2 border-foreground bg-neon px-3 py-1.5 text-xs font-bold text-accent-foreground">
              + GUARDAR PERFIL
            </button>
          </form>
        </Section>
      )}

      {/* ===== (3) DEFAULTS DE BÚSQUEDA ===== */}
      <Section title="Defaults de búsqueda">
        <div className="flex flex-wrap items-end gap-3">
          <label className="flex flex-col gap-1">
            <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">top_k</span>
            <input
              type="number"
              min={1}
              value={prefs.topK ?? 8}
              onChange={(e) => {
                const topK = Math.max(1, Number(e.target.value) || 8);
                connectionPrefs.set({ topK });
                sync();
              }}
              className={`${inputCls} w-24`}
            />
          </label>
          <label className="flex flex-col gap-1">
            <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">modo</span>
            <select
              value={prefs.mode ?? "hybrid"}
              onChange={(e) => {
                connectionPrefs.set({ mode: e.target.value as "hybrid" | "vector" });
                sync();
              }}
              className={inputCls}
            >
              <option value="hybrid">Híbrido (BM25 · HNSW · RRF)</option>
              <option value="vector">Vectorial</option>
            </select>
          </label>
        </div>
        <p className="mt-2 font-tech text-[10px] text-muted-foreground">
          La búsqueda global del topbar usa estos defaults cuando no hay filtros activos.
        </p>
      </Section>

      {/* ===== (4) IDIOMA ===== */}
      <Section title="Idioma">
        <div className="flex gap-2">
          {(["es", "en"] as const).map((l) => (
            <button
              key={l}
              type="button"
              onClick={() => {
                connectionPrefs.set({ lang: l });
                sync();
              }}
              aria-pressed={prefs.lang === l}
              className={`press border-2 border-foreground px-4 py-1.5 text-sm font-bold ${prefs.lang === l ? "bg-neon text-accent-foreground" : "bg-background"}`}
            >
              {l === "es" ? "ESPAÑOL" : "ENGLISH"}
            </button>
          ))}
        </div>
      </Section>
    </div>
  );
}
