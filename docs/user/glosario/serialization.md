---
title: Serialization
kind: glossary
status: stable
description: "La serialización es el proceso de convertir estructuras de datos en memoria (structs, objetos) a un formato que pueda ser almacenado en disco o transmitido por red. La deserialización es el proceso inverso: reconstruir las estructuras..."
aliases: [serialization, deserialization, serialize, deserialize, serialización]
tags: [glosario, serializacion, formato, binario, rust]
---


# Serialization

## Definición

La **serialización** es el proceso de convertir estructuras de datos en memoria (structs, objetos) a un formato que pueda ser almacenado en disco o transmitido por red. La **deserialización** es el proceso inverso: reconstruir las estructuras desde el formato serializado.

## En VantaDB

VantaDB utiliza dos enfoques de serialización dependiendo del subsistema:

### 1. Serde + Postcard (WAL y Metadata)

La dupla **[serde](./serde.md)** + **[postcard](./postcard.md)** se usa para serializar registros del WAL, metadatos de índices y estructuras de la SDK:

```rust
// src/text_index.rs
fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    Ok(postcard::to_extend(value, Vec::new())?)
}

fn deserialize<T: for<'de> Deserialize<'de>>(bytes: &[u8], label: &str) -> Result<T> {
    postcard::from_bytes(bytes).map_err(|err| /* … */)
}
```

> **Nota:** el proyecto usó `bincode` 1.3 hasta la migración WEB-04; hoy el
> serializer de disco es **postcard** (`Cargo.toml`: `postcard = "1.1"`). La
> versión del formato de wire la fija `WAL_POSTCARD_VERSION`. El crate `bincode`
> ya no está en el grafo de dependencias.

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
    let header = Header::deserialize(data)?;
    // ... reconstruye índice desde bytes
}
```

### 3. Rkyv (Zero-Copy)

El índice también soporta serialización via **rkyv** para deserialización zero-copy desde [mmap](./mmap.md):

```rust
// src/serialization/rkyv_archives.rs
pub fn serialize_to_rkyv(&self) -> std::io::Result<Vec<u8>> { ... }
```

## Formatos Utilizados

| Formato | Uso | Característica |
|---------|-----|----------------|
| **Postcard** | WAL, metadatos, posting lists | Compacto, sin field names |
| **Custom binary** | HNSW index en disco | Optimizado para mmap |
| **Rkyv** | Zero-copy deserialization | Acceso directo desde mmap |
| **JSON** | API HTTP (Serde) | Legible, intercambio |

## Véase También

- [serde](./serde.md) — Framework de serialización Rust
- [bincode](./bincode.md) — Serializer legacy (sustituido por postcard)
- [mmap](./mmap.md) — Memory-mapped I/O para zero-copy
- [wal](./wal.md) — Write-Ahead Log (consumidor de serialización)
- [hnsw](./hnsw.md) — Índice con serialización personalizada
- [zero-copy](./zero-copy.md) — Técnica de deserialización sin copia
