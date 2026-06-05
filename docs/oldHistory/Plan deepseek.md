# Plan de Remediación, Hardening y Escalado de VantaDB

## Comité Técnico de Remediación (Simulado)

*Principal Engineers (Google, Meta), Staff Engineers (AWS, Microsoft), SREs (Apple, SpaceX), Security Engineers (Tesla, X), Product Strategists (Amazon)*

**Fecha:** 2026-05-18  
**Estado del proyecto:** MVP técnico parcialmente endurecido, riesgo crítico de deuda arquitectónica y operacional.  
**Objetivo:** Convertir VantaDB en un producto enterprise-grade, escalable, mantenible, seguro y competitivo en 12-18 meses.

---

## Resumen Ejecutivo del Plan

El plan se organiza en **10 áreas críticas** con **96 tareas detalladas**, priorizadas por impacto y urgencia. Se estima un esfuerzo total de **12-18 meses-hombre** distribuidos en fases. La **Fase 0 (Quick Wins)** debe completarse en 30 días para reducir riesgos inmediatos. La **Fase 1 (Endurecimiento del MVP)** toma 3 meses. La **Fase 2 (Escalabilidad)** 3-6 meses. La **Fase 3 (Enterprise Readiness)** 6-12 meses.

**Advertencia:** No seguir este plan implica garantía de fallos catastróficos en producción bajo carga, corrupción de datos, fuga de memoria, inestabilidad de concurrencia y commoditización del producto.

---

# 1. Arquitectura y Refactorización

| ID | Categoría | Problema detectado | Riesgo actual | Solución propuesta | Subtareas | Prioridad | Impacto | Complejidad | Dependencias | Riesgo de implementación | Validación | Métricas | Horizonte | Owner sugerido |
|----|-----------|-------------------|---------------|--------------------|-----------|----------|---------|-------------|--------------|--------------------------|------------|----------|-----------|----------------|
| ARQ-01 | Separación Servidor/Core | Acoplamiento entre `vantadb-server` y el core; dependencias de red en `src/engine.rs` | Fallos de red afectan al motor embebido; no se puede compilar como biblioteca aislada | Mover todo el código de red (axum, tokio net, reqwest) a `vantadb-server`; exponer solo SDK puro en `vantadb-core` | 1. Crear crate `vantadb-core` con `default-features = false`<br>2. Eliminar dependencias de red de `Cargo.toml` raíz<br>3. Refactorizar `src/sdk.rs` para no usar `tokio::net`<br>4. Mover `src/api/` y `src/mcp.rs` a `vantadb-server`<br>5. Actualizar `vantadb-server/src/main.rs` para usar el core vía FFI puro<br>6. Ajustar CLI para usar el core directamente | Critical | Alto | Alta | Alto (semanas) | Refactorización de `src/engine.rs` | Alto – puede romper integración actual | `cargo build --no-default-features` pasa; tests de integración del servidor siguen verdes | Separación completa de dependencias; tiempo de compilación del core -40% | Corto (1-2 meses) | Backend Lead |
| ARQ-02 | Planificador con AST/IR | `src/planner.rs` traduce directamente de parser a VM sin AST ni plan lógico/físico | No se pueden optimizar consultas complejas; difícil añadir nuevas características (agregaciones, subconsultas) | Implementar pipeline: Parser → AST → Plan Lógico → Optimizador → Plan Físico → Ejecutor | 1. Definir AST en `src/ast.rs` con tipos para `Select`, `Filter`, `Join`, etc.<br>2. Crear `LogicalPlan` con operadores relacionales (Scan, Filter, Project, Sort, Limit)<br>3. Implementar reglas de optimización (predicate pushdown, projection pushdown)<br>4. Generar `PhysicalPlan` con referencias a índices específicos (HNSW, texto, bitset)<br>5. Adaptar `Executor` para interpretar plan físico<br>6. Mantener `src/planner.rs` solo para routing RRF (simplificado) | High | Alto | Alta | Alto (1-2 meses) | Separación servidor/core (ARQ-01) | Alto – regresiones de rendimiento | Tests de regresión `tests/logic/executor.rs`; benchmarks de latencia no degradan | Latencia p99 de consultas híbridas no aumenta >10% | Corto (2-3 meses) | Senior Engineer |
| ARQ-03 | Eliminar características experimentales | Código muerto: `governance/`, `eval/`, `parser/lisp.rs`, `mcp.rs`, `llm.rs` | Aumenta superficie de ataque, coste de mantenimiento, fricción al compilar | Eliminar todo el código bajo `#[cfg(feature = "experimental")]` y `governance` | 1. Eliminar `src/governance/` completo<br>2. Eliminar `src/eval/` y `src/parser/lisp.rs`<br>3. Eliminar `src/api/mcp.rs` y `vantadb-server/src/mcp.rs`<br>4. Mover `src/llm.rs` a ejemplo separado fuera del core<br>5. Eliminar `examples/docker/`<br>6. Eliminar `benches/high_density.rs` (basura) | High | Medio | Baja | Ninguna | Bajo – solo eliminar | `cargo check --all-features` sigue pasando; tests no dependientes | Reducción de LOC en un 15-20% | Corto (1 semana) | Cualquier dev |
| ARQ-04 | Backend abstraction unificada | `StorageBackend` tiene métodos que no aplican a todos (`checkpoint` en InMemory, `compact` en Fjall) | Operaciones fallan silenciosamente o lanzan errores confusos | Refactorizar trait con métodos opcionales o extender con `BackendCapabilities` (ya existe pero no se usa consistentemente) | 1. Añadir métodos `fn supports_checkpoint(&self) -> bool` y `fn supports_compaction(&self) -> bool`<br>2. Cambiar `create_life_insurance` para verificar capability antes de llamar<br>3. Mover lógica de checkpoint a un trait separado `Checkpointable` | Medium | Medio | Baja | Ninguna | Bajo | Test `fjall_cold_copy_restore.rs` sigue pasando | No hay errores inesperados de checkpoint | Corto (1 semana) | Backend Engineer |
| ARQ-05 | Configuración unificada | Múltiples fuentes: `VantaConfig`, env vars, `HardwareCapabilities`, defaults | Inconsistencia, difícil de debuggear | Usar `figment` o `config` crate con jerarquía: defaults → archivo TOML → env vars → overrides programáticos | 1. Definir `VantaSettings` struct serializable<br>2. Cargar desde `vantadb.toml` + env vars prefijadas `VANTA_`<br>3. Eliminar defaults dispersos en el código<br>4. Inyectar configuración en lugar de leer globalmente | Medium | Medio | Media | Ninguna | Bajo | `cargo run -- --help` muestra configuración cargada | Un solo punto de verdad para todas las configuraciones | Corto (2 semanas) | DevOps/Backend |

