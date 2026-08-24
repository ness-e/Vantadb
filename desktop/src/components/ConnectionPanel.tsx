import { FormEvent, useState } from "react";
import { ConnectionInfo, HealthReport } from "../vanta";
import { connectionPrefs, ConnectionProfile } from "../store/connections";

interface Props {
  connections: ConnectionInfo[];
  activeId: string | null;
  health: HealthReport | null;
  healthStatus: "ok" | "warn" | "err" | "idle";
  busy: boolean;
  onConnectNative: (path: string) => Promise<string | null>;
  /** DESKTOP-31: conectar vía perfil guardado (nativo o server + Bearer). */
  onUseProfile?: (profile: ConnectionProfile) => Promise<void>;
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
  onUseProfile,
  onDisconnect,
  onActivate,
  onProbeHealth,
}: Props) {
  const [path, setPath] = useState("vantadb-local");
  // DESKTOP-31: perfiles guardados (Settings) → reconexión con un clic.
  // Se leen al montar; el panel se remonta al volver desde AJUSTES, así que
  // siempre refleja el store actual sin suscripción.
  const [profiles] = useState<ConnectionProfile[]>(() => connectionPrefs.get().profiles ?? []);
  const [profileId, setProfileId] = useState<string>("");

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    const id = await onConnectNative(path);
    if (id) setPath("vantadb-local-" + new Date().getTime().toString(36));
  }

  return (
    <section className="border-[3px] border-foreground bg-card p-4 shadow-ink">
      <div className="mb-3 flex items-center justify-between gap-2">
        <h2 className="m-0 font-tech text-xs uppercase tracking-widest">Conexiones</h2>
        <button
          type="button"
          onClick={onProbeHealth}
          title="Reintentar chequeo de salud"
          data-status={healthStatus}
          className={`cursor-pointer border-2 bg-transparent px-2.5 py-1 text-xs ${
            healthStatus === "idle"
              ? "text-muted-foreground"
              : healthStatus === "ok"
                ? "border-foreground bg-paper text-foreground"
                : "border-neon text-neon"
          }`}
        >
          {healthStatus === "idle"
            ? "—"
            : health
              ? `${health.backend} · ${health.latency_ms}ms`
              : "check"}
        </button>
      </div>

      <form className="flex gap-2" onSubmit={handleSubmit}>
        <input
          value={path}
          onChange={(e) => setPath(e.target.value)}
          placeholder="Ruta de la base de datos"
          aria-label="Ruta de la base de datos"
          className="min-w-0 flex-1 border-2 border-foreground bg-background px-2.5 py-1.5"
        />
        <button
          type="submit"
          disabled={busy}
          className="press cursor-pointer border-2 border-foreground bg-background px-2.5 py-1.5 text-sm disabled:cursor-default disabled:opacity-50"
        >
          {busy ? "Conectando…" : "Conectar nativo"}
        </button>
      </form>

      {onUseProfile && profiles.length > 0 && (
        <form
          className="mt-2 flex gap-2"
          onSubmit={(e) => {
            e.preventDefault();
            const p = profiles.find((x) => x.id === profileId) ?? profiles[0];
            void onUseProfile(p);
          }}
        >
          <select
            value={profileId || profiles[0].id}
            onChange={(e) => setProfileId(e.target.value)}
            aria-label="Perfil de conexión guardado"
            className="min-w-0 flex-1 cursor-pointer border-2 border-foreground bg-background px-2.5 py-1.5 text-sm"
          >
            {profiles.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name} · {p.kind}
              </option>
            ))}
          </select>
          <button
            type="submit"
            disabled={busy}
            title="Conectar vía perfil guardado (cierra la conexión activa y abre la del perfil)"
            className="press cursor-pointer border-2 border-foreground bg-background px-2.5 py-1.5 text-sm disabled:cursor-default disabled:opacity-50"
          >
            Conectar perfil
          </button>
        </form>
      )}

      <ul className="mb-0 mt-3 list-none p-0">
        {connections.length === 0 && (
          <li className="py-2 text-muted-foreground">Sin conexiones aún. Conectá un backend nativo.</li>
        )}
        {connections.map((c) => (
          <li
            key={c.id}
            className={`flex items-center gap-2 border-t-2 border-foreground px-1 py-2 ${c.id === activeId ? "bg-neon/10" : ""}`}
          >
            <button type="button" className="flex cursor-pointer items-center gap-2 border-none bg-transparent" onClick={() => onActivate(c.id)}>
              <span
                className={`inline-block h-2 w-2 rounded-full border border-foreground ${
                  c.status === "connected" ? "bg-neon" : c.status === "error" ? "bg-foreground" : "bg-paper"
                }`}
              />
              {c.name}
              {c.id === activeId && (
                <em className="not-italic">
                  <span className="border-2 border-foreground bg-neon px-2 py-px font-tech text-[10px] uppercase tracking-widest text-accent-foreground">
                    activa
                  </span>
                </em>
              )}
            </button>
            <span className="text-muted-foreground">{c.via}{c.description ? ` · ${c.description}` : ""}</span>
            <button
              type="button"
              className="ml-auto cursor-pointer border-none bg-transparent text-muted-foreground hover:text-neon"
              onClick={() => onDisconnect(c.id)}
            >
              desconectar
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}