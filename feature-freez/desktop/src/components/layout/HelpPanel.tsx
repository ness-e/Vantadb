import { useEffect, useState } from "react";
// DESKTOP-40-slice2: descripciones + chrome vía tt() (lang por prop, default store).
import { tt, type DesktopLang } from "../../i18n";
import { connectionPrefs } from "../../store/connections";

/**
 * HelpPanel (FIND-25) — first-run usage guide: shortcuts + surfaces, in user
 * language. Opens with the "?" key; Esc or click closes it.
 */

export type HelpTab = "general" | "proxy";

// [atajo, clave de descripción, fallback ES] — nombres de superficie/atajo son
// identidad de navegación y quedan literales; la descripción se traduce.
const SHORTCUTS: Array<[string, string, string]> = [
  ["⌘K / Ctrl+K", "help.scPalette", "Paleta de comandos"],
  ["Ctrl+Z", "help.scUndo", "Deshacer (papelera)"],
  ["Alt+T", "help.scTheme", "Cambiar tema claro/oscuro"],
  ["Ctrl+,", "help.scSettings", "Ir a AJUSTES"],
  ["F1", "help.scHelpGeneral", "Ayuda general (esta guía)"],
  ["F2", "help.scHelpProxy", "Ayuda proxy / ajustes (contextual)"],
  ["?", "help.scHelpThis", "Esta ayuda"],
];

const SURFACES: Array<[string, string, string]> = [
  ["RESUMEN", "help.sfResumen", "Vista general: KPIs, namespaces y actividad reciente"],
  ["MEMORIAS", "help.sfMemorias", "Explorar, crear y editar registros de memoria"],
  ["BÚSQUEDA", "help.sfBusqueda", "Búsqueda híbrida (vector + texto) sobre la selección"],
  ["ÍNDICES", "help.sfIndices", "Estado y mantenimiento de índices vectoriales/texto"],
  ["CONSOLIDAR", "help.sfConsolidar", "Detectar y fusionar duplicados"],
  ["IQL", "help.sfIql", "Consultas estructuradas al grafo/memoria"],
  ["ESPACIO", "help.sfEspacio", "Mapa visual de similitud entre vectores"],
  ["PAPELERA", "help.sfPapelera", "Registros borrados: restaurar o purgar"],
  ["ACTIVIDAD", "help.sfActividad", "Audit log: actividad y timeline de cambios"],
  ["MEMORIA", "help.sfMemoria", "Escenas con heat, persona, skills y generation log"],
  ["PROXY", "help.sfProxy", "Dashboard proxy local: TurnReports, sesiones, write-back, rate-limit"],
  ["AJUSTES", "help.sfAjustes", "Perfiles de conexión, defaults de búsqueda e idioma"],
];

