Basado en el análisis exhaustivo del snapshot (`vanta_certification.json`, `ROADMAP.md`, `task.md`, `NEXT_5_TASKS.md`, `tests/`, `src/`, etc.), tu proyecto **VantaDB ha alcanzado un MVP técnico sólido**, pero aún tiene **brechas críticas de endurecimiento** antes de poder considerarlo un producto confiable para adopción externa controlada. El riesgo dominante ya no es la dispersión estratégica (que has controlado), sino **la coherencia operativa, la observabilidad fiable y la madurez de distribución**.

A continuación, detallo **qué falta** y **cuáles deben ser tus siguientes 5 acciones concretas**, priorizadas y ejecutables.

---

## 📌 Resumen de lo que todavía falta (evidencia del snapshot)

| Área | Estado actual | Brecha detectada | Evidencia en el código/documentación |
|------|---------------|------------------|--------------------------------------|
| **Telemetría de memoria** | Implementada pero con inconsistencias | Métricas de proceso no siempre reflejan la huella real del índice HNSW; algunos benchmarks muestran picos de RAM no explicados. `vanta_certification.json` reporta RSS de hasta 700 MB en SIFT, pero la documentación advierte que no es comparable. | `MEMORY_TELEMETRY.md`: “these numbers do **not** directly represent logical HNSW memory footprint”. `metrics.rs` usa `sysinfo` que depende del OS. |
| **Auditoría profunda del text index** | Existe `audit_text_index_deep` y `repair_text_index` | No está integrada en el CLI por defecto; falta documentación de cómo usarla en producción. Las pruebas de corrupción (`text_index_recovery.rs`) cubren solo casos básicos. | `src/sdk.rs` líneas 1416-1421 (deep audit). `vanta-cli audit-index --deep` no está documentado en `--help`. |
| **Distribución Python** | Wheels se construyen, TestPyPI funciona | **Producción PyPI no está activado**. No hay signing automático, ni release policy publicada. El workflow `python_wheels.yml` tiene `publish-pypi` pero solo se activa con tags; falta probarlo. | `PYTHON_RELEASE_POLICY.md` y `ROADMAP.md` (Phase 3 completada, pero sin publicación real). |
| **Modo read‑only** | Existe `VantaConfig::read_only` | Aún es posible que ciertas operaciones de auditoría o rebuild intenten escribir. El `audit_text_index` en read‑only no repara, pero no impide que otras llamadas (como `rebuild_index`) lancen error confuso. | `src/sdk.rs` – `open_with_config` respeta read_only, pero `audit_text_index` aún accede al engine de escritura en algunos paths (ver `ensure_text_index_current`). |
| **Documentación y claims** | README dice “híbrido implementado”, pero algunos documentos antiguos (IQL.md) aún sugieren lo contrario. | Inconsistencia entre `README.MD` (correcto) y `docs/api/IQL.md` (obsoleto). Además, el repositorio contiene archivos generados (`vantadb_data/`, `vanta_certification.json`) que no deben estar en Git. | `docs/api/IQL.md` – “Historical / Experimental Notice” pero aún visible. `git status` (imaginario) mostraría esos artefactos. |
| **Benchmark competitivo real** | SIFT1M usado solo como stress, no comparable (coseno vs L2) | No se puede hacer afirmaciones de rendimiento frente a otras bases vectoriales. Falta un corpus propio y métricas de recall/latencia en escenarios reales. | `competitive_bench.rs` admite explícitamente que no es comparable. `BENCHMARKS.md` solo muestra datos sintéticos. |

---

## 🚀 Tus próximas 5 acciones (ordenadas por impacto en el endurecimiento)

### 1. **Corregir y unificar la telemetría de memoria**

**Objetivo:** Que las métricas de RAM sean consistentes, interpretables y útiles para decisiones de producto.

**Acciones secundarias / desglose:**

- Separar en `VantaOperationalMetrics` tres familias:  
  - `process_rss_bytes` (actual)  
  - `hnsw_logical_bytes` (estimación interna desde `estimate_memory_bytes`)  
  - `mmap_resident_bytes` (si es posible, mediante `mincore` en Linux)  
- Añadir una función `debug_memory_breakdown()` que devuelva un JSON con estos tres valores.  
- Modificar el test `memory_telemetry.rs` para que compare proceso vs lógico en escenarios controlados.  
- Documentar en `MEMORY_TELEMETRY.md` que **solo la métrica lógica** puede usarse para comparativas de eficiencia.

**Archivos involucrados:**

