---
title: "Serialization"
type: glossary-entry
status: stable
tags: [glosario, serializacion, formato, binario, rust]
last_reviewed: 2026-07-03
aliases: [serialization, deserialization, serialize, deserialize, serialización]
description: "Conversión de estructuras de datos en memoria a un formato binario o textual para almacenamiento o transmisión"
---

# Serialization

## Definición

La **serialización** es el proceso de convertir estructuras de datos en memoria (structs, objetos) a un formato que pueda ser almacenado en disco o transmitido por red. La **deserialización** es el proceso inverso: reconstruir las estructuras desde el formato serializado.

## En VantaDB

VantaDB utiliza dos enfoques de serialización dependiendo del subsistema:

### 1. Serde + Bincode (WAL y Metadata)

La dupla **[[serde]]** + **[[bincode]]** se usa para serializar registros del WAL, metadatos de índices y estructuras de la SDK:

```rust
// src/text_index.rs
fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    bincode::serialize(value)
}

fn deserialize<T: for<'de> Deserialize<'de>>(bytes: &[u8], label: &str) -> Result<T> {
    bincode::deserialize(bytes)
}
```

### 2. Serialización Manual (HNSW Index)

El índice HNSW implementa serialización binaria optimizada a medida para máximo control sobre el layout en disco:

```rust
// src/index/core.rs
pub fn serialize_to_bytes(&self) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&header.serialize());
    // ... escribe nodos, aristas, metadatos
}

pub fn deserialize_from_bytes(data: &[u8], force_copy: bool) -> std::io::Result<Self> {
    let header = VantaHeader::deserialize(data)?;
    // ... reconstruye índice desde bytes
}
```

### 3. Rkyv (Zero-Copy)

El índice también soporta serialización via **rkyv** para deserialización zero-copy desde [[mmap]]:

```rust
// src/serialization/rkyv_archives.rs
pub fn serialize_to_rkyv(&self) -> std::io::Result<Vec<u8>> { ... }
```

## Formatos Utilizados

| Formato | Uso | Característica |
|---------|-----|----------------|
| **Bincode** | WAL, metadatos, posting lists | Compacto, sin field names |
| **Custom binary** | HNSW index en disco | Optimizado para mmap |
| **Rkyv** | Zero-copy deserialization | Acceso directo desde mmap |
| **JSON** | API HTTP (Serde) | Legible, intercambio |

## Véase También

- [[serde]] — Framework de serialización Rust
- [[bincode]] — Formato binario compacto
- [[mmap]] — Memory-mapped I/O para zero-copy
- [[wal]] — Write-Ahead Log (consumidor de serialización)
- [[hnsw]] — Índice con serialización personalizada
- [[zero-copy]] — Técnica de deserialización sin copia
