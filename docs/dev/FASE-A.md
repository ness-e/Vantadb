---
title: Gate Fase A — kit del tester (stranger-test + checklist todo-SÍ)
kind: research
status: active
description: "Kit canónico de evaluación Stranger Test y Checklist de Salida Fase A, actualizado a v0.10.0 (2026-11-07)"
tags: [vantadb, fase-a, gate, early-access, stranger-test, dx10, v0100]
---

# Gate Fase A — kit del tester

> Kit fuente-única del gate que desbloquea el anuncio y lanzamiento público.
> Spec fuente: `docs/dev/research/archive/Investigacion-plan.md` Fase A (A.1/A.2/A.3) + fila `EXE-03` / `FASE-A` en `docs/dev/Backlog.md`.
> Anuncios y distribución pública pausados hasta certificación total de Fase A.

---

## Estado del Gate Fase A (2026-11-07 — v0.10.0)

1. **Pre-validación Automatizada (`DX-10` — 5 Instalaciones Limpias End-to-End):**
   - **Estado:** ✅ **COMPLETADA (2026-11-06)**.
   - **Evidencia Técnica:** Ejecución certificada mediante `scripts/verify_clean_installs.py` con reporte emitido en `docs/dev/reviews/2026-11-06-dx10-clean-installs-report.json`.
   - **Canales verificados en entornos limpios aislados:**
     1. **Python SDK:** `pip install vantadb-py` (`v0.10.0`, import canónico `import vantadb`, `Client`, `put`/`get`/`hybrid_search` en verde).
     2. **TypeScript/WASM SDK:** `npm install vantadb` (`v0.10.0`, bundle ESM/CJS, fallback OPFS/IndexedDB verificado).
     3. **Node.js Native SDK:** `npm install vantadb-node` (`v0.10.0`, NAPI-RS prebuilts x64/arm64 nativos sobre Fjall LSM).
     4. **Instaladores CLI:** `scripts/install.ps1` (Windows) y `scripts/install.sh` (Linux/macOS) desplegando binario `vanta-cli` con smoke test interactivo y no interactivo exitoso.
     5. **Servidor MCP:** Servidor `vantadb-mcp` ejecutado con perfil `--profile brain`, exponiendo 15 herramientas cognitivas estructuradas en `tools/list` (`memory_put`, `memory_get`, `memory_recall`, `memory_forget`, `scene_*`, `thread_*`, `axiom_*`, etc.).

2. **Evaluación Externa Humana (Stranger Test con 2–3 desarrolladores sin contexto previo):**
   - **Estado:** ⏸️ **DIFERIDA AL CIERRE FINAL PREVIO AL ANUNCIO PÚBLICO** por requerir usuarios humanos externos sin contexto del equipo.
   - **Fundamento Técnico:** El motor ha completado las Fases 1 a 5 y los 4 adelantos críticos de Fase 6 en el repo (modularización de `StorageEngine` en 3 coordinadores `Q8-A`, cifrado AES-256-GCM unificado, vacuum HNSW, `FastGate`, `group_commit` y `vantadb mcp init`). La compuerta final para autorizar el anuncio del release `v0.10.0` consiste en que 2–3 desarrolladores externos ejecuten el Quickstart sin asistencia y emitan veredicto GO.
   - **Preparación:** El protocolo de prueba, las instrucciones (§3) y las plantillas de evaluación (`usuario-01..05`, §1) quedan 100% listos y congelados para ejecutarse de forma inmediata como compuerta previa al anuncio en Hacker News, Reddit y redes.

---

## 0. Objetivo medible (A.3)

> **Que 2–3 desarrolladores externos (stranger test) y 5 evaluadores en total instalen en limpio, entiendan la propuesta de valor ("SQLite for AI Agents"), completen un flujo real en menos de 60 segundos y documenten cualquier fricción.**

