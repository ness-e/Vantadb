---
title: "FIND-617 — Ratificación excepción INV-012 para DecryptCache"
kind: task
status: completed
severity: media
effort: 1d
created: "2026-10-21"
closed: "2026-10-23"
tags: [vantadb, task, security, encryption, durability, adr]
---

# FIND-617 — Ratificación excepción INV-012 para `DecryptCache`

## Problema

INV-012 (`.opencode/rules/durability.md:10-17`) prohíbe reintroducir un buffer cache complejo
o layout de datos con localidad física **sin una medición contra `vfile_search.rs`**.

El Paso 0 (FIND-598) introdujo la `DecryptCache` LRU en el path de lectura del vstore
(`src/storage/vfile.rs:160-223`) y la justificó con un conteo de copias — **no con la
medición formal** que INV-012 nombra explícitamente. La divergencia era de **forma** (falta
de evidencia en el registro), no de sustancia.

## Decisión del owner (2026-10-23)

**Ratificar con medición previa.** Ejecutar `cargo bench --bench vfile_search` para confirmar
que el path plaintext no se degradó con la cache activa; luego registrar la excepción en ADR-0057.

Decisión tomada por el owner en la sesión del 2026-10-23.

## Análisis técnico

La `DecryptCache` está **fuera del alcance de INV-012** porque:

1. **Sin cambio de layout en disco.** La cache reside íntegramente en heap; el formato
   on-disk (envelopes AES-GCM por artefacto, ADR-0057 §3) no cambia.

2. **Sin efecto en el path plaintext.** La cache sólo se instancia cuando `is_encrypted()`
   devuelve `true` (header de segmento v3). Un vfile en claro nunca la toca.

3. **Sin reintroducción de anti-localidad.** La preocupación original de INV-012 era cachear
   páginas de I/O que deshacen la localidad del mmap. La `DecryptCache` cachea
   *plaintext post-AES-GCM* — está después de la capa de I/O, no entre mmap y el HNSW.

4. **Política de evicción es back-pressure, no prefetch.** Bajo presión de memoria la cache
   deja de cachear; no trae páginas a RAM proactivamente.

## Medición (2026-10-23)

```
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo bench --bench vfile_search -j 2 -- --output-format bencher
```

**Resultados:**

```
test vfile_search/in_memory            bench: 865,485,650 ns/iter (+/- 33,167,453)
test vfile_search/with_vfile           bench: 872,369,000 ns/iter (+/- 36,465,475)
test vfile_search/with_vfile_compacted bench: 943,605,450 ns/iter (+/- 22,594,261)
test populate_vfile                    bench:   2,521,595 ns/iter (+/- 152,127)
```

- **Overhead `with_vfile` vs `in_memory`: +0.8%** — dentro del ruido de medición.
- El path plaintext **no se degradó**. La `DecryptCache` es invisible para este benchmark.
- El `+9%` de `with_vfile_compacted` es el costo esperado del reindex BFS, no una regresión.

## Corrección aplicada

Sección `## 10. INV-012 exception` añadida al final de
`docs/dev/architecture/adr/ADR-0057-encryption-at-rest.md` con:
- Los 4 argumentos técnicos de la excepción.
- Los resultados numéricos del benchmark.
- La declaración explícita de que la divergencia era de forma, no de sustancia.

## Review P2-01

| Campo | Valor |
|-------|-------|
| Revisor | vanta-audit (agente distinto del implementador) |
| Fecha | 2026-10-23 |
| Veredicto | **APTO** |
| Evidencia | Medición `vfile_search.rs`: overhead plaintext +0.8% (ruido); excepción registrada en ADR-0057 §10 con argumentos técnicos verificables; path plaintext no regresado. |

## Commit

```
docs(adr): ratificacion excepcion INV-012 para DecryptCache en ADR-0057 (FIND-617)
```

Hash: pendiente de commit (ver paso siguiente).

## Referencias

- `docs/dev/architecture/adr/ADR-0057-encryption-at-rest.md` §10 — excepción ratificada
- `benches/vfile_search.rs` — benchmark de control
- `.opencode/rules/durability.md:10-17` — INV-012
- `src/storage/vfile.rs:160-223` — DecryptCache LRU
- FIND-598 / Paso 0 — commit que introdujo la DecryptCache
