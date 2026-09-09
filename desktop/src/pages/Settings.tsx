// DESKTOP-31: superficie AJUSTES — perfiles de conexión multi-perfil
// (path nativo / URL server con Bearer token), defaults de búsqueda e idioma.
// Persistencia vía connectionPrefs (localStorage inyectable, patrón DESKTOP-23);
// la conexión real pasa por actions.connectNativePath/connectServerCfg del hook
// useConnectionState — sin comandos Tauri nuevos (el transporte Bearer ya vive
// en Rust: ServerClientConfig.token).
// DESKTOP-40 (slice 1): strings vía tt()/tp() (desktop/src/i18n, catálogo
// propio ES/EN mínimo); idioma fuente = connectionPrefs.lang.
import { FormEvent, useState } from "react";
import { connectionPrefs, ConnectionProfile, profileTarget } from "../store/connections";
import { EMBED_MODELS, EmbedModelId, embedPrefs } from "../store/embed-prefs";
import { tp, tt, type DesktopLang } from "../i18n";

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
    <section className="border-4 border-foreground bg-card p-5 shadow-ink">
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
  const lang: DesktopLang = prefs.lang ?? "es";

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
    onNotice(tp(lang, "settings.profileSaved", 'Perfil "{name}" guardado.', { name: name.trim() }));
  }

  async function connectProfile(p: ConnectionProfile) {
    const id =
      p.kind === "native"
        ? await onConnectNative(p.path ?? "")
        : await onConnectServer(p.url ?? "", p.port ?? 8080, p.token ?? "");
    if (id) {
      connectionPrefs.set({ activeProfileId: p.id });
      sync();
      onNotice(tp(lang, "settings.connectedVia", 'Conectado vía perfil "{name}".', { name: p.name }));
    }
  }

  function removeProfile(p: ConnectionProfile) {
    connectionPrefs.removeProfile(p.id);
    sync();
    onNotice(tp(lang, "settings.profileRemoved", 'Perfil "{name}" eliminado.', { name: p.name }));
  }

  const profiles = prefs.profiles ?? [];

  return (
    <div className="mx-auto max-w-3xl space-y-5 p-6">
      {/* ===== (1+2) PERFILES DE CONEXIÓN + AUTH BEARER ===== */}
      {!embedded && (
        <Section title={tt(lang, "settings.connections", "Conexiones guardadas")}>
          {profiles.length === 0 ? (
            <p className="font-tech text-[11px] text-muted-foreground">
              {tt(lang, "settings.noProfiles", "Sin perfiles — guardá uno abajo para reconectar con un clic.")}
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
                      {tt(lang, "settings.connect", "conectar")}
                    </button>
                    <button type="button" onClick={() => removeProfile(p)} aria-label={tp(lang, "settings.removeProfile", "Eliminar {name}", { name: p.name })} className="press flex h-7 w-7 shrink-0 items-center justify-center border-2 border-foreground bg-background text-[10px]">
                      ✕
                    </button>
                  </li>
                );
              })}
            </ul>
          )}

          <form onSubmit={handleAddProfile} className="mt-4 space-y-2 border-t-2 border-dashed border-muted-foreground pt-4">
            <div className="flex gap-2">
              <input value={name} onChange={(e) => setName(e.target.value)} placeholder={tt(lang, "settings.profileName", "Nombre del perfil")} aria-label={tt(lang, "settings.profileName", "Nombre del perfil")} className={inputCls} />
              <select value={kind} onChange={(e) => setKind(e.target.value as ConnectionProfile["kind"])} aria-label={tt(lang, "settings.connType", "Tipo de conexión")} className={inputCls}>
                <option value="server">{tt(lang, "settings.server", "Server remoto")}</option>
                <option value="native">{tt(lang, "settings.native", "Nativo (path)")}</option>
              </select>
            </div>
            {kind === "native" ? (
              <input value={path} onChange={(e) => setPath(e.target.value)} placeholder={tt(lang, "settings.nativePath", "Ruta de base de datos")} aria-label={tt(lang, "settings.nativePath", "Ruta nativa")} className={inputCls} />
            ) : (
              <>
                <div className="flex gap-2">
                  <input value={url} onChange={(e) => setUrl(e.target.value)} placeholder="http://host" aria-label={tt(lang, "settings.serverUrl", "URL del servidor")} className={`${inputCls} min-w-0 flex-1`} />
                  <input type="number" value={port} onChange={(e) => setPort(Number(e.target.value) || 0)} placeholder={tt(lang, "settings.port", "Puerto")} aria-label={tt(lang, "settings.port", "Puerto")} className={`${inputCls} w-24`} />
                </div>
                <input type="password" value={token} onChange={(e) => setToken(e.target.value)} placeholder={tt(lang, "settings.bearer", "Bearer token (opcional)")} aria-label={tt(lang, "settings.bearer", "Bearer token")} autoComplete="off" className={inputCls} />
              </>
            )}
            <button type="submit" className="press border-2 border-foreground bg-neon px-3 py-1.5 text-xs font-bold text-accent-foreground">
              {tt(lang, "settings.saveProfile", "+ GUARDAR PERFIL")}
            </button>
          </form>
        </Section>
      )}

      {/* ===== (3) DEFAULTS DE BÚSQUEDA ===== */}
      <Section title={tt(lang, "settings.searchDefaults", "Defaults de búsqueda")}>
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
            <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">{tt(lang, "settings.mode", "modo")}</span>
            <select
              value={prefs.mode ?? "hybrid"}
              onChange={(e) => {
                connectionPrefs.set({ mode: e.target.value as "hybrid" | "vector" });
                sync();
              }}
              className={inputCls}
            >
              <option value="hybrid">{tt(lang, "settings.modeHybrid", "Híbrido (BM25 · HNSW · RRF)")}</option>
              <option value="vector">{tt(lang, "settings.modeVector", "Vectorial")}</option>
            </select>
          </label>
        </div>
        <p className="mt-2 font-tech text-[10px] text-muted-foreground">
          {tt(lang, "settings.searchHint", "La búsqueda global del topbar usa estos defaults cuando no hay filtros activos.")}
        </p>
      </Section>

      {/* ===== (4) IDIOMA ===== */}
      <Section title={tt(lang, "settings.language", "Idioma")}>
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

      {/* ===== (5) MODELO DE EMBEDDING (DESKTOP-EMBED-01) ===== */}
      <Section title={tt(lang, "settings.embedTitle", "Modelo de embedding (local)")}>
        <div className="flex flex-col gap-2">
          <label className="flex flex-col gap-1">
            <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">{tt(lang, "settings.embedModel", "modelo")}</span>
            <select
              value={embedPrefs.get().model}
              onChange={(e) => {
                embedPrefs.set({ model: e.target.value as EmbedModelId });
                sync();
              }}
              aria-label={tt(lang, "settings.embedTitle", "Modelo de embedding (local)")}
              className={inputCls}
            >
              {EMBED_MODELS.map((m) => (
                <option key={m.id} value={m.id}>{m.label}</option>
              ))}
            </select>
          </label>
          <p className="font-tech text-[10px] text-muted-foreground">
            {tt(
              lang,
              "settings.embedHint",
              "Modelo ONNX usado por vanta_embed_text. El default (multilingual-e5-small) coincide con embeddings/manifest.json. Cambialo solo si el modelo esta descargado via python embeddings/download.py --only <id>. Si el backend no expone embed-local, la seleccion se ignora (vector dummy).",
            )}
          </p>
        </div>
      </Section>
    </div>
  );
}