- **Stranger-test:** 2–3 desarrolladores siguiendo **únicamente** el `README.md` o `docs/user/QUICKSTART.md`, con cero asistencia externa.
- **Flujo real:** Instalar en limpio → QUICKSTART (60s) → inserción de recuerdo (`put`) → búsqueda híbrida (`hybrid`) → export/inspección de atestación.
- **Salida requerida:** 5 fichas `usuario-01..05` cumplimentadas (§1) + veredicto GO/NO-GO de lanzamiento en `docs/dev/Backlog.md`.

---

## 1. Plantilla por persona — `usuario-0N` (A.1, una ficha por tester)

Copiar este bloque por evaluador:

```text
# Prueba de instalación: usuario-01

Fecha: / OS: / Runtime (Python/Node/Rust): / Tiempo hasta instalación: / Tiempo hasta primer resultado:
Bloqueos: / Preguntas: / Errores: / Cambios derivados: [ ]

1. ¿Instalaste sin ayuda? (comando exacto usado):
2. ¿Corrió el Quickstart de 60s (put/get/list)? (pega el output de `get`):
3. ¿Corrió la búsqueda híbrida (vector/texto/híbrida)? (pega las líneas de hits devueltas):
4. ¿Qué te impidió seguir o qué te confundió? (nada / cita el bloqueo):
5. Del 1 al 5, ¿entendiste qué es VantaDB y por qué es "SQLite for AI Agents"? (marca + 1 frase):
```

**Reglas de aplicación:** Sin ayuda del equipo core (solo documentación pública enlazada); cada mensaje de error o advertencia se captura literal; tiempos cronometrados objetivamente.

---

## 2. Checklist Fase A — todo SÍ o no se anuncia (A.2)

### Producto
- [ ] Flujo principal desde cero reproducido por desarrolladores externos sin asistencia (§1 fichas `usuario-01..03`).
- [x] Instala en limpio certificado en los 5 canales de distribución (`DX-10`: Python, TS/WASM, Node native, CLI scripts, MCP `--profile brain`).
- [ ] QUICKSTART ejecutado por desarrolladores externos (`get` confirma persistencia durable local).
- [x] 0 bugs críticos abiertos (suite core, integration tests y sanitizers en verde).
- [x] Versión idéntica en código, paquetes y documentación — verificado `2026-11-07`: `0.10.0` en `Cargo.toml`, `vantadb-python/pyproject.toml`, `vantadb-ts/package.json` y `vantadb-node/package.json`.

### Calidad
- [x] Unitarios en operaciones críticas + suite de integración en verde (`cargo nextest run -p vantadb --lib`).
- [x] CI en verde en limpio (Fast Gate, checks de linaje y compatibilidad).
- [x] Escaneo estricto de secretos en CI (`gitleaks` + `secretlint` integrados en workflow).
- [x] Licencia Apache-2.0 verificada + `docs/CHANGELOG.md` generado vía release-plz alineado a `v0.10.0`.

### Documentación
- [x] `README.md` y `README_ES.md` con paridad 1:1, quickstart de 60s, límites declarados, demo visual animada (VHS tape) sobre el fold y guía clara de reporte/contribución.
- [x] `docs/user/QUICKSTART.md` probado en limpio (ejemplos CLI, Python y TypeScript ejecutables).
- [x] Límites arquitectónicos explícitos (`Product Boundary` + `docs/user/operations/EXPERIMENTAL_FEATURES.md` purgado en Fase 2 `ACT-04`).
- [x] `SURFACE_MANIFEST.md` reconciliado automáticamente en CI contra registros públicos (npm, PyPI, crates.io).

### Comunidad
- [x] Canal de soporte técnico y bienvenida activo (Discord canónico `https://discord.gg/g8nqB3NtXt` verificado en READMEs).
- [x] Enlace de invitación verificado y permanente (`g8nqB3NtXt`).