---

# 2. Código y Calidad Técnica

| ID | Categoría | Problema detectado | Riesgo actual | Solución propuesta | Subtareas | Prioridad | Impacto | Complejidad | Dependencias | Riesgo de implementación | Validación | Métricas | Horizonte | Owner |
|----|-----------|-------------------|---------------|--------------------|-----------|----------|---------|-------------|--------------|--------------------------|------------|----------|-----------|--------|
| COD-01 | Concurrencia async/sync | Mezcla de `std::thread` bloqueante dentro de `tokio` runtime (ej. `WalWriter::sync`) | Bloqueo del reactor, degradación de rendimiento, deadlocks | Mover todas las operaciones de I/O síncronas a `tokio::task::spawn_blocking` o usar `tokio::fs` | 1. Identificar todas las llamadas `std::fs` y `std::io` en `storage.rs`, `wal.rs`, `backends/`<br>2. Reemplazar con `tokio::fs` donde sea posible<br>3. Usar `spawn_blocking` para `fsync`, `mmap`, `compress`<br>4. Configurar `tokio` con `multi_thread` y `worker_threads` adecuado | Critical | Alto | Media | Ninguna | Medio – cambios en rutas críticas | Pruebas de estrés con 1000 conexiones concurrentes | Latencia p99 no aumenta >5% bajo carga | Corto (2-3 semanas) | SRE/Backend |
| COD-02 | Serialización zero-copy | `bincode` deserializa todo en heap; costoso para grandes datasets | Alto consumo de CPU y memoria, cuello de botella | Reemplazar `bincode` por `rkyv` (zero-copy) o `postcard` (más compacto) | 1. Evaluar `rkyv` vs `postcard` vs `protobuf`<br>2. Migrar `UnifiedNode`, `WalRecord`, `DiskNodeHeader` a `rkyv`<br>3. Modificar `VantaFile` para leer estructuras zero-copy<br>4. Actualizar serialización de WAL | High | Alto | Alta | ARQ-01 (separación core) | Alto – cambio de formato rompe compatibilidad hacia atrás | Implementar migrador de versiones; tests de integridad de datos | Latencia de lectura de nodos -30% | Medio (1-2 meses) | Performance Engineer |
| COD-03 | Asignador de memoria | Usa `std::alloc` (por defecto) que fragmenta severamente bajo carga de vectores | OOM progresivo, degradación de rendimiento | Cambiar a `jemalloc` o `mimalloc` vía `#[global_allocator]` | 1. Agregar dependencia `jemallocator` o `mimalloc`<br>2. Configurar global allocator en `main` y en `lib.rs` para CDylib<br>3. Ajustar parámetros (dirty decay, background threads)<br>4. Verificar con pruebas de fragmentación | Critical | Alto | Baja | Ninguna | Bajo – solo añadir allocator | Prueba de larga duración (4h) con inserciones continuas; memoria RSS no crece indefinidamente | Estabilización de memoria después de 1h de escritura | Corto (1 día) | SRE |
| COD-04 | Linting y formato | `cargo fmt --check` falla; `clippy` tiene warnings en `src/python.rs` y `src/sdk.rs` | CI inconsistente, código difícil de leer | Ejecutar `cargo fmt` y `cargo clippy --fix`; añadir CI gate estricto | 1. Ejecutar `cargo fmt` en todo el workspace<br>2. Corregir warnings manualmente (o con `--fix`)<br>3. Añadir step en CI: `cargo clippy -- -D warnings`<br>4. Configurar `rustfmt.toml` con estilo estándar | High | Bajo | Baja | Ninguna | Bajo | CI pasa sin warnings | 0 warnings, 0 format issues | Corto (1 día) | QA/DevOps |
| COD-05 | Gestión de errores | Uso inconsistente de `VantaError`; a veces `unwrap()`, a veces `expect()` | Panics en producción, mala experiencia de usuario | Reemplazar `unwrap/expect` con `?` o manejo explícito; definir categorías de error | 1. Auditar todo el código base buscando `.unwrap()` y `.expect()`<br>2. Reemplazar en rutas críticas con `?` o `map_err`<br>3. Añadir contexto con `anyhow` o `thiserror` (ya existe)<br>4. Definir política: prohibir unwrap en producción | Medium | Medio | Media | Ninguna | Medio – puede introducir cambios de API | `cargo test` pasa; no hay panics en pruebas de estrés | 0 panics en 24h de ejecución continua | Corto (1-2 semanas) | Backend |

