---
title: "FIND-674: harness de MCP Server / Code Mode entregado - sin corrida, la fila NO se cierra"
kind: task
status: active
description: "benchmarks/mcp_profile_bench.py mide bytes/tokens de tools/list por perfil con una captura, y la latencia JSON-RPC. Self-check offline verde. La corrida necesita vantadb-mcp corriendo y el workspace no compila por WIP ajeno."
id: FIND-674
type: feature
---

# FIND-674 — Benchmark estándar de MCP Server y Code Mode

- **Prioridad:** 🟠 Alta · **Origen:** *ProMCP* (ACL Findings 2026), *MCP-Universe* (`arXiv:2508.14704`)
- **Creado:** 2026-11-22 · **Harness entregado:** 2026-11-27
- **Estado:** 🟡 **HARNESS ENTREGADO, CORRIDA PENDIENTE.** La fila **no se cierra**:
  sin corrida no hay número, y sin número no hay benchmark (es exactamente el
  defecto que `FIND-664` documenta).

## Qué se entregó

`benchmarks/mcp_profile_bench.py` (sin dependencias nuevas, `ruff` limpio):

1. **Coste de inyección de esquema** (`tools/list`) por perfil: bytes exactos,
   caracteres y tokens estimados a 4 B/token (la constante está declarada y
   etiquetada como estimación en cada impresión). Consume una captura por perfil
   —`--tools-json full=… brain=… code=…`— y publica la reducción con su
   aritmética (`1 - perfil/full`), que es lo que la fila llama «cuantificar la
   reducción de contexto».
2. **Latencia JSON-RPC p50/p95/p99** de `context_assemble`, `memory_recall` y
   `memory_put` por stdio y HTTP (`--stdio-cmd`, ganchos de HTTP).
3. **Overhead de wire de `vanta-proxy`** (`WriteBack::track`): exige dos capturas
   (proxied vs directo) y publica el porcentaje.

**Criterio fijado antes de ver números** (queda en el JSON como `criterion`):
reducción de esquema publicada con su aritmética; dispatch por `code` **< 2×** el
p50 de la llamada directa y su p99 en el mismo orden; overhead de proxy **< 5 %**
de los bytes de respuesta.

## Qué se verificó (y qué no)

```
$ python benchmarks/mcp_profile_bench.py --self-check
self-check OK
  88 tools -> 9495 B (~2374 tok)
  16 tools -> 1719 B (~430 tok)  reduction 81.90%
   1 tool  ->  108 B (~27 tok)  reduction 98.86%     exit 0

$ python benchmarks/mcp_profile_bench.py            # sin capturas ni servidor
FIND-674: nothing to measure. ... No number is invented.              exit 2
```

El self-check valida **la contabilidad** (los tres perfiles ordenan por tamaño, la
reducción es monótona, el resumen de percentiles es el documentado) sobre un
fixture sintético. **No** es una medición del servidor: los 88 tools del fixture
son sintéticos, no el catálogo real de `vantadb-mcp/src/handlers/tools.rs`.

## Por qué no hay corrida

El catálogo y los perfiles viven en Rust (`vantadb-mcp/src/handlers/tools.rs`:
`full` 88 tools, `brain` 16, `code`/`vanta_dispatch`), así que sólo el servidor
corriendo puede renderizar el `tools/list` de un perfil. Correrlo exige
`cargo build -p vantadb-mcp`, y en este momento **el workspace no compila por WIP
de otro carril**:

```
error[E0425]: cannot find type `Path` in this scope  --> src\storage\vfile_crypto.rs:377
error[E0382]: borrow of moved value: `path`          --> src\wal.rs:920
```

Además la fila apunta a `vanta-proxy/`, que el árbol tiene con `publish = false` y
fuera de `default-members`. Medir un árbol que no compila sería publicar el WIP
ajeno.

## Lo que falta para cerrarla

1. Un árbol que compile, y `cargo build -p vantadb-mcp` (declarar el tiempo).
2. Capturar `tools/list` en los tres perfiles (`VANTADB_MCP_PROFILE=full|brain|code`)
   y pasarlos por `--tools-json`.
3. Implementar el cliente stdio/HTTP del punto (2) y las dos capturas de proxy del
   punto (3) — hoy son ganchos declarados, no código.
4. Publicar la tabla en `docs/user/operations/BENCHMARKS.md` (fuera de la zona de
   este lote).

## Impacto mapeado (Regla 0)

- **Leídos:** `vantadb-mcp/src/handlers/resources.rs:82-83` (los perfiles `brain`
  16 / `full` 88 y `vanta_dispatch`), `vantadb-mcp/src/handlers/tools.rs`
  (catálogo y `vanta_dispatch`), `vantadb-mcp/src/handlers/initialize.rs:23`.
- **Referencias hacia dentro:** ninguna — archivo nuevo bajo `benchmarks/`.
- **Veredicto:** sólo se añade el harness. **Cero cambios en `vantadb-mcp/**`,
  `vanta-proxy/**` ni `src/**`.**

## Review (P2-01)

Tier: **Fast**. Verify: `--self-check` exit 0 (contabilidad), `--help`/sin
argumentos exit 2 nombrando lo que falta, `ruff check` + `ruff format --check`
limpios. **La fila NO se marca completada**: no hay corrida.
