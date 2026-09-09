// DESKTOP-40 (slice 1): catálogo ES/EN mínimo — solo claves de Settings.
// El catálogo web (web/src/lib/dictionaries.ts, ~1.2k claves, Next "use client")
// no es portable a Tauri; slice 2 (DEFER) extiende al resto de la UI.
export type DesktopLang = "es" | "en";

const es = {
  "settings.connections": "Conexiones guardadas",
  "settings.noProfiles": "Sin perfiles — guardá uno abajo para reconectar con un clic.",
  "settings.connect": "conectar",
  "settings.removeProfile": "Eliminar {name}",
  "settings.profileName": "Nombre del perfil",
  "settings.connType": "Tipo de conexión",
  "settings.server": "Server remoto",
  "settings.native": "Nativo (path)",
  "settings.nativePath": "Ruta de base de datos",
  "settings.serverUrl": "URL del servidor",
  "settings.port": "Puerto",
  "settings.bearer": "Bearer token (opcional)",
  "settings.saveProfile": "+ GUARDAR PERFIL",
  "settings.profileSaved": 'Perfil "{name}" guardado.',
  "settings.connectedVia": 'Conectado vía perfil "{name}".',
  "settings.profileRemoved": 'Perfil "{name}" eliminado.',
  "settings.searchDefaults": "Defaults de búsqueda",
  "settings.mode": "modo",
  "settings.modeHybrid": "Híbrido (BM25 · HNSW · RRF)",
  "settings.modeVector": "Vectorial",
  "settings.searchHint": "La búsqueda global del topbar usa estos defaults cuando no hay filtros activos.",
  "settings.language": "Idioma",
  "settings.embedTitle": "Modelo de embedding (local)",
  "settings.embedModel": "modelo",
  "settings.embedHint":
    "Modelo ONNX usado por vanta_embed_text. El default (multilingual-e5-small) coincide con embeddings/manifest.json. Cambialo solo si el modelo esta descargado via python embeddings/download.py --only <id>. Si el backend no expone embed-local, la seleccion se ignora (vector dummy).",
} as const;

const en: Record<keyof typeof es, string> = {
  "settings.connections": "Saved connections",
  "settings.noProfiles": "No profiles — save one below to reconnect in one click.",
  "settings.connect": "connect",
  "settings.removeProfile": "Remove {name}",
  "settings.profileName": "Profile name",
  "settings.connType": "Connection type",
  "settings.server": "Remote server",
  "settings.native": "Native (path)",
  "settings.nativePath": "Database path",
  "settings.serverUrl": "Server URL",
  "settings.port": "Port",
  "settings.bearer": "Bearer token (optional)",
  "settings.saveProfile": "+ SAVE PROFILE",
  "settings.profileSaved": 'Profile "{name}" saved.',
  "settings.connectedVia": 'Connected via profile "{name}".',
  "settings.profileRemoved": 'Profile "{name}" removed.',
  "settings.searchDefaults": "Search defaults",
  "settings.mode": "mode",
  "settings.modeHybrid": "Hybrid (BM25 · HNSW · RRF)",
  "settings.modeVector": "Vector",
  "settings.searchHint": "The topbar global search uses these defaults when no filters are active.",
  "settings.language": "Language",
  "settings.embedTitle": "Embedding model (local)",
  "settings.embedModel": "model",
  "settings.embedHint":
    "ONNX model used by vanta_embed_text. The default (multilingual-e5-small) matches embeddings/manifest.json. Change it only if the model is downloaded via python embeddings/download.py --only <id>. If the backend does not expose embed-local, the selection is ignored (dummy vector).",
};

export const dictionaries: Record<DesktopLang, Record<string, string>> = { es, en };

/** t() con fallback hardcodeado cuando la clave falta (patrón web createTt). */
export function tt(lang: DesktopLang, key: string, fallback: string): string {
  return dictionaries[lang]?.[key] ?? fallback;
}

/** tt() con interpolación {param} (avisos con nombre de perfil). */
export function tp(
  lang: DesktopLang,
  key: string,
  fallback: string,
  params: Record<string, string>,
): string {
  let value = tt(lang, key, fallback);
  for (const [k, v] of Object.entries(params)) value = value.replace(`{${k}}`, v);
  return value;
}