---

# 3. Testing y QA

| ID | Categoría | Problema detectado | Riesgo actual | Solución propuesta | Subtareas | Prioridad | Impacto | Complejidad | Dependencias | Riesgo de implementación | Validación | Métricas | Horizonte | Owner |
|----|-----------|-------------------|---------------|--------------------|-----------|----------|---------|-------------|--------------|--------------------------|------------|----------|-----------|--------|
| TST-01 | Chaos engineering | No hay pruebas de fallos de red, disco, OOM, particiones | El sistema fallará silenciosamente en producción | Implementar chaos testing con `chaos-mesh` o `toxiproxy` | 1. Configurar Kubernetes local con `kind`<br>2. Desplegar `chaos-mesh`<br>3. Crear experimentos: pérdida de paquetes, latencia, cierre de archivos, OOM<br>4. Verificar que WAL y recovery funcionan | High | Alto | Media | CI/CD, K8s | Medio – requiere entorno | Ejecutar suite de caos en CI nightly | 100% de experimentos pasan sin corrupción | Medio (1 mes) | SRE/QA |
| TST-02 | Fuzzing del WAL y VantaFile | Solo hay fuzzing del parser LISP (experimental) | Datos corruptos pueden causar panic o corrupción | Añadir fuzzing para `WalReader`, `VantaFile::read_header`, `DiskNodeHeader` | 1. Crear fuzz target `fuzz_wal_record` en `fuzz/`<br>2. Crear `fuzz_vanta_file_header`<br>3. Ejecutar con `cargo fuzz` por 1 hora en CI<br>4. Añadir seeds de casos límite | High | Alto | Media | COD-02 (rkyv) si se migra | Medio – puede encontrar bugs reales | No se encuentran crashes después de 1h de fuzzing | 0 crashes en fuzzing | Corto (1 semana) | Security/QA |
| TST-03 | Performance regression suite | No hay benchmarks automatizados en CI; solo manuales | Regresiones de rendimiento llegan a producción | Integrar `criterion` en CI con guarda de umbrales | 1. Configurar `criterion` con `baseline` y `comparison`<br>2. Ejecutar benchmarks en cada PR (solo muestreo pequeño)<br>3. Fallar CI si latencia aumenta >10% o throughput baja >10%<br>4. Guardar baseline en `vanta-benchmark-baseline.json` | High | Alto | Media | Ninguna | Medio – falsos positivos por ruido | Ajustar umbrales gradualmente | Benchmarks pasan en CI; dashboard de rendimiento | Corto (2 semanas) | Performance/DevOps |
| TST-04 | Cobertura de código mínima | Desconocida; probablemente baja (<50%) | Mucho código no testeado | Establecer cobertura mínima del 70% para core, 80% para módulos críticos | 1. Configurar `tarpaulin` o `grcov`<br>2. Ejecutar en CI y reportar<br>3. Identificar módulos sin cobertura<br>4. Priorizar escritura de tests para `storage.rs`, `wal.rs`, `index.rs` | Medium | Medio | Media | Ninguna | Bajo | `cargo tarpaulin --out Html` produce reporte | Cobertura ≥70% en 3 meses | Medio (2-3 meses) | QA/Backend |

---

# 4. Seguridad y Hardening

