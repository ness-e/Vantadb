---
title: "crc32c"
type: glossary-entry
status: stable
tags: [integridad, checksum, hash, crc]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Cyclic Redundancy Check, CRC32 Castagnoli]
description: "Checksum algorithm that produces a 32-bit hash using the Castagnoli polynomial, used to detect hardware-supported data corruption on modern CPUs"
---
#CRC32C—Cyclic Redundancy Check (Castagnoli)

##Definition

**CRC32C** is a **checksum** algorithm that produces a 32-bit hash using the Castagnoli polynomial. It is used to **detect data corruption** in storage and transmission, with hardware support on modern CPUs (SSE4.2, ARM CRC).

## How It Works

### Basic Concept

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

Reading:
data, stored_checksum = read_from_disk()
computed_checksum = CRC32C(data)

if computed_checksum == stored_checksum:
    # ✅ Complete data
else:
    # ❌ Corrupt data
```

## Why CRC32C (and not MD5, SHA-256)

| Algoritmo | Velocidad | Detección de Errores | Caso de Uso |
|-----------|-----------|---------------------|-------------|
| **CRC32C** | **~10 GB/s** (hardware) | Excelente (errores de transmisión) | Storage, redes |
| **CRC32** | ~10 GB/s (hardware) | Buena | Legacy (Ethernet, ZIP) |
| **MD5** | ~0.5 GB/s | Excelente (colisiones posibles) | Legacy checksums |
| **SHA-256** | ~0.3 GB/s | Criptográfica | Seguridad |

### Advantages of CRC32C

1. **Hardware-accelerated:** Modern CPUs have dedicated instructions
2. **Fast:** 10-50x faster than MD5/SHA
3. **Robust Detection:** Detects all 1-3 bit errors
4. **Low overhead:** Only 4 bytes per record

## Usage in VantaDB

### Integrity of [[wal]]

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

### Writing Flow in WAL

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

### Recovery Flow

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

## Hardware-Accelerated Implementation

### Rust: crc32c crate

```rust
use crc32c::crc32c;

let data = b"Hello, VantaDB!";
let checksum = crc32c(data);  // Use SSE4.2 if available

// Performance:
// - Con SSE4.2: ~10 GB/s
// - Sin SSE4.2 (fallback): ~1 GB/s
```

### CPU Features Detection

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
    
    // software fallback
    crc32c_sw(data)
}
```

## Error Detection Capability

### Types of Errors Detected

| Tipo de Error | Detección |
|--------------|-----------|
| **1 bit flip** | 100% |
| **2 bits flip** | 100% |
| **3 bits flip** | 100% |
| **Burst error (≤32 bits)** | 100% |
| **Burst error (>32 bits)** | 99.99999998% |

### Collision Probability

For random data, the probability that two different messages produce the same CRC32C is:

$$
P(\text{collision}) \approx \frac{1}{2^{32}} \approx 2.3 \times 10^{-10}
$$

**Interpretation:** You would need ~4 billion messages to wait for a collision.

## Known Issues

### AUD-02: WAL without Checksums

**Severity:** 🔒 Blocking

**Description:** The VantaDB snapshot does not show that every record in the WAL has CRC32C.

**Impact:** 
- Cannot detect data corruption
- Recovery can apply corrupted records
- Silent loss of integrity

**Mitigación Requerida:**
```rust
// Cada registro debe tener checksum
pub struct WalRecord {
    pub length: u32,
    pub payload: Vec<u8>,
    pub checksum: u32,  // CRC32C(payload)
}

// Recovery must verify checksum
fn replay(&mut self) -> Result<()> {
    for record in self.read_records() {
        if !record.verify() {
            // Truncate at this point
            self.truncate()?;
            break;
        }
        self.apply(&record)?;
    }
    Ok(())
}
```

### Issue: Corruption vs Truncation

| Escenario | Causa | Detección | Acción |
|-----------|-------|-----------|--------|
| **Truncation** | Crash durante write | EOF prematuro | Recuperar hasta último registro completo |
| **Corruption** | Bit flip en disco | CRC32C no coincide | Truncar en registro corrupto |
| **Partial write** | Crash a mitad de write | Longitud incorrecta | Truncar en registro incompleto |

## Comparison with Other Checksums

| Algoritmo | Bits | Velocidad | Detección | Uso en DBs |
|-----------|------|-----------|-----------|------------|
| **CRC32C** | 32 | ~10 GB/s | Excelente | PostgreSQL, RocksDB, VantaDB |
| **CRC32** | 32 | ~10 GB/s | Buena | Ethernet, ZIP, PNG |
| **Adler-32** | 32 | ~5 GB/s | Regular | zlib |
| **xxHash** | 64 | ~15 GB/s | Excelente | RapidHash, backups |
| **MD5** | 128 | ~0.5 GB/s | Excelente (colisiones) | Legacy |
| **SHA-256** | 256 | ~0.3 GB/s | Criptográfica | Seguridad |

### Why VantaDB Uses CRC32C

1. **Speed:** 10-50x faster than MD5/SHA
2. **Hardware:** SSE4.2 on x86, ARM CRC extension
3. **Sufficient:** For storage corruption detection, no cryptography is needed
4. **Standard:** Used by PostgreSQL, RocksDB, SQLite

## CRC32C testing

### Corruption Detection Test

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

### Recovery Test with Corruption

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

## See Also

- [[wal]] — System using CRC32C for integrity
- [[fsync]] — Durability complementary to integrity
- [[transactional]] — Property that CRC32C helps ensure
- [[chaos-testing]] — How to validate corruption detection

---

*CRC32C is the first line of defense against silent data corruption.*

