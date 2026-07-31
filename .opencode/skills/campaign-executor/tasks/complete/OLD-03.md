# OLD-03: Chaos Testing (Failpoint Framework Formal)

**Fuente:** Backlog Phase 9 (Old Docs Rescue)  
**Estado:** ⚠️ Parcial — `tests/storage/chaos_integrity.rs` + `chaos_test_wal.sh` existen, no marco formal  
**Effort:** 🟡 2-3 sem → ponytail: 🟢 2-3d (formalizar lo que ya existe, no implementar Jepsen)  
**Dependencias:** Docker. WAL shipping existente ✅

## Gate
✅ DO — failpoint infra ya existe: `fail` crate v0.5, feature `failpoints`, `cfg_failpoint()`/`remove_failpoint()` públicos, 4 failpoints (WAL append, storage insert, mmap flush, HNSW serialize), 1 test de integridad. Falta: formalizar como test harness reutilizable.

## Objetivo (ponytail)
No implementar Jepsen/Maelstrom completo (2-3 sem). En cambio:
1. Refactorizar el failpoint test existente en un `ChaosTestHarness` reutilizable
2. Agregar failpoints en paths críticos faltantes (edge write, snapshot, index query)
3. Script CI que corre failpoint tests con `--features failpoints`
4. Documentar el patrón para agregar failpoints nuevos

## Archivos

| Archivo | Qué hacer |
|---------|-----------|
| `tests/storage/chaos_integrity.rs` | Refactor: extraer `ChaosTestHarness` con setup/teardown/assert |
| `src/lib.rs` | Si hace falta, mejorar API de failpoints |
| `src/edge_index.rs` | Agregar failpoint `edge_write_fail` |
| `src/storage/engine/snapshot.rs` | Agregar failpoint `snapshot_serialize_fail` |
| `src/index/hnsw.rs` | Verificar si `hnsw_serialize_fail` ya cubre query path |
| `scripts/chaos-test.ps1` o similar | Script CI que corre failpoints con --features failpoints |
| `.github/workflows/chaos.yml` | Workflow CI para failpoint tests |
| `docs/chaos-testing.md` | Documentar patrón de failpoints |

## Pasos

### 1. Leer infra actual
- `tests/storage/chaos_integrity.rs` — entender estructura
- `tests/storage/wal_resilience.rs` — WAL-specific failpoints
- `src/lib.rs` — cfg_failpoint API

### 2. Crear ChaosTestHarness
Extraer patrón común:
```rust
pub struct ChaosTestHarness {
    dir: TempDir,
    failpoints_active: Vec<String>,
}

impl ChaosTestHarness {
    pub fn new() -> Result<Self> { /* setup tempdir + engine */ }
    pub fn enable_failpoint(&mut self, name: &str, action: &str) { /* track + enable */ }
    pub fn disable_all(&mut self) { /* remove all tracked failpoints */ }
    pub fn assert_recovery(&self) { /* verify engine recovers after failpoints removed */ }
}
```

### 3. Agregar failpoints faltantes
- `edge_index.rs`: `fail::cfg("edge_write_fail", "return")` antes de insert edge
- `snapshot.rs`: `fail::cfg("snapshot_serialize_fail", "return")` antes de serializar snapshot

### 4. Script CI
```powershell
# scripts/chaos-test.ps1
cargo nextest run --features failpoints -p vantadb -- test_chaos
```

### 5. Documentar
`docs/chaos-testing.md` con:
- Cómo agregar un failpoint nuevo
- Cómo correr tests
- Expected behavior: engine debe recuperarse siempre

### 6. Verificación
```bash
cargo nextest run --features failpoints -p vantadb -- test_chaos
```

### 7. Progreso
- Marcar OLD-03 ✅ en Backlog.md
- Agregar entry en progreso/README.md
- Auto-commit
