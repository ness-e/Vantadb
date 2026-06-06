# T3.1 — Chaos Testing Expandido y Validación de Durabilidad

## Contexto y Diagnóstico

### Estado Pre-Implementación (Auditado)

| Subtarea | Estado Real | Gap |
|---|---|---|
| ST3.1.1 — `chaos_integrity.rs` failpoints | ✅ Implementado | Solo cubre `wal_append_fail`. `storage_insert_fail` instrumentado pero sin test. |
| ST3.1.2 — Script de loop de caos (1,000 iter.) | ⬜ No existe | `dev-tools/` no tiene loop. `nocturnal_suite.ps1` ejecuta chaos **una sola vez**. |
| ST3.1.3 — `RELIABILITY_GATE.md` completo | 🔄 Parcial | Existe (93 líneas) pero solo documenta RSS/T2.2. No cubre caos, no enlazado desde README. |

### Failpoints Existentes en Producción
- `wal_append_fail` → `src/wal.rs:286` — WAL append
- `storage_insert_fail` → `src/storage.rs:1244` — Storage insert

El test `chaos_integrity_failpoints_certification` **solo ejerce `wal_append_fail`**. `storage_insert_fail` es código muerto de cara a la certificación.

---

## Objetivo

Completar T3.1 con evidencia verificable de que el motor tolera y se recupera de fallos inyectados de forma repetida (≥ 100 iteraciones en CI, ≥ 1,000 en el loop manual). Esto incluye:

1. **Cerrar el gap de cobertura de failpoints** — añadir test para `storage_insert_fail`.
2. **Crear el script de loop de caos en PowerShell** (plataforma del proyecto: Windows) en `dev-tools/chaos_loop.ps1`.
3. **Expandir y completar `RELIABILITY_GATE.md`** para que sea la puerta formal de certificación de caos + durabilidad.
4. **Enlazar `RELIABILITY_GATE.md` desde el README principal**.
5. **Registrar `chaos_integrity` en el perfil `audit` de nextest** para que sea gate en CI rápido.

---

## User Review Required

> [!IMPORTANT]
> **Número de iteraciones del loop de caos:** El Plan Maestro especifica 1,000 iteraciones. En Windows, ejecutar 1,000 compilaciones Rust completas en CI es inviable (tiempo > 4h). La propuesta es:
> - **CI (`nextest audit`)**: 1 ejecución del test de chaos en cada PR (ya corre en ~2s).
> - **Script `chaos_loop.ps1`**: Loop de N iteraciones ejecutando el binario de test ya compilado (no recompila), parametrizable. Por defecto `--iterations 100` en CI nocturno y `--iterations 1000` en certificación manual. ¿Aceptas esta interpretación?

> [!IMPORTANT]
> **Adición de `chaos_integrity` a `nextest audit`:** El perfil `audit` actual excluye tests pesados. `chaos_integrity_certification` es ligero (~0.5s) y `chaos_integrity_failpoints_certification` requiere `--features failpoints` (no se puede incluir en el perfil estándar sin un perfil separado). La propuesta es: añadir `chaos_integrity_certification` (sin failpoints) al perfil `audit` existente, y crear un perfil `chaos` separado en nextest.toml para los tests con `--features failpoints`. ¿Conforme?

---

## Open Questions

> [!NOTE]
> **¿Nuevos escenarios de caos a añadir?** Actualmente los failpoints cubren WAL append y storage insert. Existen puntos naturales adicionales: `mmap_flush_fail`, `hnsw_serialize_fail`. No están en scope de T3.1 pero si quieres añadir alguno, dímelo antes de ejecutar.

---

## Proposed Changes

### Componente 1: Tests de Caos (Nuevos Escenarios)

---

#### [MODIFY] [chaos_integrity.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/storage/chaos_integrity.rs)

Añadir un tercer bloque al final del archivo: **`chaos_integrity_storage_failpoint_certification`**. Este test ejerce el failpoint `storage_insert_fail` de forma análoga al test del WAL:

1. Activa `storage_insert_fail` → verifica que `StorageEngine::insert()` retorna `Err`.
2. Desactiva el failpoint → verifica auto-recuperación (insert posterior exitoso).
3. Verifica que el nodo insertado post-recovery es recuperable vía `get()`.

```rust
#[test]
fn chaos_integrity_storage_failpoint_certification() {
    #[cfg(feature = "failpoints")]
    {
        // Escenario: Fallo catastrófico de I/O durante StorageEngine::insert()
        // Valida que el motor rechaza limpiamente y se auto-recupera sin corrupción.
        ...
    }
}
```

