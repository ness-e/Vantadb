---
type: glosario-entry
status: stable
tags: [testing, fault-injection, debugging]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Failpoint Injection, Error Injection]
---

# Failpoints

## Definición

**Failpoints** son **puntos de inyección de errores** insertados en el código que permiten simular fallos específicos (I/O errors, timeouts, corruption) de forma controlada durante testing, validando el manejo de errores sin necesidad de fallos reales.

## Cómo Funciona

### Inserción de Failpoint

```rust
use fail::fail_point;

pub fn write_to_disk(&self, data: &[u8]) -> Result<()> {
    // Failpoint: simular error de I/O
    fail_point!("disk_write_error", |_| {
        Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Simulated disk error"
        )))
    });
    
    // Código real
    self.file.write_all(data)?;
    self.file.sync_all()?;
    Ok(())
}
```

### Activación en Tests

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

## Tipos de Failpoints

| Tipo | Descripción | Ejemplo |
|------|-------------|---------|
| **return** | Retornar error inmediatamente | `fail_point!("name", \|_\| Err(...))` |
| **delay** | Introducir latencia | `fail_point!("name", \|_\| sleep(Duration::from_secs(5)))` |
| **panic** | Causar panic | `fail_point!("name", \|_\| panic!("test"))` |
| **corrupt** | Corromper datos | `fail_point!("name", \|data\| corrupt(data))` |

## Failpoints en VantaDB

### Ubicaciones Clave

```rust
// WAL: Error durante append
fail_point!("wal_append_error");

// WAL: Error durante fsync
fail_point!("wal_fsync_error");

// Storage: Error durante compactación
fail_point!("compaction_error");

// Recovery: Corrupción de registro
fail_point!("recovery_corruption");

// HNSW: Error durante rebuild
fail_point!("hnsw_rebuild_error");
```

### Testing con Failpoints

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

## Ventajas de Failpoints

| Ventaja | Descripción |
|---------|-------------|
| **Determinismo** | Fallos reproducibles en tests |
| **Cobertura** | Validar paths de error raros |
| **Seguridad** | Sin riesgo de daño real |
| **Velocidad** | Más rápido que chaos testing real |

## Comparación: Failpoints vs Chaos Testing

| Dimensión | Failpoints | Chaos Testing |
|-----------|-----------|---------------|
| **Granularidad** | Línea de código | Sistema completo |
| **Determinismo** | 100% | Variable |
| **Setup** | Bajo (código) | Alto (infraestructura) |
| **Realismo** | Simulado | Real |
| **Caso de uso** | Unit tests | Integration tests |

## Véase También

- [Chaos Testing](Chaos Testing.md) — Testing de fallos a nivel de sistema
- [WAL](WAL.md) — Principal usuario de failpoints
- [CI_CD](CI_CD.md) — Failpoints en tests automatizados

---

*Failpoints permiten testing de error handling sin depender de fallos reales.*

