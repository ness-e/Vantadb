---
title: "Failpoints"
type: glossary-entry
status: stable
tags: [testing, fault-injection, debugging]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Failpoint Injection, Error Injection]
---
# Failpoints

##Definition

**Failpoints** are **error injection points** inserted in the code that allow specific failures (I/O errors, timeouts, corruption) to be simulated in a controlled way during testing, validating error handling without the need for real failures.

## How It Works

### Failpoint insertion

```rust
use fail::fail_point;

pub fn write_to_disk(&self, data: &[u8]) -> Result<()> {
    // Failpoint: simulate I/O error
    fail_point!("disk_write_error", |_| {
        Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Simulated disk error"
        )))
    });
    
    // Actual code
    self.file.write_all(data)?;
    self.file.sync_all()?;
    Ok(())
}
```

### Activation in Tests

```rust
#[test]
fn test_disk_error_handling() {
    // Activar failpoint
    fail::cfg("disk_write_error", "return").unwrap();
    
    // Ejecutar código que usa failpoint
    let result = db.put("key", &vec![1.0; 128], "test");
    
    // Verificar manejo de error
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("disk error"));
    
    // Desactivar failpoint
    fail::remove("disk_write_error");
}
```

## Types of Failpoints

| Tipo | Descripción | Ejemplo |
|------|-------------|---------|
| **return** | Retornar error inmediatamente | `fail_point!("name", \|_\| Err(...))` |
| **delay** | Introducir latencia | `fail_point!("name", \|_\| sleep(Duration::from_secs(5)))` |
| **panic** | Causar panic | `fail_point!("name", \|_\| panic!("test"))` |
| **corrupt** | Corromper datos | `fail_point!("name", \|data\| corrupt(data))` |

## Failpoints in VantaDB

### Key Locations

```rust
// WAL: Error durante append
fail_point!("wal_append_error");

// WAL: Error during fsync
fail_point!("wal_fsync_error");

// Storage: Error during compaction
fail_point!("compaction_error");

// Recovery: Registry corruption
fail_point!("recovery_corruption");

// HNSW: Error during rebuild
fail_point!("hnsw_rebuild_error");
```

### Testing with Failpoints

```rust
#[test]
fn test_wal_fsync_failure() {
    fail::cfg("wal_fsync_error", "return").unwrap();
    
    let db = VantaEmbedded::open("./test")?;
    let result = db.put("key", &vec![1.0; 128], "test");
    
    // Debe fallar gracefully
    assert!(result.is_err());
    
    // DB debe seguir usable
    fail::remove("wal_fsync_error");
    db.put("key2", &vec![2.0; 128], "test2")?;
    
    Ok(())
}
```

## Advantages of Failpoints

| Ventaja | Descripción |
|---------|-------------|
| **Determinismo** | Fallos reproducibles en tests |
| **Cobertura** | Validar paths de error raros |
| **Seguridad** | Sin riesgo de daño real |
| **Velocidad** | Más rápido que chaos testing real |

## Comparison: Failpoints vs Chaos Testing

| Dimensión | Failpoints | Chaos Testing |
|-----------|-----------|---------------|
| **Granularidad** | Línea de código | Sistema completo |
| **Determinismo** | 100% | Variable |
| **Setup** | Bajo (código) | Alto (infraestructura) |
| **Realismo** | Simulado | Real |
| **Caso de uso** | Unit tests | Integration tests |

## See Also

- [[chaos-testing]] — System-level fault testing
- [[wal]] — Primary failpoint user
- [[ci-cd]] — Failpoints in automated tests

---

*Failpoints allow error handling testing without depending on real failures.*