---

### Componente 2: Script de Loop de Caos

---

#### [NEW] [chaos_loop.ps1](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/dev-tools/chaos_loop.ps1)

Script PowerShell parametrizable que:

1. Recibe `--iterations N` (default 100), `--release` (flag), `--log-path` (optional).
2. **Compila una sola vez** el binario de test con `cargo test --test chaos_integrity --features failpoints --release --no-run`.
3. Ejecuta el binario compilado N veces en un loop, capturando stdout/stderr.
4. Cuenta pases y fallos. Reporta ratio al final.
5. Devuelve exit code 1 si hay cualquier fallo.
6. Genera un archivo de log JSON con timestamp, iteraciones, resultado por iteración y ratio final.

**Firma:**
```powershell
.\dev-tools\chaos_loop.ps1 [-Iterations 100] [-Release] [-LogPath "chaos_results.json"]
```

---

### Componente 3: Configuración CI (nextest)

---

#### [MODIFY] [nextest.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/.config/nextest.toml)

Dos cambios:

**A. Añadir `chaos_integrity_certification` al perfil `audit`** (no requiere `failpoints`):
- Eliminar `chaos_integrity_certification` del filtro de exclusión implícito del perfil `audit` (actualmente no está listado, así que ya corre — confirmar).
- Ajustar `slow-timeout` del perfil `audit` para tolerarlo si es lento.

**B. Nuevo perfil `chaos`** para tests que requieren `--features failpoints`:
```toml
[profile.chaos]
default-filter = "test(chaos_integrity_failpoints) or test(chaos_integrity_storage_failpoint)"
fail-fast = false
failure-output = "immediate-final"
slow-timeout = { period = "30s", terminate-after = 3 }
```

> [!WARNING]
> El perfil `chaos` no puede activar features de Cargo directamente desde nextest.toml. El comando correcto será:
> `cargo nextest run --profile chaos --features failpoints`
> Esto debe documentarse en RELIABILITY_GATE.md.

---

### Componente 4: Documentación

---

#### [MODIFY] [RELIABILITY_GATE.md](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/operations/RELIABILITY_GATE.md)

Expandir el documento de 93 líneas a un documento de puerta de certificación completo:

**Secciones a añadir:**

1. **Sección 2 — Chaos Integrity Gate** (nueva):
   - Descripción de cada failpoint instrumentado (`wal_append_fail`, `storage_insert_fail`).
   - Comando CI: `cargo nextest run --profile chaos --features failpoints`.
   - Comando de loop manual: `.\dev-tools\chaos_loop.ps1 -Iterations 1000 -Release`.
   - Criterio de aceptación: **0 fallos en N=1,000 iteraciones**.
   - Plantilla de registro de resultado (tabla markdown con fecha, iteraciones, ratio, observaciones).

2. **Sección 3 — Durabilidad WAL y Recuperación Fría** (nueva):
   - Referencia a los tests de `wal_resilience.rs` and `durability_recovery.rs`.
   - Comandos de ejecución manual.

3. **Actualizar Sección 1** (RSS/T2.2 existente): Renombrar a "Sección 1 — RSS Stability Gate" para contexto.

---

#### [MODIFY] [README.MD](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/README.MD)

Añadir enlace a `docs/operations/RELIABILITY_GATE.md` en la sección de desarrollo/testing del README. Una línea bajo el encabezado de Tests o CI.

---

## Verification Plan

### Automated Tests

```powershell
# 1. Test topológico sin failpoints (debe pasar en el perfil audit normal)
cargo test --test chaos_integrity --release -- chaos_integrity_certification --nocapture

# 2. Tests con failpoints habilitados (WAL + Storage)
cargo test --test chaos_integrity --features failpoints --release -- --nocapture --test-threads=1

# 3. Loop de 10 iteraciones (smoke test del script)
.\dev-tools\chaos_loop.ps1 -Iterations 10 -Release
```

### Manual Verification

- Ejecutar `.\dev-tools\chaos_loop.ps1 -Iterations 1000 -Release` y verificar que el ratio de éxito es 100%.
- Confirmar que `RELIABILITY_GATE.md` está enlazado y accesible desde `README.MD`.
- Confirmar que el perfil `chaos` en nextest.toml es seleccionable sin errores de configuración.