- `src/metrics.rs` – añadir nuevos contadores y la función de breakdown.  
- `src/sdk.rs` – exponer `debug_memory_breakdown()` (solo si `cfg(debug_assertions)`).  
- `src/index.rs` – la función `estimate_memory_bytes` ya existe, moverla a un lugar accesible.  
- `tests/memory_telemetry.rs` – ampliar casos.  
- `docs/operations/MEMORY_TELEMETRY.md` – actualizar.

**Relaciones con otras áreas:**

- La nueva telemetría afectará a la documentación de `operational_metrics()` y a las expectativas de los usuarios del SDK.  
- Si se expone públicamente, debe ser bajo feature flag.

**Revisiones necesarias después del cambio:**

- Ejecutar `cargo test --test memory_telemetry -- --nocapture` y verificar que la suma de componentes lógicos no supera el RSS en más de un 20% (margen por page cache).  
- Actualizar el `vanta_certification.json` con las nuevas métricas.

---

### 2. **Completar la auditoría y reparación profunda del text index (modo CLI + read‑only seguro)**

**Objetivo:** Que un operador pueda diagnosticar y reparar el índice textual sin riesgo de corrupción adicional.

**Acciones secundarias / desglose:**

- Añadir flag `--deep` a `vanta-cli audit-index` que invoque `audit_text_index_deep`.  
- Implementar `vanta-cli repair-text-index` (ya existe en `src/bin/vanta-cli.rs` como comando `repair-text-index` – verificar que funcione).  
- Asegurar que `audit_text_index_deep` nunca intente escribir, incluso si el motor se abrió en modo lectura‑escritura (actualmente no es así – revisar).  
- Añadir un test que corrompa el índice de formas variadas (TF, posiciones, DF, longitudes) y verifique que `repair-text-index` lo corrige y que `audit --deep` lo detecta.

**Archivos involucrados:**

- `src/bin/vanta-cli.rs` – añadir opción `--deep`.  
- `src/sdk.rs` – exponer `audit_text_index_deep` (ya existe) y quizás `repair_text_index`.  
- `tests/text_index_recovery.rs` – ampliar con casos de corrupción múltiple.  
- `docs/operations/TEXT_INDEX_DESIGN.md` – documentar el flujo de auditoría profunda.

**Relaciones:**

- Depende de que el modo `read_only` sea estricto (ver siguiente acción).  
- La reparación debe actualizar las métricas de `operational_metrics()`.

**Revisiones necesarias:**

- Ejecutar `cargo test --test text_index_recovery -- --nocapture` y confirmar que todas las aserciones pasan.  
- Probar manualmente: crear una base, corromper con scripts `debug_corrupt_*_for_tests`, luego `vanta-cli repair-text-index` y verificar con `audit --deep --json`.

---

### 3. **Endurecer el modo read‑only para evitar cualquier escritura inesperada**

**Objetivo:** Garantizar que abrir la base de datos con `read_only=true` no modifique ningún archivo, ni siquiera para reparar índices derivados.

**Acciones secundarias / desglose:**

- Modificar `StorageEngine::open_with_config` para que cuando `read_only` es true, **no** llame a `ensure_derived_indexes_current()` ni a `ensure_text_index_current()`.  
- Cambiar `VantaEmbedded::open_with_config` para que, si `read_only`, no ejecute `ensure_derived_indexes_current` y `ensure_text_index_current`.  
- Añadir una verificación en `VantaEmbedded::search` para que, si el text index está en estado incompatible, devuelva un error claro en lugar de intentar reparar (actualmente `ensure_text_index_query_ready` lanza un error que es correcto, pero asegurar que no se escriba).  
- Crear un test que abra en read‑only, intente `rebuild_index`, y falle con mensaje adecuado.

**Archivos involucrados:**

- `src/storage.rs` – constructor `open_with_config`.  
- `src/sdk.rs` – `open_with_config` y `ensure_text_index_current`.  
- `tests/memory_api.rs` – añadir test `read_only_does_not_repair`.

**Relaciones:**

- Afecta a cualquier operación que hoy asuma que puede reparar índices al abrir.  
- Debe coordinarse con el punto 2 (auditoría profunda no debe escribir).

**Revisiones necesarias:**

- Ejecutar todo el test suite con `VANTA_READ_ONLY=1` (simulado) y asegurar que no haya escrituras.  
- Revisar los logs para confirmar que no aparecen mensajes de “rebuilding index”.

---

### 4. **Publicar en PyPI producción con firma Sigstore (release candidate)**

