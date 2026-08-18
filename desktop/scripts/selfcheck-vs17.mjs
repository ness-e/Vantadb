// Self-check VS-17 (Favoritos/historial de búsqueda + Copy-as) — el desktop no
// tiene test runner, así que este script compila los módulos puros con tsc
// (patrón VS-15), mockea localStorage en memoria ANTES de importarlos y valida:
//   1. Roundtrip de favoritos: toggle ns/key → nueva instancia sobre el MISMO
//      storage → persiste; toggle off no toca favoritos de key independientes.
//   2. Roundtrip de historial: add/dedup no-consecutivo/cap N=10/clear → reload.
//   3. Shape de copy-as: recordToJson parseable con el record completo;
//      recordToMarkdown con encabezado + payload + metadata + versión.
//
// Uso: node scripts/selfcheck-vs17.mjs   (desde desktop/)
import { execSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const desktop = resolve(fileURLToPath(new URL("..", import.meta.url)));
const tscBin =
  process.platform === "win32"
    ? join(desktop, "node_modules", ".bin", "tsc.cmd")
    : join(desktop, "node_modules", ".bin", "tsc");

// 1. Mock de localStorage EN MEMORIA (antes de cualquier import de los stores).
function makeStorage() {
  const map = new Map();
  return {
    getItem: (k) => (map.has(k) ? map.get(k) : null),
    setItem: (k, v) => map.set(k, String(v)),
    removeItem: (k) => map.delete(k),
    clear: () => map.clear(),
    key: (i) => [...map.keys()][i] ?? null,
    get length() {
      return map.size;
    },
  };
}
globalThis.localStorage = makeStorage();

const tmp = mkdtempSync(join(tmpdir(), "vs17-selfcheck-"));
try {
  // 2. Compilar los 3 módulos puros a CommonJS (copy-as solo importa tipos →
  //    se erasan; favorites/search-history sin imports runtime).
  const files = [
    join(desktop, "src", "store", "favorites.ts"),
    join(desktop, "src", "store", "search-history.ts"),
    join(desktop, "src", "components", "copy", "copy-as.ts"),
  ];
  execSync(
    `"${tscBin}" ${files.map((f) => `"${f}"`).join(" ")} --module commonjs --target es2020 --outDir "${tmp}" --skipLibCheck`,
    { cwd: desktop, stdio: "pipe" },
  );

  const favMod = await import(pathToFileURL(join(tmp, "store", "favorites.js")).href);
  const histMod = await import(pathToFileURL(join(tmp, "store", "search-history.js")).href);
  const copyMod = await import(pathToFileURL(join(tmp, "components", "copy", "copy-as.js")).href);

  // 3. Roundtrip favoritos: toggle → nueva instancia sobre el MISMO storage → persiste.
  const s1 = makeStorage();
  const store = new favMod.FavoritesStore(s1);
  assert(!store.isFavorite("docs", null), "arranca sin favoritos");
  assert(store.toggle("docs", null) === true, "toggle ns → favorito");
  assert(store.isFavorite("docs", null), "isFavorite ns");
  assert(store.toggle("docs", "k1") === true, "toggle key → favorito");
  assert(store.isFavorite("docs", "k1"), "isFavorite key");
  assert(
    store.getFavorites().length === 2 && store.getFavorites()[0].key === "k1",
    "newest-first (key toggleado después va primero)",
  );
  const favReload = new favMod.FavoritesStore(s1); // misma storage → lee JSON persistido
  assert(
    favReload.isFavorite("docs", null) && favReload.isFavorite("docs", "k1"),
    "roundtrip: favoritos persisten tras reload",
  );
  assert(store.toggle("docs", null) === false, "toggle off ns");
  assert(
    !store.isFavorite("docs", null) && store.isFavorite("docs", "k1"),
    "toggle off del ns no toca el favorito de key (entradas independientes)",
  );
  assert(
    new favMod.FavoritesStore(s1).isFavorite("docs", "k1") && !new favMod.FavoritesStore(s1).isFavorite("docs", null),
    "toggle off persiste tras reload",
  );

  // 4. Roundtrip historial: add/dedup no-consecutivo/cap/clear → reload.
  const s2 = makeStorage();
  const h = new histMod.SearchHistory(s2);
  for (const q of ["alpha", "beta", "alpha", "gamma"]) h.add(q);
  assert(h.get()[0] === "gamma", "add ordena newest-first");
  assert(
    h.get().length === 3 && h.get()[1] === "alpha",
    "dedup no-consecutivo (alpha re-buscada va al frente, sin duplicado)",
  );
  for (let i = 0; i < 12; i++) h.add(`q${i}`);
  assert(h.get().length === 10, `cap N=10 (hay ${h.get().length})`);
  const hReload = new histMod.SearchHistory(s2);
  assert(hReload.get().length === 10 && hReload.get()[0] === "q11", "roundtrip historial tras reload");
  h.clear();
  assert(h.get().length === 0, "clear vacía");
  assert(new histMod.SearchHistory(s2).get().length === 0, "clear persiste tras reload");

  // 5. Copy-as: shape de JSON + markdown.
  const record = {
    id: "k1",
    namespace: "docs",
    text: "# Título\n\ncuerpo markdown",
    metadata: { kind: "note", tags: ["a", "b"] },
    version: 3,
    vector: [0.1, 0.2],
    updated_at_ms: 1752900000000,
  };
  const json = copyMod.recordToJson(record);
  const parsed = JSON.parse(json);
  assert(
    parsed.id === "k1" && parsed.version === 3 && parsed.metadata.kind === "note",
    "recordToJson: JSON parseable con el record completo (id/version/metadata)",
  );
  const md = copyMod.recordToMarkdown(record);
  assert(
    md.includes("# docs/k1") && md.includes("cuerpo markdown") && md.includes("| kind | note |") && md.includes("v3"),
    "recordToMarkdown: encabezado ns/key + payload + metadata + versión",
  );

  console.log("✅ self-check VS-17: roundtrip localStorage (favoritos + historial) + copy-as OK");
} finally {
  rmSync(tmp, { recursive: true, force: true });
}

function assert(cond, msg) {
  if (!cond) throw new Error("❌ self-check VS-17: " + msg);
}