# OLD-08: Life Insurance — Snapshots via Hard Links

**Fuente:** Backlog Phase 9 (Old Docs Rescue)  
**Estado:** ⚠️ Parcial — `tests/core/snapshot_certification.rs` existe, hard-link pattern no  
**Effort:** 🟡 3-4d  
**Dependencias:** Ninguna. Solo syscalls POSIX  

## Gate
✅ DO — `snapshot_certification.rs` test exists. POSIX `link()` syscall available on all target platforms (Linux, macOS, Windows via `CreateHardLinkA`). The hard-link pattern gives instant O(1) snapshots without copying data.

## Objetivo
Implementar snapshot via hard links: instant copy-on-write snapshots usando `std::os::unix::fs::link()` (POSIX) y `CreateHardLinkA` (Windows).

## Archivos

| Archivo | Qué hacer |
|---------|-----------|
| `src/storage/engine/snapshot.rs` | Agregar `SnapshotManager::create_hardlink_snapshot(name)` |
| `src/storage/engine/mod.rs` | Exponer `create_snapshot(name)` en `StorageEngine` |
| `src/sdk/builder.rs` | Agregar `VantaEmbedded::create_snapshot(name)` |
| `src/cli.rs` | Agregar subcomando `vantadb snapshot create <name>` |
| `tests/core/snapshot_certification.rs` | Tests para hard-link snapshot |
| `Cargo.toml` | `#[cfg(unix)]` / `#[cfg(windows)]` para link syscall |

## Pasos

### 1. Leer snapshot_certification.rs
Entender qué certifica actualmente.

### 2. Implementar hard-link snapshot en SnapshotManager
```rust
impl SnapshotManager {
    /// Create an instant O(1) snapshot by hard-linking all data files.
    #[cfg(unix)]
    pub fn create_hardlink_snapshot(&self, name: &str) -> Result<Snapshot> {
        let snap_dir = self.dir.join("snapshots").join(name);
        std::fs::create_dir_all(&snap_dir)?;
        for entry in std::fs::read_dir(&self.dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let dest = snap_dir.join(entry.file_name());
                std::os::unix::fs::link(&path, &dest)?;
            }
        }
        Ok(Snapshot { path: snap_dir, created_at: Instant::now() })
    }
}
```

### 3. Exponer en StorageEngine
```rust
impl StorageEngine {
    pub fn create_snapshot(&self, name: &str) -> Result<Snapshot> {
        self.snapshot_manager.create_hardlink_snapshot(name)
    }
}
```

### 4. Exponer en VantaEmbedded
```rust
impl VantaEmbedded {
    pub fn create_snapshot(&self, name: &str) -> Result<SnapshotInfo> {
        let engine = self.engine_handle()?;
        let snapshot = engine.create_snapshot(name)?;
        Ok(SnapshotInfo { name: name.into(), path: snapshot.path })
    }
}
```

### 5. Subcomando CLI
```rust
Command::Snapshot { sub } => match sub {
    SnapshotSub::Create { name } => {
        let snap = engine.create_snapshot(&name)?;
        println!("Snapshot created: {:?}", snap.path);
    }
}
```

### 6. Tests
- Test que snapshot contiene archivos esperados
- Test que snapshot es instantáneo (< 1s)
- Test que modificar data original no afecta snapshot (COW)
- Test que se pueden listar snapshots existentes

### 7. Verificación
```bash
cargo check -p vantadb
cargo nextest run -p vantadb -- snapshot
cargo clippy -p vantadb -- -D warnings
```

### 8. Progreso
- Marcar OLD-08 ✅ en Backlog.md
- Agregar entry en progreso/README.md
- Auto-commit
