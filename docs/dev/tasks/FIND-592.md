---
title: "FIND-592 — Vector canary: bytes IEEE-754 cifrados en vstore_L*.vanta y vector_index.bin"
kind: task
---

# FIND-592 — Vector canary: bytes IEEE-754 cifrados en vstore_L*.vanta y vector_index.bin

- Origen: plan 2026-11-07-consolidacion-tecnica-pre-v0100.md · Task 15
- Estado: COMPLETED · commit ebda11821
- SDP: test-driven-development, security-and-hardening, doubt-driven-development
- Contrato: `cargo nextest run --profile audit -p vantadb --features encryption --test encryption_canary --locked --build-jobs 2`

## Impacto mapeado (Regla 0)
- Archivos leídos: tests/encryption_canary.rs (completo), src/sdk/types/record.rs:184-289 (MemoryInput), src/index/serialize/file.rs:519-608 (imagen HNSW guarda Full vectors + cifrado), src/index/graph/core.rs:73-92 (normas cacheadas, no reescritura del vector), src/sdk/api/memory.rs:944-1094 (ingesta no normaliza).
- Entrantes: ninguno — solo se toca el test.
- Veredicto: blast radius = 1 fichero de test. Sin cambio de producción.

## Steps
- [x] S1: generalizar buscador a &[u8] y añadir vector canario L2-normalizado (16 floats, firma 64B LE)
- [x] S2: control positivo (plaintext): firma SÍ en vstore_L*.vanta y vector_index.bin
- [x] S3: control negativo (cifrado): firma NO en ningún fichero ni snapshot; read-back bit-exact
- [x] S4: RED — aserción negativa temporal contra DB plaintext (debe fallar nombrando fichero), revertir
- [x] S5: contrato + clippy + fmt + review P2-01 + commit local
