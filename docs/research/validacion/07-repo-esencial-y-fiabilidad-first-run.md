# 07 — Archivos Esenciales del Repo y Contrato de Fiabilidad First-Run

**Fecha:** 2026-08-25 · **Agente:** G (auditoría release/producto) · **Alcance:** estado actual del repo vs checklist de lanzamiento público + definición del contrato "lo que NO debe fallar jamás" para un evaluador nuevo.

---

## 1. Inventario actual — community / CI / docs

| Ítem | Estado | Notas |
| :--- | :---: | :--- |
| `LICENSE` | ✓ | Apache 2.0 (verificado en cabecera del archivo) |
| `README.md` | ✓ | Alta calidad: banner GIF, badges CI/PyPI/npm/Discord/Colab, quickstart copy-paste, tabla Quick Links, boundary honesto del producto |
| `README_ES.md` | ✓ | Bilingüe ES/EN |
| `CONTRIBUTING.md` | ✓ | Existe; linkeado desde README |
| `SECURITY.md` | ✓ | Linkeado desde README |
| `CODE_OF_CONDUCT.md` | ✓ | En raíz |
| `SUPPORT.md` | ✓ | Bonus no exigido por checklist |
| `NOTICE` | ✓ | En raíz |
| `CITATION.cff` | ✗ **FALTA** | Relevante para adopción académica |
| `CHANGELOG.md` | ✓ | Stub raíz → `docs/CHANGELOG.md`; mantenido por release-plz (REVIEW-19) |
| `.github/CODEOWNERS` | ✓ | Presente |
| `.github/FUNDING.yml` | ✗ **FALTA** | Opcional pero barato |
| `.github/dependabot.yml` | ✓ | Presente |
| `.github/ISSUE_TEMPLATE/*` | ✓ | `bug_report.{md,yml}`, `feature_request.{md,yml}`, `config.yml`, `documentation.yml` (duplicidad md+yml: redundancia menor) |
| `.github/pull_request_template.md` | ✓ | Nombre lowercase válido |
| `.github/workflows/*` | ✓✓ | 20 workflows: CI Rust con matriz **ubuntu/windows/macos** (`ci-rust-10.yml`: jobs test-macos, windows-latest…), web (`ci-web-11.yml`, solo ubuntu — aceptable: WASM es platform-independent), examples smoke (`ci-examples-12.yml`), security CodeQL, fuzz, chaos, benches, releases (wheels/npm/binaries/sbom/adapters), docs gate |
| `.github/clabot-config.json` + CLAs | ✓ | CLA individual + corporativo |
| `examples/` funcionales | ✓ | 11 ejemplos Python (mem0, Semantic Kernel, DSPy, LangChain, Haystack, CrewAI, AutoGen, LangGraph, agent_memory), 4 Rust (`basic`, `concurrent`, `graphrag`, `hybrid`), notebook Colab, demo end-to-end |
| `docs/QUICKSTART.md` | ✓ | Excelente: venv por OS, 3 rutas de instalación, ejemplo Python verificado contra API, embeddings opcionales (Ollama/OpenAI), TS SDK, CLI audit, TTFT medido (Python ~6s, TS ~2s), límites de IDs u128 documentados |

**Higiene extra verificada:** `.env.local`, `.env.tokens`, `vector_index.bin` están gitignored (no trackeados) — sin fuga de secretos ni artefactos basura en el repo.

---

## 2. Checklist priorizada

### P0 — Obligatorio antes de lanzar

| Ítem | Estado | Acción / plantilla sugerida |
| :--- | :---: | :--- |
| LICENSE claro | ✓ | Nada pendiente |
| README con quickstart copy-paste | ✓ | Mantener sincronizado con cada release |
| Ejemplos que corren tal cual | ✓ | Ya cubiertos por `ci-examples-12.yml`; extender matriz a Windows/macOS (ver P1) |
| CI con tests en PR, matriz 3 OS | ✓ | Cubierto por `ci-rust-10.yml` (ubuntu/win/mac) |
| Firmado de binarios Windows | ✗ | SmartScreen bloquea primera impresión en Windows (el propio README lo advierte). Acción mínima: certificado de firma de código (OV) o al menos publicar SHA256 checksums + guía "bypass seguro" visible en Releases. Plantilla checksums: `Get-FileHash vantadb-server.exe -Algorithm SHA256` publicado junto al asset |

### P1 — Fuerte

| Ítem | Estado | Acción |
| :--- | :---: | :--- |
| CITATION.cff | ✗ | Plantilla: `cff-version: 1.2.0` + `title: VantaDB` + `authors` + `version` + `date-released` + `license: Apache-2.0` + `repository-code`. GitHub lo renderiza en "Cite this repository" |
| Smoke de ejemplos en Windows/macOS | ✗ | `ci-examples-12.yml` corre solo en ubuntu. Agregar `strategy.matrix.os: [ubuntu-latest, windows-latest, macos-latest]` al job de ejemplos Python |
| Badges de cobertura/tests count | ✗ | Un badge de coverage (codecov o similar) refuerza confianza |
| FUNDING.yml | ✗ | Plantilla trivial: `github: [ness-e]` (o ko-fi/opencollective) |
| Duplicidad ISSUE_TEMPLATE md+yml | ⚠ | Consolidar a forms YAML puros; los `.md` legacy compiten en el selector |