### Comunicación
- [x] Kit de anuncios y notas de prensa listo en modo PREPARE-only (`docs/dev/strategy/LAUNCH_PLAYBOOK_AND_POSTS_KIT.md`), con publicación pausada hasta el gate final.
- [x] Cero funcionalidades no existentes prometidas (especificación estricta alineada con el código; purga `ACT-04`).
- [x] Cero métricas sin comando reproductor en el repositorio (Regla 11: benchmarks reproducibles en `BENCHMARKS.md` §24 vía `benchmarks/agent_memory_vs_sqlite_lancedb.py`).

---

## 3. Instrucciones para el tester (15 min)

1. **Instalación limpia (elegir un canal, sin clonar el repositorio):**
   - **Python:** `pip install vantadb-py`
   - **TypeScript/JavaScript:** `npm install vantadb` (o `npm install vantadb-node` para aceleración nativa Fjall)
   - **CLI Standalone:**
     - Linux/macOS: `curl -fsSL https://raw.githubusercontent.com/ness-e/VantaDB/main/scripts/install.sh | bash`
     - Windows PowerShell: `irm https://raw.githubusercontent.com/ness-e/VantaDB/main/scripts/install.ps1 | iex`
2. **Ejecución Quickstart:**
   - Seguir `README.md` §Quickstart en 60 segundos (insertar memoria, recuperar por clave, ejecutar búsqueda híbrida semántica).
3. **Reporte:**
   - Completar la plantilla §1 y enviarla al canal de soporte o registrar issue con prefijo `[fase-a]`.

---

## 4. README — veredicto de honestidad (Auditado 2026-11-07 — v0.10.0)

**Veredicto General:** ✅ **HONESTO Y AUDITADO A v0.10.0**

| Dimensión Auditada | Evidencia en Repositorio (`2026-11-07`) | Veredicto |
| :--- | :--- | :--- |
| **Cero features no existentes / purga v0.10.0** | `EXPERIMENTAL_FEATURES.md` purgado en Fase 2 (`ACT-04`). Features diferidas (multi-tenancy nube, RBAC corporativo remoto) declaradas explícitamente fuera del core embebido Apache-2.0. | ✅ Verificado |
| **Superficie de herramientas MCP** | El servidor MCP expone exactamente **15 herramientas cognitivas** en su perfil `--profile brain` (`tools/list`), verificado en auditoría de Paso 23 (`FIND-623` documenta la actualización del operating manual estático). | ✅ Verificado |
| **Límites de producto declarados** | `Product Boundary` declara claramente el posicionamiento como motor embebido in-process (WAL local + vector HNSW + BM25 léxico + fusión RRF + grafos de decisión L1-L3), sin pretender ser un cluster distribuido. | ✅ Verificado |
| **Paridad de versiones** | `v0.10.0` coherente y fijada en `Cargo.toml`, `pyproject.toml`, `package.json` (TS y Node). | ✅ Verificado |
| **Benchmarks reproducibles** | Métricas respaldadas por scripts ejecutables: `benchmarks/agent_memory_vs_sqlite_lancedb.py` documentado en `BENCHMARKS.md` §24, `evals/bench_sequential_niah.py` en §25 y suite de latencia local. | ✅ Verificado |
| **Paridad y Experiencia de Usuario** | `README.md` y `README_ES.md` cuentan con paridad estructural 1:1, Quickstart unificado de 60s, tabla de compatibilidad por IDE y demo animada VHS sobre el fold. | ✅ Verificado |

---

## 5. Handoff de Salida y Siguientes Pasos

1. **Pre-requisito Automatizado DX-10:** Superado y cerrado con reporte formal `docs/dev/reviews/2026-11-06-dx10-clean-installs-report.json`.
2. **Cierre de Fases 1 a 5 y Adelantos Fase 6:** Modularización de `StorageEngine`, cifrado AES-256-GCM unificado, vacuum HNSW, `vantadb mcp init`, Soak test 1,000 turnos y Sequential-NIAH completados.
3. **Convocatoria del Stranger Test:** Inmediatamente previo al anuncio público, se reclutan los 2–3 desarrolladores externos para completar las fichas `usuario-01..03`. Su aprobación unánime constituirá el GO final previo a la publicación masiva del release `v0.10.0`.
