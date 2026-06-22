---
type: glosario-entry
status: stable
tags: [integridad, checksum, hash, crc]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Cyclic Redundancy Check, CRC32 Castagnoli]
description: "Algoritmo de checksum que produce un hash de 32 bits usando el polinomio de Castagnoli, usado para detectar corrupción de datos con soporte hardware en CPUs modernas"
---

# CRC32C — Cyclic Redundancy Check (Castagnoli)

## Definición

**CRC32C** es un algoritmo de **checksum** (suma de verificación) que produce un hash de 32 bits usando el polinomio de Castagnoli. Se usa para **detectar corrupción de datos** en almacenamiento y transmisión, con soporte hardware en CPUs modernas (SSE4.2, ARM CRC).

## Cómo Funciona

### Concepto Básico

```
Datos: "Hello, VantaDB!"
    │
    ▼
CRC32C(data)
    │
    ▼
Checksum: 0xA3F2C8B1 (32 bits)
```

### Verificación de Integridad

```
Escritura:
data = "Hello, VantaDB!"
checksum = CRC32C(data)  # 0xA3F2C8B1
write_to_disk(data, checksum)

Lectura:
data, stored_checksum = read_from_disk()
computed_checksum = CRC32C(data)

if computed_checksum == stored_checksum:
    # ✅ Datos íntegros
else:
    # ❌ Datos corruptos
```

## Por Qué CRC32C (y no MD5, SHA-256)

| Algoritmo | Velocidad | Detección de Errores | Caso de Uso |
|-----------|-----------|---------------------|-------------|
| **CRC32C** | **~10 GB/s** (hardware) | Excelente (errores de transmisión) | Storage, redes |
| **CRC32** | ~10 GB/s (hardware) | Buena | Legacy (Ethernet, ZIP) |
| **MD5** | ~0.5 GB/s | Excelente (colisiones posibles) | Legacy checksums |
| **SHA-256** | ~0.3 GB/s | Criptográfica | Seguridad |

### Ventajas de CRC32C

1. **Hardware-accelerated:** CPUs modernas tienen instrucciones dedicadas
2. **Rápido:** 10-50x más rápido que MD5/SHA
3. **Detección robusta:** Detecta todos los errores de 1-3 bits
4. **Bajo overhead:** Solo 4 bytes por registro

## Uso en VantaDB

### Integridad del [WAL](WAL.md)

```rust
pub struct WalRecord {
    pub length: u32,
    pub record_type: u8,
    pub payload: Vec<u8>,
    pub checksum: u32,  // CRC32C(payload)
}

impl WalRecord {
    pub fn new(record_type: u8, payload: Vec<u8>) -> Self {
        let checksum = crc32c::crc32c(&payload);
        Self {
            length: payload.len() as u32,
            record_type,
            payload,
            checksum,
        }
    }
    
    pub fn verify(&self) -> bool {
        crc32c::crc32c(&self.payload) == self.checksum
    }
}
```

### Flujo de Escritura en WAL

```rust
impl WalWriter {
    pub fn append(&mut self, mutation: &Mutation) -> Result<()> {
        // 1. Serializar
        let payload = bincode::serialize(mutation)?;
        
        // 2. Calcular CRC32C
        let checksum = crc32c::crc32c(&payload);
        
        // 3. Crear registro
        let record = WalRecord {
            length: payload.len() as u32,
            record_type: RecordType::Mutation as u8,
            payload,
            checksum,
        };
        
        // 4. Escribir a disco
        self.file.write_all(&record.serialize())?;
        
        // 5. fsync para durabilidad
        self.file.sync_all()?;
        
        Ok(())
    }
}
```

### Flujo de Recovery

```rust
impl WalReader {
    pub fn replay(&mut self) -> Result<Vec<Mutation>> {
        let mut mutations = Vec::new();
        
        loop {
            match self.read_record() {
                Ok(record) => {
                    // Verificar integridad
                    if !record.verify() {
                        // ❌ Checksum no coincide
                        // Truncar WAL aquí (datos corruptos)
                        self.truncate_at_current_position()?;
                        warn!("WAL corruption detected, truncating");
                        break;
                    }
                    
                    // ✅ Checksum válido
                    let mutation: Mutation = bincode::deserialize(&record.payload)?;
                    mutations.push(mutation);
                }
                Err(Error::UnexpectedEof) => {
                    // Fin del WAL (normal)
                    break;
                }
                Err(e) => return Err(e),
            }
        }
        
        Ok(mutations)
    }
}
```

## Implementación Hardware-Accelerated

### Rust: crc32c crate

```rust
use crc32c::crc32c;

let data = b"Hello, VantaDB!";
let checksum = crc32c(data);  // Usa SSE4.2 si disponible

// Performance:
// - Con SSE4.2: ~10 GB/s
// - Sin SSE4.2 (fallback): ~1 GB/s
```

### Detección de Features CPU