### P2 — Nice to have

| Ítem | Estado | Acción |
| :--- | :---: | :--- |
| Demo GIF animado del CLI | ✓ parcial | Banner existe; un GIF de 10s del flujo `put→search` sería más persuasivo |
| ROADMAP visible | ✗ | Link a issues milestone o `docs/ROADMAP.md` |
| Snippets de integraciones auto-contenidos | ⚠ | Los snippets de README (Mem0/SK/DSPy) usan clases definidas dentro de los archivos de ejemplo sin mostrar el import/constructor completo; añadir la línea de construcción de la clase para copy-paste real |
| `test-runner.mjs` (TS interno) desactualizado | ⚠ | Usa `await` sobre métodos sync y `hits[0].score` cuando el tipo actual expone `distance` — migrarlo a Vitest o alinear campos |

---

## 3. Hallazgos de consistencia docs ↔ API

Verificado con `codegraph_explore` contra el código real (PyO3 `vantadb-python/src/lib.rs`, TS `vantadb-ts/src/vantadb.ts`):

### Ejemplo 1 — Quickstart Python (README líneas 96–126): ✅ COINCIDE AL 100%

| Llamada en doc | Código real | Veredicto |
| :--- | :--- | :---: |
| `vantadb.VantaDB("./vanta_data", memory_limit_bytes=512_000_000)` | `#[pyo3(signature = (db_path, memory_limit_bytes=None, read_only=false, backend=None))]` (lib.rs:387) | ✓ |
| `db.put(ns, key, payload, metadata={...}, vector=[...])` | `put(namespace, key, payload, metadata=None, vector=None, ttl_ms=None)` (lib.rs:875) | ✓ |
| `db.get_memory(ns, key)` | `get_memory(namespace, key) -> Option<Record>` (lib.rs:931) | ✓ |
| `db.search_memory(ns, query_vector=[...], top_k=5)` | `search_memory(namespace, query_vector, filters=None, text_query=None, top_k=10, …)` (lib.rs:1249); kwarg `query_vector` coincide | ✓ |
| `db.hardware_profile()` / `db.flush()` / `db.close()` | lib.rs:1840 / 1787 / 1904 | ✓ |

### Ejemplo 2 — Quickstart (triple búsqueda + TypeScript)

- **Python:** `search_memory(ns, [1.0,0.0,0.0], top_k=3)` posicional ✓; `text_query="durable memory"` kwarg ✓; `query_vector=[]` con solo texto ✓ (extract_vector tolera lista vacía).
- **TypeScript:** `VantaDB.create()` ✓ (vantadb.ts:200), `db.put({namespace,key,payload,metadata,vector})` ✓ (:417), `db.search({query_vector, top_k})` ✓ (:595), `hits[0].record.payload` ✓ (SearchHit = `{record, distance, explanation}`). **Matiz:** `create()` es **in-memory** (con `storage_path` emite warning); el quickstart no promete persistencia ahí — correcto pero conviene decirlo explícito para evitar confusión del evaluador.

### Deuda menor detectada

1. `distance_metric` desconocido solo loguea warning y cae a cosine silenciosamente (lib.rs:1267) — comportamiento suave no documentado en QUICKSTART.
2. Snippets de integración README no son standalone (falta línea de instanciación).
3. `test-runner.mjs` interno con drift de tipos (`await` sobre sync, `.score` vs `.distance`).

**No se encontró ningún ejemplo roto u obsoleto en README/QUICKSTART.**

---

## 4. Contrato de Fiabilidad First-Run — las 10 cosas que NO deben fallar