| ID | Categoría | Problema detectado | Riesgo actual | Solución propuesta | Subtareas | Prioridad | Impacto | Complejidad | Dependencias | Riesgo de implementación | Validación | Métricas | Horizonte | Owner |
|----|-----------|-------------------|---------------|--------------------|-----------|----------|---------|-------------|--------------|--------------------------|------------|----------|-----------|--------|
| SEC-01 | Autenticación en servidor | El servidor HTTP no tiene auth; cualquiera puede ejecutar queries | Exposición total de datos, riesgo de compliance | Implementar autenticación Bearer token + OIDC opcional | 1. Añadir middleware de autenticación en `vantadb-server`<br>2. Soporte para `Authorization: Bearer <token>`<br>3. Integración opcional con OIDC (Google, Okta)<br>4. Configurar tokens via `VANTA_AUTH_TOKEN` env | Critical | Alto | Media | ARQ-01 | Medio – cambios en API | Tests con token inválido devuelven 401 | 0 accesos no autorizados en logs | Corto (2 semanas) | Security/Backend |
| SEC-02 | TLS/mTLS | Comunicación en texto plano entre servidor y clientes (si se expone) | Interceptación de datos sensibles | Habilitar TLS 1.3 con certificados automáticos (Let's Encrypt) o mTLS para comunicación inter-nodo | 1. Añadir soporte para certificados en `axum` via `rustls`<br>2. Configurar via `VANTA_TLS_CERT`, `VANTA_TLS_KEY`<br>3. Opcional: mTLS con CA propia para clúster | High | Alto | Media | SEC-01 | Medio | Tests con `curl --cacert` verifican conexión segura | Todas las conexiones externas usan TLS | Corto (1-2 semanas) | Security/DevOps |
| SEC-03 | Rate limiting | Sin límite de peticiones; ataque DDoS trivial | Denegación de servicio | Implementar rate limiting por IP o token | 1. Usar `tower_governor` o similar<br>2. Configurar límites: 1000 req/min por IP<br>3. Exponer headers `X-RateLimit-*`<br>4. Log de violaciones | High | Medio | Baja | SEC-01 | Bajo | Prueba de carga con >límite devuelve 429 | Requests bloqueados <1% bajo carga normal | Corto (1 semana) | SRE |
| SEC-04 | Secret scanning | Posibles credenciales en logs o código (PyPI token, etc.) | Filtración de secrets | Implementar pre-commit hook con `gitleaks` o `detect-secrets` | 1. Instalar `gitleaks` en CI<br>2. Escanear cada PR<br>3. Bloquear si se detecta secret<br>4. Limpiar histórico con `BFG` si es necesario | High | Alto | Baja | Ninguna | Bajo | CI falla si hay secret hardcoded | 0 secrets en repositorio | Corto (1 día) | Security/DevOps |
| SEC-05 | SBOM y dependencias | Sin inventario de dependencias ni escaneo de vulnerabilidades | Riesgo de supply chain attack | Generar SBOM (SPDX/CycloneDX) y escanear con `cargo-audit` en CI | 1. Añadir step `cargo audit` en CI<br>2. Generar SBOM con `cargo sbom` o `cargo cyclonedx`<br>3. Subir a repositorio de artefactos<br>4. Configurar alertas de nuevas vulnerabilidades | Medium | Medio | Baja | Ninguna | Bajo | CI pasa con `cargo audit` sin vulnerabilidades críticas | Tiempo de parcheo de vulnerabilidades <48h | Corto (1 semana) | Security |

---

# 5. DevOps, Infraestructura y SRE

| ID | Categoría | Problema detectado | Riesgo actual | Solución propuesta | Subtareas | Prioridad | Impacto | Complejidad | Dependencias | Riesgo de implementación | Validación | Métricas | Horizonte | Owner |
|----|-----------|-------------------|---------------|--------------------|-----------|----------|---------|-------------|--------------|--------------------------|------------|----------|-----------|--------|
| OPS-01 | Observabilidad (tracing) | Solo hay métricas Prometheus; no hay tracing distribuido | Imposible depurar problemas en microservicios o consultas complejas | Implementar OpenTelemetry tracing con `tracing` + `opentelemetry-otlp` | 1. Añadir `tracing-opentelemetry`<br>2. Crear spans para cada query, fase del planificador, I/O<br>3. Exportar a Jaeger ou Tempo<br>4. Añadir `trace_id` en logs estructurados | High | Alto | Media | Ninguna | Medio | Ver traces en Jaeger para cada request | Latencia de tracing <5% overhead | Corto (2-3 semanas) | SRE |
| OPS-02 | Logs estructurados | Logs en texto plano, sin niveles consistentes, sin correlation IDs | Difícil de filtrar y correlacionar | Migrar a `tracing` con formato JSON y `trace_id` | 1. Configurar `tracing_subscriber` con formato JSON<br>2. Añadir campos estándar: `level`, `target`, `trace_id`, `span`<br>3. Enviar a stdout para recolección por Fluentd/Vector | High | Medio | Baja | OPS-01 | Bajo | Logs en JSON se pueden consultar con `jq` | 100% de logs estructurados | Corto (1 semana) | SRE |
| OPS-03 | Kubernetes readiness | No hay Helm charts, ni configmaps, ni probes | Despliegue manual, difícil de escalar | Crear Helm chart con liveness/readiness probes, recursos limitados, HPA | 1. Crear `Chart.yaml`, `values.yaml`<br>2. Definir deployment, service, configmap<br>3. Añadir probes HTTP (`/health`)<br>4. Configurar recursos requests/limits<br>5. Añadir HPA basado en CPU/memoria<br>6. Documentar despliegue | Medium | Alto | Alta | OPS-01, OPS-02 | Medio – cambios en configuración | Desplegar en minikube y ejecutar pruebas de carga | Tiempo de despliegue <2 minutos; autoescalado funcional | Medio (1-2 meses) | DevOps |
| OPS-04 | Disaster recovery | Solo cold backup via copia de directorio; RTO/RPO altos | Pérdida de datos y tiempo de inactividad prolongado | Implementar backups automáticos a S3 + restauración point-in-time para Fjall y RocksDB | 1. Script de backup diario que copia data dir a S3<br>2. Para RocksDB, usar checkpoint + upload<br>3. Para Fjall, detener escrituras brevemente, copiar, reanudar<br>4. Automatizar restore con `vanta-cli restore`<br>5. Probar restore semanalmente | High | Alto | Media | Ninguna | Medio | Simular desastre y restaurar en <1 hora | RTO < 1h, RPO < 24h | Medio (1 mes) | SRE |
| OPS-05 | Autoscaling horizontal | No soportado; solo modo standalone | Cuello de botella al crecer | Implementar modo cluster con descubrimiento de peers y sharding básico (por hash de namespace) | 1. Diseñar protocolo de gossip (usar `memberlist`)<br>2. Sharding consistente por `namespace`<br>3. Proxy de enrutamiento en servidor<br>4. Health checks entre nodos | Medium (post-MVP) | Alto | Muy Alta | ARQ-01, ARQ-02, OPS-03 | Muy alto – cambio fundamental | Cluster de 3 nodos procesa 3x escrituras | Throughput agregado escala linealmente con nodos | Largo (6-9 meses) | Distributed Systems Engineer |

---

# 6. Base de Datos y Datos

| ID | Categoría | Problema detectado | Riesgo actual | Solución propuesta | Subtareas | Prioridad | Impacto | Complejidad | Dependencias | Riesgo de implementación | Validación | Métricas | Horizonte | Owner |
|----|-----------|-------------------|---------------|--------------------|-----------|----------|---------|-------------|--------------|--------------------------|------------|----------|-----------|--------|
| DB-01 | Checksums en WAL | WAL no tiene CRC32; recuperación puede aplicar registros corruptos | Corrupción de datos silenciosa | Añadir CRC32 en cada registro WAL (como planeado) | 1. Modificar `WalWriter::append` para calcular CRC<br>2. Modificar `WalReader::next_record` para verificar<br>3. Agregar campo `crc` al formato<br>4. Añadir test que corrompe byte y verifica rechazo | Critical | Alto | Baja | Ninguna | Bajo – cambio de formato (migración necesaria) | Test de corrupción detecta error | 100% de registros con CRC válido | Corto (1 semana) | Backend |
| DB-02 | Checkpoints consistentes | WAL se trunca sin coordinación con backends | Pérdida de datos si falla después de checkpoint | Implementar checkpoint coordinado: sync backend → truncar WAL | 1. Añadir `WalWriter::checkpoint()` que llama `flush()` a backend<br>2. Solo truncar después de confirmación<br>3. Almacenar último offset truncado en metadata | High | Alto | Media | DB-01 | Medio | Test de crash recovery después de checkpoint | No se pierden datos en corte de energía | Corto (2 semanas) | Backend |
| DB-03 | Versionado de formato | `VantaFile` y `vector_index.bin` no tienen versión de schema | Migraciones imposibles; corrupción al cambiar structs | Añadir header con versión mayor/menor y magic bytes | 1. Modificar `VantaFile` para escribir `VANTA_VANTAFILE_V1`<br>2. Modificar `CPIndex::serialize_to_bytes` para incluir versión<br>3. Implementar lectores compatibles hacia atrás<br>4. Documentar política de versionado | Medium | Alto | Media | COD-02 (rkyv) | Medio | Migración desde V0 a V1 funciona sin pérdida | Soporte para al menos 3 versiones anteriores | Medio (1 mes) | Backend |
| DB-04 | Índices derivados reconstruibles | Ya existe `rebuild_index`, pero no se valida integridad automáticamente | Índices corruptos pasan desapercibidos | Añadir validación de integridad de índices derivados en cada apertura (si no read-only) | 1. Calcular checksum de namespace/payload indexes al cerrar<br>2. Al abrir, verificar checksum; si no coincide, rebuild<br>3. Añadir flag `force_rebuild` | Medium | Medio | Baja | Ninguna | Bajo | Test de corrupción manual desencadena rebuild | Tiempo de rebuild <1 min por 1M registros | Corto (1 semana) | Backend |

---

# 7. Producto y UX

| ID | Categoría | Problema detectado | Riesgo actual | Solución propuesta | Subtareas | Prioridad | Impacto | Complejidad | Dependencias | Riesgo de implementación | Validación | Métricas | Horizonte | Owner |
|----|-----------|-------------------|---------------|--------------------|-----------|----------|---------|-------------|--------------|--------------------------|------------|----------|-----------|--------|
| PD-01 | Onboarding CLI | `vanta-cli --help` no es claro; falta autocompletado | Nueva adopción lenta | Mejorar CLI con `clap` y subcomandos intuitivos | 1. Reemplazar parseo manual por `clap`<br>2. Añadir autocompletado para bash/zsh/fish<br>3. Escribir ejemplos en `--help`<br>4. Añadir `--interactive` para flujo guiado | Medium | Medio | Baja | ARQ-01 | Bajo | Usuario prueba quickstart en 5 minutos | Tasa de éxito de primer comando >90% | Corto (1 semana) | DevEx |
| PD-02 | Python SDK documentación | Falta docstrings, ejemplos, tipos | Dificultad de uso | Generar documentación con `pdoc` y añadir type hints | 1. Añadir docstrings a todas las funciones públicas en `vantadb-python/src/lib.rs`<br>2. Generar documentación HTML en CI<br>3. Publicar en GitHub Pages<br>4. Añadir type hints para PyO3 (usando `pyo3::pyclass` ya lo hace) | Medium | Medio | Baja | Ninguna | Bajo | `pdoc` genera docs sin errores | Documentación visitada >100 veces/semana | Corto (1 semana) | Developer Advocate |
| PD-03 | Telemetría de producto | Sin tracking de uso de features | Decisiones de producto basadas en intuición | Añadir telemetría anónima opcional (opt-in) | 1. Definir eventos: `query_executed`, `index_rebuilt`, `export_used`<br>2. Enviar a segment o self-hosted<br>3. Respetar `VANTA_TELEMETRY_OPTOUT`<br>4. Documentar política de privacidad | Low (post-MVP) | Bajo | Baja | Ninguna | Bajo | Dashboard muestra eventos | Tasa de opt-in >30% | Largo (3 meses) | Product Manager |

---

# 8. Mercado, Estrategia y Competitividad

| ID | Categoría | Problema detectado | Riesgo actual | Solución propuesta | Subtareas | Prioridad | Impacto | Complejidad | Dependencias | Riesgo de implementación | Validación | Métricas | Horizonte | Owner |
|----|-----------|-------------------|---------------|--------------------|-----------|----------|---------|-------------|--------------|--------------------------|------------|----------|-----------|--------|
| MKT-01 | Benchmark competitivo | No hay comparativas públicas contra LanceDB, Qdrant, pgvector | Imposible justificar superioridad técnica | Crear benchmark público con dataset estándar (Cohere, GIST) y métricas (QPS, recall, latencia) | 1. Seleccionar dataset (ej. Cohere 1M embeddings)<br>2. Implementar cliente de benchmark en Rust<br>3. Ejecutar contra VantaDB, LanceDB, Qdrant<br>4. Publicar resultados en blog técnico | High | Alto | Media | TST-03 | Medio – requiere infra externa | Resultados reproducibles por terceros | VantaDB es top-2 en al menos una métrica | Medio (2-3 meses) | Product/Engineering |
| MKT-02 | Posicionamiento "SQLite for vectors" | El mensaje "embedded first" no es único | Riesgo de commoditización | Adoptar narrativa "Embedded vector database with full-text and graph" | 1. Revisar README, landing page<br>2. Crear ejemplos de uso embebido (no servidor)<br>3. Enfatizar low-latency microsegundos<br>4. Comparar con SQLite + pgvector (más lento) | High | Alto | Baja | Ninguna | Bajo | Feedback de early adopters | Aumento de stars/forks en GitHub | Corto (1 semana) | Product Marketing |

---

# 9. Organización y Gestión Técnica

| ID | Categoría | Problema detectado | Riesgo actual | Solución propuesta | Subtareas | Prioridad | Impacto | Complejidad | Dependencias | Riesgo de implementación | Validación | Métricas | Horizonte | Owner |
|----|-----------|-------------------|---------------|--------------------|-----------|----------|---------|-------------|--------------|--------------------------|------------|----------|-----------|--------|
| ORG-01 | Architectural Decision Records (ADR) | No hay ADRs; decisiones se pierden | Inconsistencia, onboarding lento | Implementar ADRs en `docs/adr/` con formato estándar | 1. Crear plantilla ADR<br>2. Documentar decisiones pasadas (Fjall vs RocksDB, HNSW, WAL)<br>3. Exigir ADR para cualquier cambio arquitectónico futuro<br>4. Revisar en PR | High | Alto | Baja | Ninguna | Bajo | Existen al menos 5 ADRs en el repo | Nuevas decisiones se documentan en <2 días | Corto (1 semana) | Tech Lead |
| ORG-02 | Onboarding técnico | No hay guía de contribución clara (CONTRIBUTING.md existe pero incompleta) | Dificultad para nuevos contribuyentes | Expandir `CONTRIBUTING.md` con setup, testing, arquitectura, ejemplos | 1. Añadir sección "Setup local"<br>2. Añadir "Arquitectura en 5 minutos"<br>3. Añadir "Cómo añadir una nueva característica"<br>4. Añadir "Guía de debugging" | Medium | Medio | Baja | Ninguna | Bajo | Un nuevo contribuidor puede hacer un PR funcional en 1 semana | Tiempo medio para primer PR <30 días | Corto (1 semana) | DevRel |
| ORG-03 | Postmortem y cultura de incidentes | No hay proceso documentado | Los errores se repiten | Establecer plantilla de postmortem y revisión obligatoria después de incidentes | 1. Crear plantilla en `docs/incidents/`<br>2. Definir severidades (P0-P4)<br>3. Asignar acciones correctivas con dueños y fechas<br>4. Revisar en retrospectiva mensual | Medium | Medio | Baja | Ninguna | Bajo | Después de cada incidente se publica postmortem | Tiempo medio de resolución de incidentes -20% | Corto (1 semana) | SRE Manager |

---

# 10. Escalabilidad a Futuro

| ID | Categoría | Problema detectado | Riesgo actual | Solución propuesta | Subtareas | Prioridad | Impacto | Complejidad | Dependencias | Riesgo de implementación | Validación | Métricas | Horizonte | Owner |
|----|-----------|-------------------|---------------|--------------------|-----------|----------|---------|-------------|--------------|--------------------------|------------|----------|-----------|--------|
| SCA-01 | Sharding horizontal | No soportado; crecimiento >1TB imposible | Límite a 10M-100M nodos | Diseñar particionamiento por `namespace` con hash consistente | 1. Definir `ShardManager` en servidor<br>2. Implementar redirección de queries<br>3. Soportar rebalanceo online<br>4. Integrar con etcd o Raft para metadata | Low (post-MVP) | Alto | Muy Alta | ARQ-01, OPS-05 | Muy alto | Cluster de 10 shards maneja 10x datos | Throughput lineal con número de shards | Largo (9-12 meses) | Distributed Systems |
| SCA-02 | HNSW mmap para capas superiores | Actualmente todo en memoria; limita tamaño de datasets | No se pueden indexar >RAM | Implementar mmap para capas superiores (L1+) y caché de capas calientes | 1. Refactorizar `CPIndex` para usar `IndexBackend::MMap` para layers >0<br>2. Configurar página tamaño y prefetch<br>3. Añadir métricas de page faults | High | Alto | Alta | COD-02 (rkyv) | Alto | Indexar 10M vectores en máquina con 16GB RAM | Memoria RSS <4GB para 10M vectores | Medio (3-4 meses) | Storage Engineer |
| SCA-03 | Replicación y failover | Sin modo cluster; single point of failure | No apto para producción enterprise | Implementar líder-elector con Raft y replicación síncrona | 1. Integrar `raft-rs` o `openraft`<br>2. Definir log de replicación para mutaciones<br>3. Soportar lecturas en followers (eventual consistency)<br>4. Failover automático | Low (post-MVP) | Alto | Muy Alta | OPS-05, SCA-01 | Muy alto | Cluster de 3 nodos tolera fallo de 1 nodo | RTO < 30s, RPO = 0 | Largo (12-18 meses) | Distributed Systems |

---

# Entregables Finales

## 1. Roadmap de 30 días (Quick Wins + Estabilización)

| Semana | Tareas clave | Entregables |
|--------|--------------|--------------|
| 1 | COD-03 (jemalloc), TST-03 (benchmark CI), SEC-04 (secret scanning), ORG-01 (ADRs iniciales) | Memoria estable, CI con benchmarks, sin secrets en repo, 5 ADRs |
| 2 | ARQ-03 (eliminar experimental), COD-01 (spawn_blocking parcial), DB-01 (CRC32 WAL), PD-01 (CLI con clap) | Reducción de LOC 15%, sin bloqueos en I/O, WAL con checksums, CLI usable |
| 3 | ARQ-05 (config unificada), OPS-02 (logs estructurados), PD-02 (doc Python) | Configuración centralizada, logs JSON, docs Python generadas |
| 4 | TST-01 (chaos mesh básico), SEC-01 (auth token), OPS-01 (tracing básico) | Experimentos de caos iniciales, servidor con auth, traces a Jaeger |

## 2. Roadmap de 90 días (Endurecimiento del MVP)

| Mes | Tareas clave | Entregables |
|-----|--------------|--------------|
| 1 | ARQ-01 (separación servidor/core completada), COD-02 (rkyv migración parcial), DB-02 (checkpoints coordinados) | `vantadb-core` sin dependencias de red, zero-copy en estructuras clave, WAL truncado seguro |
| 2 | ARQ-02 (AST/IR completado), TST-02 (fuzzing WAL), SEC-02 (TLS) | Planificador pipeline funcionando, fuzzing sin crashes, servidor con TLS |
| 3 | OPS-03 (Helm chart), DB-03 (versionado), SCA-02 (HNSW mmap capas superiores) | Despliegue en K8s, formatos versionados, indexación de datasets >RAM |

## 3. Roadmap de 6 meses

| Meses | Tareas clave | Entregables |
|-------|--------------|--------------|
| 4-5 | OPS-04 (disaster recovery), SEC-03 (rate limiting), TST-04 (cobertura 70%) | Backups automáticos, rate limiting, reporte de cobertura |
| 6 | MKT-01 (benchmark competitivo), SCA-01 (sharding diseño), COD-05 (errores consistentes) | Benchmark público, diseño de sharding, 0 panics en producción |

## 4. Roadmap de 1 año

| Meses | Tareas clave | Entregables |
|-------|--------------|--------------|
| 7-9 | OPS-05 (autoscaling), SCA-01 (sharding implementación parcial) | Cluster con 3 nodos, escalado manual |
| 10-12 | SCA-03 (replicación Raft), MKT-02 (posicionamiento enterprise) | Failover automático, caso de éxito enterprise |

## 5. Orden óptimo de ejecución (por dependencias)

1. **Semana 1:** COD-03, SEC-04, ORG-01 (cambios de bajo riesgo, alto impacto)
2. **Semana 2:** ARQ-03, DB-01, COD-01 (preparan el terreno)
3. **Semana 3:** ARQ-05, OPS-02, PD-01 (mejoras operativas)
4. **Semana 4:** TST-01, SEC-01, OPS-01 (observabilidad y seguridad)
5. **Mes 2:** ARQ-01 (separación servidor – requiere estabilidad previa)
6. **Mes 3:** ARQ-02 (planificador – requiere core estable)
7. **Mes 4-5:** DB-02, DB-03, OPS-03 (resiliencia y despliegue)
8. **Mes 6+:** SCA-02, SCA-01, SCA-03 (escalabilidad distribuida)

## 6. Matriz Impacto vs Esfuerzo

| Esfuerzo | Bajo | Medio | Alto |
|----------|------|-------|------|
| **Impacto Alto** | COD-03 (allocator), SEC-04 (secret scanning), DB-01 (CRC32), PD-01 (CLI) | COD-01 (async I/O), SEC-01 (auth), OPS-01 (tracing), ARQ-03 (eliminar experimental) | ARQ-01 (separación server), ARQ-02 (AST), SCA-02 (mmap HNSW) |
| **Impacto Medio** | PD-02 (doc Python), ORG-01 (ADRs) | TST-04 (cobertura), DB-03 (versionado), OPS-02 (logs) | OPS-03 (K8s), OPS-05 (autoscaling) |
| **Impacto Bajo** | - | TST-02 (fuzzing) | SCA-01 (sharding), SCA-03 (replicación) |

## 7. Riesgos de No Actuar

| Riesgo | Probabilidad | Consecuencia | Plazo |
|--------|--------------|--------------|-------|
| Corrupción de datos por WAL sin checksum | Media | Pérdida irreversible de datos | 3-6 meses |
| OOM por fragmentación de memoria | Alta | Caídas frecuentes en producción | 6-12 meses |
| Estancamiento del mercado por falta de benchmark competitivo | Alta | Pérdida de cuota frente a LanceDB | 12 meses |
| Imposibilidad de escalar >10M vectores | Alta | Pérdida de clientes enterprise | 18 meses |

## 8. Áreas que requieren reescritura

| Módulo | Por qué | Alternativa | Esfuerzo |
|--------|--------|-------------|----------|
| `src/governance/` | Complejidad accidental, no usada | Eliminar | Bajo |
| `src/eval/vm.rs` | LISP VM obsoleta | Eliminar | Bajo |
| `src/llm.rs` | Dependencia externa frágil | Mover a crate aparte | Bajo |
| `src/parser/lisp.rs` | Experimental | Eliminar | Bajo |

## 9. Áreas que solo necesitan refactor

| Módulo | Refactor necesaria | Esfuerzo |
|--------|-------------------|----------|
| `src/planner.rs` | AST/IR pipeline | Alto |
| `src/storage.rs` | Dividir en múltiples archivos | Medio |
| `src/sdk.rs` | Extraer export/import | Medio |
| `src/wal.rs` | Añadir CRC32, rotación | Bajo |

## 10. Áreas que deben eliminarse (inmediato)

- `src/governance/` (todo)
- `src/eval/`
- `src/api/mcp.rs`
- `vantadb-server/src/mcp.rs`
- `examples/docker/`
- `benches/high_density.rs` (si no es usado)
- `vanta_certification.json` del raíz (debe generarse)

## 11. Quick Wins (menos de 1 día)

| Tarea | Impacto |
|-------|---------|
| Añadir `jemalloc` | Memoria estable |
| Ejecutar `cargo fmt` y `clippy --fix` | Código limpio |
| Mover `vantadb_data/` a `.gitignore` | Repo limpio |
| Añadir `cargo audit` a CI | Seguridad de dependencias |
| Crear primer ADR (Fjall vs RocksDB) | Trazabilidad |

## 12. Bottlenecks futuros (prepararse ahora)

| Bottleneck | Solución temprana |
|------------|-------------------|
| WAL escritura secuencial | Batching, async write, sharded WAL |
| HNSW en memoria | Mmap para capas superiores (SCA-02) |
| Lock global en `StorageEngine::hnsw` | Sharding por partición HNSW |
| Serialización bincode | rkyv zero-copy (COD-02) |
| Single node | Diseño de sharding (SCA-01) |

## 13. Costos ocultos futuros

| Área | Costo estimado | Mitigación |
|------|----------------|-------------|
| Migración de formato VantaFile | 2-3 semanas ingeniería | Versionado ahora (DB-03) |
| Reescribir sistema de configuración | 1 semana | Hacerlo ahora (ARQ-05) |
| Añadir autenticación después | 2 semanas | Hacerlo ahora (SEC-01) |

## 14. Riesgos de escalabilidad

| Crecimiento | Problema | Solución |
|-------------|----------|----------|
| 10x nodos (10M) | OOM, WAL write bottleneck | mmap HNSW, async WAL |
| 100x nodos (100M) | No cabe en un nodo, índices derivados gigantes | Sharding, replicación |
| 10x QPS (100k) | Lock en planificador, contención en HNSW | Planificador sin locks, sharding |

## 15. Riesgos de mercado

| Riesgo | Mitigación |
|--------|-------------|
| Commoditización de vectores | Enfatizar diferenciación: híbrido + embedded + bajo costo |
| Competidores con más recursos (AWS, Google) | Enfoque en nicho "desarrolladores locales" |
| Falta de ecosistema | Construir integraciones con LangChain, LlamaIndex |

## 16. Riesgos operacionales

| Riesgo | Mitigación |
|--------|-------------|
| Despliegue complejo | Helm chart, documentación clara |
| Falta de habilidades SRE | Contratar SRE dedicado |
| Onboarding lento | Guías, ADRs, ejemplos |

## 17. Riesgos de seguridad

| Riesgo | Mitigación |
|--------|-------------|
| Exposición del servidor sin auth | SEC-01 (token) |
| Dependencias vulnerables | `cargo audit` diario |
| Filtración de secrets | pre-commit hook (SEC-04) |

## 18. Riesgos organizacionales

| Riesgo | Mitigación |
|--------|-------------|
| Bus factor alto | Documentación, ADRs, código legible, ownership compartido |
| Decisiones no documentadas | ADRs obligatorios |
| Cultura de no postmortem | Postmortem obligatorio (ORG-03) |

## 19. Plan de reducción de deuda técnica

| Deuda | Esfuerzo | Semana |
|-------|----------|--------|
| Eliminar experimental | Bajo | 1 |
| Formatear y linting | Bajo | 1 |
| Separar servidor/core | Alto | 4-8 |
| Refactorizar planificador | Alto | 8-12 |
| Unificar configuración | Medio | 3 |
| Añadir CRC32 | Bajo | 2 |

## 20. Plan de preparación para enterprise customers

| Requisito | Tarea | Mes |
|-----------|-------|-----|
| Alta disponibilidad | Replicación + failover (SCA-03) | 12 |
| Seguridad | TLS, auth, audit logs (SEC-01, SEC-02) | 2 |
| Cumplimiento | GDPR, SOC2 (auditoría de logs, encryption at rest) | 6 |
| Soporte | Contrato SLA, versión LTS | 9 |
| Monitoreo | Dashboards Grafana, alertas PagerDuty | 4 |

---

*Fin del plan de remediación.*  
*El comité recomienda ejecutar este plan con disciplina militar, o aceptar que VantaDB nunca será más que un prototipo académico.*