```rust
#[cfg(target_arch = "x86_64")]
fn has_sse42() -> bool {
    is_x86_feature_detected!("sse4.2")
}

fn crc32c_hw(data: &[u8]) -> u32 {
    #[cfg(target_arch = "x86_64")]
    if has_sse42() {
        return unsafe { crc32c_sse42(data) };
    }
    
    // Fallback software
    crc32c_sw(data)
}
```

## Capacidad de Detección de Errores

### Tipos de Errores Detectados

| Tipo de Error | Detección |
|--------------|-----------|
| **1 bit flip** | 100% |
| **2 bits flip** | 100% |
| **3 bits flip** | 100% |
| **Burst error (≤32 bits)** | 100% |
| **Burst error (>32 bits)** | 99.99999998% |

### Probabilidad de Colisión

Para datos aleatorios, la probabilidad de que dos mensajes diferentes produzcan el mismo CRC32C es:

$$
P(\text{collision}) \approx \frac{1}{2^{32}} \approx 2.3 \times 10^{-10}
$$

**Interpretación:** Necesitarías ~4 mil millones de mensajes para esperar una colisión.

## Problemas Conocidos

### AUD-02: WAL sin Checksums

**Severidad:** 🔒 Bloqueante

**Descripción:** El snapshot de VantaDB no demuestra que cada registro del WAL tenga CRC32C.

**Impacto:** 
- No se puede detectar corrupción de datos
- Recovery puede aplicar registros corruptos
- Pérdida silenciosa de integridad

**Mitigación Requerida:**
```rust
// Cada registro debe tener checksum
pub struct WalRecord {
    pub length: u32,
    pub payload: Vec<u8>,
    pub checksum: u32,  // CRC32C(payload)
}

// Recovery debe verificar checksum
fn replay(&mut self) -> Result<()> {
    for record in self.read_records() {
        if !record.verify() {
            // Truncar en este punto
            self.truncate()?;
            break;
        }
        self.apply(&record)?;
    }
    Ok(())
}
```

### Problema: Corruption vs Truncation

| Escenario | Causa | Detección | Acción |
|-----------|-------|-----------|--------|
| **Truncation** | Crash durante write | EOF prematuro | Recuperar hasta último registro completo |
| **Corruption** | Bit flip en disco | CRC32C no coincide | Truncar en registro corrupto |
| **Partial write** | Crash a mitad de write | Longitud incorrecta | Truncar en registro incompleto |

## Comparación con Otros Checksums

| Algoritmo | Bits | Velocidad | Detección | Uso en DBs |
|-----------|------|-----------|-----------|------------|
| **CRC32C** | 32 | ~10 GB/s | Excelente | PostgreSQL, RocksDB, VantaDB |
| **CRC32** | 32 | ~10 GB/s | Buena | Ethernet, ZIP, PNG |
| **Adler-32** | 32 | ~5 GB/s | Regular | zlib |
| **xxHash** | 64 | ~15 GB/s | Excelente | RapidHash, backups |
| **MD5** | 128 | ~0.5 GB/s | Excelente (colisiones) | Legacy |
| **SHA-256** | 256 | ~0.3 GB/s | Criptográfica | Seguridad |

### Por Qué VantaDB Usa CRC32C

1. **Velocidad:** 10-50x más rápido que MD5/SHA
2. **Hardware:** SSE4.2 en x86, ARM CRC extension
3. **Suficiente:** Para detección de corrupción de storage, no se necesita criptografía
4. **Estándar:** Usado por PostgreSQL, RocksDB, SQLite

## Testing de CRC32C

### Test de Detección de Corrupción

```rust
#[test]
fn test_crc32c_detects_corruption() {
    let data = b"Hello, VantaDB!";
    let checksum = crc32c::crc32c(data);
    
    // Corromper 1 bit
    let mut corrupted = data.to_vec();
    corrupted[5] ^= 0x01;  // Flip bit
    
    // CRC32C debe detectar
    let corrupted_checksum = crc32c::crc32c(&corrupted);
    assert_ne!(checksum, corrupted_checksum);
}
```

### Test de Recovery con Corrupción

```rust
#[test]
fn test_wal_recovery_with_corruption() {
    let mut wal = WalWriter::create("test.wal")?;
    
    // Escribir 3 registros válidos
    wal.append(&mutation1)?;
    wal.append(&mutation2)?;
    wal.append(&mutation3)?;
    
    // Corromper el tercer registro
    corrupt_file_at_offset("test.wal", offset_of_record3)?;
    
    // Recovery debe recuperar solo los primeros 2
    let mut reader = WalReader::open("test.wal")?;
    let mutations = reader.replay()?;
    
    assert_eq!(mutations.len(), 2);
    assert_eq!(mutations[0], mutation1);
    assert_eq!(mutations[1], mutation2);
}
```

## Véase También

- [WAL](WAL.md) — Sistema que usa CRC32C para integridad
- [fsync](fsync.md) — Durabilidad complementaria a integridad
- [Transaccional](Transaccional.md) — Propiedad que CRC32C ayuda a garantizar
- [Chaos Testing](Chaos Testing.md) — Cómo validar detección de corrupción

---

*CRC32C es la primera línea de defensa contra corrupción silenciosa de datos.*

