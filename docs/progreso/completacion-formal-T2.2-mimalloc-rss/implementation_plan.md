# T2.2 — Completación Formal: Asignador mimalloc + Métricas RSS + Estrés de Estabilidad

## Descripción del Problema

La tarea T2.2 del Plan Maestro tiene **ST2.2.1 ya completado** (mimalloc integrado bajo feature flag `custom-allocator`), pero los sub-criterios ST2.2.2 y ST2.2.3 siguen **pendientes de implementación formal**:

- **ST2.2.3:** `db.hardware_profile()` actualmente es un alias de `capabilities()` y retorna solo capacidades booleanas (`{profile, read_only, vector_search, …}`). El criterio exige que retorne las **3 métricas de memoria diferenciadas** (RSS físico del OS, memoria lógica HNSW, páginas residentes mmap). La infraestructura en `src/metrics.rs` (`MemoryBreakdownSnapshot`) y en `VantaOperationalMetrics` **ya existe** — el gap es únicamente en la superficie pública del método Python.
- **ST2.2.2:** No existe ningún test de estrés de RSS. Requiere crear un script Rust de medición de drift de memoria bajo carga de 100K inserciones.

## Verificación Previa: Lo que YA existe

| Componente | Estado |
|---|---|
| `mimalloc` en `Cargo.toml` feature `custom-allocator` | ✅ Ya implementado |
| `src/metrics.rs::MemoryBreakdownSnapshot` con RSS/HNSW/mmap | ✅ Ya implementado |
| `VantaOperationalMetrics` con campos de memoria en `sdk.rs` | ✅ Ya implementado |
| `record_memory_breakdown()` con APIs nativas Win32/Linux/macOS | ✅ Ya implementado |
| `operational_metrics()` en Python binding (expone las 3 métricas) | ✅ Ya implementado |
| `hardware_profile()` retornando las 3 métricas de memoria | ❌ **Pendiente** — alias incorrecto |
| Test de estrés de RSS 100K vectores / estabilidad | ❌ **Pendiente** |

## Cambios Propuestos

---

### Componente: Python SDK Binding

#### [MODIFY] [lib.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-python/src/lib.rs)

- Reemplazar el alias `hardware_profile() → capabilities()` por una implementación que tome un snapshot de `operational_metrics()` y retorne un dict con las **3 métricas diferenciadas** más las capacidades básicas.
- Estructura del dict de retorno:
  ```python
  {
    # Capacidades (backwards-compatible)
    "profile": "PERFORMANCE",
    "read_only": False,
    "persistence": True,
    "vector_search": True,
    "iql_queries": True,
    # Métricas de memoria (nuevas — criterio ST2.2.3)
    "process_rss_bytes": 134217728,
    "process_virtual_bytes": 268435456,
    "hnsw_logical_bytes": 51200000,
    "hnsw_nodes_count": 100000,
    "mmap_resident_bytes": None,  # None si no hay backend mmap
    "volatile_cache_entries": 0,
    "volatile_cache_cap_bytes": 0,
  }
  ```
- **Criterio de Aceptación:** El test existente `test_hardware_profile_alias` en `vantadb-python/tests/test_sdk.py` debe seguir pasando. Se añade validación de las nuevas claves.

---

### Componente: Test de Estrés de RSS (ST2.2.2)

#### [MODIFY] [tests/certification/hardware_profiles.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/certification/hardware_profiles.rs)

- Los 30 minutos de estrés en CI no son viables por límite de tiempo. Implementamos un test rápido de carga unitario que valida el drift de working set de RSS con 100K vectores (~30s).
- El test extendido se documenta operacionalmente en `docs/operations/RELIABILITY_GATE.md`.

---

### Componente: Plan Maestro Unificado (Actualización de Estado)

#### [MODIFY] [VantaDB_Plan_Maestro_Unificado.md](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/VantaDB_Plan_Maestro_Unificado.md)

- Actualizar T2.2 de `⬜ PENDIENTE` a `✅ COMPLETADA` con evidencia al finalizar.
- Actualizar T2.4 de `🔄 EN PROGRESO` a `✅ COMPLETADA` con evidencia de la sesión anterior.

---

## Open Questions

> [!IMPORTANT]
> **ST2.2.2 — Interpretación del criterio "30 minutos":** El Plan Maestro dice "Medir RSS en un loop de inserción de 100K vectores durante 30 minutos". Un test de CI de 30 minutos no es viable. La propuesta es:
> - **Test rápido en CI:** 100K inserciones + medición de drift RSS al finalizar (< 30s).
> - **Instrucción manual:** Documento en `RELIABILITY_GATE.md` con el comando para correr el estrés extendido manualmente.
> ¿Aceptas este enfoque?

> [!NOTE]
> **`hardware_profile()` backwards compatibility:** El método actualmente retorna el mismo dict que `capabilities()`. Al agregar las nuevas claves de memoria, el dict será un **superconjunto** del anterior — no hay breaking change. El test existente `test_hardware_profile_alias` seguirá pasando sin modificación.

## Plan de Verificación

### Tests Automáticos
```powershell
# Verificar que el binding hardware_profile() retorna las métricas de memoria
cargo test --package vantadb mmap_vector_index_certification -- --nocapture

# Test de estabilidad RSS
cargo test --test hardware_profiles rss_stability_under_bulk_insert --release -- --nocapture

# Verificar que tests Python SDK siguen pasando
# (ejecutar manualmente desde vantadb-python/)
# pytest tests/test_sdk.py -v -k "hardware_profile"
```

### Verificación Manual
1. Confirmar que `db.hardware_profile()` desde Python devuelve `process_rss_bytes` > 0.
2. Confirmar que el test de estrés RSS reporta drift < 15%.
3. Actualizar el Plan Maestro con el estado ✅ de T2.2 y T2.4.