**Objetivo:** Tener un canal de distribución oficial para el SDK de Python, con verificación de integridad.

**Acciones secundarias / desglose:**

- Configurar el secret `PYPI_API_TOKEN` en GitHub Actions (requiere permisos de administrador).  
- En `.github/workflows/python_wheels.yml`, el job `publish-pypi` ya está preparado; solo falta activarlo en un tag real.  
- Crear un tag `v0.1.2-rc.1` (siguiendo PEP‑440) y verificar que el workflow se dispara y publica en TestPyPI primero (cambiando `repository` a `testpypi` temporalmente).  
- Después de validar, cambiar a PyPI producción y lanzar `v0.1.2`.  
- Documentar en `PYTHON_RELEASE_POLICY.md` el proceso completo y cómo los usuarios verifican las firmas.

**Archivos involucrados:**

- `.github/workflows/python_wheels.yml` – ajustar el target de publicación.  
- `vantadb-python/pyproject.toml` – asegurar que la versión coincide.  
- `docs/operations/PYTHON_RELEASE_POLICY.md` – completar la sección de producción.  
- `README.MD` – actualizar el enlace de instalación.

**Relaciones:**

- Depende de que el core esté estable (los puntos anteriores deben estar resueltos antes de lanzar a PyPI).  
- La publicación debería incluir solo el SDK Python, no el binario del servidor.

**Revisiones necesarias:**

- Ejecutar el workflow manualmente con `workflow_dispatch` en una rama de prueba.  
- Verificar que las wheels se suben y que `pip install vantadb-py` funciona en Linux, macOS y Windows.

---

### 5. **Alinear la documentación y limpiar el repositorio de artefactos generados**

**Objetivo:** Que el repositorio refleje fielmente el estado del producto y no contenga archivos que puedan confundir a nuevos contribuyentes.

**Acciones secundarias / desglose:**

- Eliminar del control de versiones `vantadb_data/` (si está trackeado) y `vanta_certification.json` raíz.  
- Añadir esas rutas al `.gitignore` (ya lo están en `.gitignore` líneas 6–8, pero verificar que no haya excepciones).  
- Actualizar `docs/api/IQL.md` para moverlo a `docs/experimental/` o añadir un aviso más prominente de que es histórico y no soportado.  
- Revisar `README.MD` para que mencione explícitamente que Hybrid Search está implementado (actualmente lo hace, pero confirmar que no haya contradicciones con otros docs).  
- Ejecutar `cargo fmt --check` y `cargo clippy --all-features -- -D warnings` y corregir los errores reportados en la auditoría.

**Archivos involucrados:**

- `.gitignore` – confirmar entradas.  
- `docs/api/IQL.md` – mover o renombrar.  
- `README.MD`, `docs/operations/ROADMAP.md` – sincronizar claims.  
- `src/sdk.rs`, `benches/hybrid_queries.rs` – formatear.

**Relaciones:**

- Ninguna técnica profunda, pero mejora la experiencia del desarrollador.

**Revisiones necesarias:**

- `git status --ignored` debe mostrar que `vantadb_data/` está ignorado.  
- `cargo doc --open` y navegar por la documentación para asegurar coherencia.

---

## 🧭 Resumen ejecutivo de prioridades

| Prioridad | Acción | Plazo estimado (días) | Archivos clave |
|-----------|--------|----------------------|----------------|
| 1 | Telemetría de memoria fiable | 2‑3 | `metrics.rs`, `index.rs`, `sdk.rs`, `MEMORY_TELEMETRY.md` |
| 2 | Auditoría profunda + CLI repair | 2 | `vanta-cli.rs`, `sdk.rs`, `text_index_recovery.rs` |
| 3 | Read‑only estricto | 1 | `storage.rs`, `sdk.rs` |
| 4 | PyPI producción | 1 (una vez los anteriores estén verdes) | `.github/workflows/python_wheels.yml`, `PYTHON_RELEASE_POLICY.md` |
| 5 | Limpieza repo y docs | 1 | `.gitignore`, `IQL.md`, `README.MD` |

**Nota final:** Una vez completadas estas 5 acciones, tu proyecto estará en condiciones de declarar el **MVP endurecido** y podrás avanzar con confianza a la **Fase 4: Search Quality v2** (stemming, stopwords, ranking explicable, etc.) y a la **evaluación competitiva real** (Euclidean distance). Sin embargo, no subestimes la importancia de la telemetría y la robustez del modo read‑only – son la base para cualquier afirmación de fiabilidad.