export function HelpPanel({ onClose, initialTab = "general", lang = connectionPrefs.get().lang ?? "es" }: { onClose: () => void; initialTab?: HelpTab; lang?: DesktopLang }) {
  const [tab, setTab] = useState<HelpTab>(initialTab);
  useEffect(() => setTab(initialTab), [initialTab]);
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    // FIND-25: lightweight in-app usage guide (first-run + "?" reference).
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      onClick={onClose}
    >
      <div
        className="max-h-[80vh] w-[min(640px,92vw)] overflow-y-auto border-4 border-black bg-[var(--background)] p-6 shadow-[8px_8px_0_0_#000]"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-start justify-between">
          <h2 className="font-[family-name:var(--font-anton)] text-2xl uppercase tracking-wide text-[var(--foreground)]">
            {tt(lang, "help.title", "Guía rápida")}
          </h2>
          <button
            aria-label={tt(lang, "help.close", "Cerrar ayuda")}
            onClick={onClose}
            className="h-7 w-7 border border-black/30 text-[var(--foreground)] hover:bg-black/10"
          >
            ✕
          </button>
        </div>

        <div className="mt-4 flex gap-2" role="tablist" aria-label={tt(lang, "help.sections", "Secciones de ayuda")}>
          <button
            role="tab"
            aria-selected={tab === "general"}
            onClick={() => setTab("general")}
            className={`press border-2 border-foreground px-3 py-1 text-xs font-semibold ${tab === "general" ? "bg-foreground text-background" : "bg-background"}`}
          >
            {tt(lang, "help.tabGeneral", "General (F1)")}
          </button>
          <button
            role="tab"
            aria-selected={tab === "proxy"}
            onClick={() => setTab("proxy")}
            className={`press border-2 border-foreground px-3 py-1 text-xs font-semibold ${tab === "proxy" ? "bg-foreground text-background" : "bg-background"}`}
          >
            {tt(lang, "help.tabProxy", "Proxy / Ajustes (F2)")}
          </button>
        </div>

        {tab === "proxy" ? (
          <>
            <h3 className="mt-5 font-[family-name:var(--font-space-mono)] text-xs uppercase tracking-widest text-[var(--muted-foreground)]">
              {tt(lang, "help.proxyLocal", "Proxy local")}
            </h3>
            <ul className="mt-2 space-y-1.5 text-sm text-[var(--foreground)]">
              <li>• <span className="font-bold">{tt(lang, "help.trTurnsLabel", "TurnReports:")}</span> {tt(lang, "help.trTurns", "registro de turnos del upstream LLM (request/response + latencia)")}</li>
              <li>• <span className="font-bold">{tt(lang, "help.trSessionsLabel", "Sesiones activas:")}</span> {tt(lang, "help.trSessions", "conexiones proxy abiertas y su estado")}</li>
              <li>• <span className="font-bold">{tt(lang, "help.trWritebackLabel", "Cola write-back:")}</span> {tt(lang, "help.trWriteback", "escrituras pendientes hacia VantaDB")}</li>
              <li>• <span className="font-bold">{tt(lang, "help.trRatelimitLabel", "Rate-limit:")}</span> {tt(lang, "help.trRatelimit", "límites del upstream y backoff")}</li>
              <li className="text-xs text-[var(--muted-foreground)]">{tt(lang, "help.proxyHint", "Configurá la URL del proxy en AJUSTES → Proxy. F2 abre este tab contextual.")}</li>
            </ul>
            <h3 className="mt-5 font-[family-name:var(--font-space-mono)] text-xs uppercase tracking-widest text-[var(--muted-foreground)]">
              {tt(lang, "help.settingsH", "Ajustes")}
            </h3>
            <ul className="mt-2 space-y-1.5 text-sm text-[var(--foreground)]">
              <li>• {tt(lang, "help.setProfiles", "Perfiles de conexión (native / server + Bearer)")}</li>
              <li>• {tt(lang, "help.setDefaults", "Defaults de búsqueda (topK) y preferencias de workspace")}</li>
              <li>• {tt(lang, "help.setTheme", "Selector de tema claro/oscuro")}</li>
            </ul>
          </>
        ) : null}
        {tab === "general" && (
          <>
            <h3 className="mt-5 font-[family-name:var(--font-space-mono)] text-xs uppercase tracking-widest text-[var(--muted-foreground)]">
              {tt(lang, "help.shortcuts", "Atajos")}
            </h3>
            <ul className="mt-2 space-y-1.5">
              {SHORTCUTS.map(([k, dk, df]) => (
                <li key={k} className="flex items-center gap-3 text-sm text-[var(--foreground)]">
                  <kbd className="min-w-28 border border-black/40 px-2 py-0.5 text-center font-[family-name:var(--font-space-mono)] text-xs">
                    {k}
                  </kbd>
                  {tt(lang, dk, df)}
                </li>
              ))}
            </ul>

            <h3 className="mt-6 font-[family-name:var(--font-space-mono)] text-xs uppercase tracking-widest text-[var(--muted-foreground)]">
              {tt(lang, "help.surfaces", "Superficies")}
            </h3>
            <ul className="mt-2 space-y-1.5">
              {SURFACES.map(([name, dk, df]) => (
                <li key={name} className="text-sm text-[var(--foreground)]">
                  <span className="font-[family-name:var(--font-space-mono)] text-xs font-bold uppercase tracking-wide">
                    {name}
                  </span>{" "}
                  — {tt(lang, dk, df)}
                </li>
              ))}
            </ul>

            <p className="mt-6 border-t border-black/20 pt-3 text-xs text-[var(--muted-foreground)]">
              {tt(lang, "help.tip", "Tip: presioná ? en cualquier momento para volver a esta guía.")}
            </p>
          </>
        )}
      </div>
    </div>
  );
}