| # | Requisito | Estado actual (evidencia) | Riesgo | Smoke test mínimo |
| :-: | :--- | :--- | :--- | :--- |
| 1 | Instalación limpia Win/mac/Linux (pip/npm/cargo) | Wheels workflow (`release-wheels-60.yml`), npm (`release-npm-61.yml`), brew Formula, TTFT medido (~6s Py / ~2s TS); matriz CI 3 OS | **MEDIO**: binarios Windows sin firmar (SmartScreen) | VM limpia por OS → `pip install vantadb-py` / `npm i vantadb` / `cargo add` → registrar cualquier fallo |
| 2 | Import sin errores | Import canónico `import vantadb`; `verify_published_wheel.py` valida wheel publicado | BAJO: dualidad `vantadb` vs `vantadb_py` confunde novatos | `python -c "import vantadb; print(vantadb.VantaDB)"` tras instalar cada wheel |
| 3 | Abrir/crear DB en disco | `open_vantadb` con path o `:memory:`; backends fjall/RocksDB (lib.rs:202–223) | MEDIO: directorio read-only o antivirus bloqueando en Windows | Abrir path nuevo en `%TEMP%` → afirmar que el dir y archivos WAL existen |
| 4 | Put/get roundtrip correcto | Firmas verificadas; TS test-runner "put and get" ✓; suite Python (10 archivos de test); roundtrip cubre payload+metadata+vector | MEDIO: metadata `undefined` explícita rompe deserializador WASM (ya guardado en código, frágil) | put con metadata+vector → get → comparar campo a campo |
| 5 | Persistencia tras reiniciar proceso | WAL con `sync_data()` fsync (src/wal.rs:358); `close()` drena ops in-flight vía OpGate (lib.rs:1904–1918); `vantadb-node/tests/persistence.test.ts` existe | MEDIO: kill -9 sin close previo — recovery WAL está testeado en chaos/fuzz pero no en el camino feliz del SDK | Proceso A: put+flush+close. Proceso B: reabrir mismo path → get devuelve registro idéntico |
| 6 | Búsqueda híbrida devuelve resultados relevantes | BM25+HNSW vía RRF (`search_with_method`); QUICKSTART demuestra vector/texto/híbrido con seeds deterministas | MEDIO: namespace vacío devuelve `[]` sin error (correcto pero puede leerse como "no funciona"); mismatch de dimensionalidad poco claro para novatos | Sembrar 3 registros conocidos → 3 queries (vector/texto/hybrid) → afirmar que el key esperado rankea #1 |
| 7 | Primera consulta rápida (sin warmup de minutos) | TTFT medido y documentado (0.7s primera query Python); benchmarks 10K–100K publicados | MEDIO: reabrir una DB grande reconstruye/carga índice HNSW — sin dato publicado de cold-open | Cold-open de DB con 10K registros → primera query < 500 ms |
| 8 | Errores claros en inputs inválidos (sin panics feos) | `map_vanta_error` (Rust→Py), `wrapWasmError` con contexto ("connect"), OverflowError para IDs fuera de u128, mensaje "database is closing" en gate; 712 tests Rust incluyen fuzz | MEDIO-ALTO: un panic dentro de WASM aborta el proceso JS sin stack útil; enums desconocidos caen silenciosos en vez de fallar | Enviar: vector dims inconsistentes, key duplicado, backend inválido, operación tras `close()` → afirmar excepción tipada con mensaje accionable, nunca panic/segfault |
| 9 | Docs/ejemplos compilan tal cual | `ci-examples-12.yml` ejecuta los ejemplos end-to-end; `gate-docs-21.yml`; QUICKSTART fue corregido de samples rotos (lo documenta) | BAJO-MEDIO: solo Linux en CI de ejemplos; snippets README no standalone | Job CI que copia-pega cada snippet del README/QUICKSTART verbatim en un script temporal y lo ejecuta (en los 3 OS) |
| 10 | Desinstalación limpia | Diseño embedded: sin daemon, sin config global, datos solo en el dir que el usuario creó | BAJO: `./vanta_data` persiste tras uninstall (esperable, pero documentarlo evita sorpresa) | pip/npm uninstall → verificar ausencia de residuos en site-packages/node_modules y %APPDATA%; documentar que el data-dir es del usuario |

---

## 5. Conclusiones

1. **El repo está notablemente cerca del estándar de lanzamiento público.** De la checklist definitiva, solo faltan `CITATION.cff`, `FUNDING.yml` y firma de binarios Windows; todo lo demás (LICENSE Apache-2.0, README bilingüe de alta calidad, SECURITY/CONTRIBUTING/COC/CLA, plantillas de issues/PR, dependabot, CI con matriz 3 OS, 20 workflows incluyendo releases y seguridad, ejemplos ejecutados en CI) ya existe.
2. **La consistencia docs↔API es real, no cosmética:** ambos ejemplos auditados coinciden byte a nivel de firma con el binding PyO3 y el SDK TS vigentes. La deuda detectada es menor (snippets no standalone, drift interno en `test-runner.mjs`, fallback silencioso de `distance_metric`).
3. **El mayor riesgo de primera impresión es Windows:** binarios sin firmar (SmartScreen) + ejemplos smoke solo en Linux. Un evaluador en Windows es exactamente el perfil que descarta por un mal primer contacto.
4. **El contrato de fiabilidad tiene base sólida** (WAL con fsync, durability gate en close, suites Rust/Python/Node, chaos+fuzz en CI), pero los smoke tests #5, #6 y #7 (persistencia entre procesos, relevancia híbrida, cold-open) deberían formalizarse como una sola prueba E2E "first-run" ejecutable por cualquiera — sería además el mejor artefacto de marketing técnico del proyecto.

**Top-3 riesgos de primera impresión:** (1) SmartScreen en binarios Windows sin firmar; (2) evaluador que interpreta `[]` de un namespace vacío como "la búsqueda no funciona"; (3) cold-open de DBs grandes sin número publicado.
