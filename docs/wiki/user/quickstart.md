---
title: "Quickstart de VantaDB"
kind: howto
status: active
description: "Primer arranque con VantaDB: qué instalar según el runtime y dónde seguir, sin duplicar el quickstart canónico."
---

# Quickstart de VantaDB

La ruta canónica y verificada es [docs/user/QUICKSTART.md](../../user/QUICKSTART.md); esta página es la orientación mínima.

## Elegir artefacto

`[HECHO]` Artefactos 0.9.0 con metadata verificada en registry el 2026-11-06: `pip install vantadb-py` ([PyPI](https://pypi.org/pypi/vantadb-py/json); Python ≥3.11), `npm install vantadb` ([npm](https://registry.npmjs.org/vantadb); wrapper TS sobre WASM, Node ≥22.19), `npm install vantadb-node` ([npm](https://registry.npmjs.org/vantadb-node)) y el crate `vantadb` ([crates.io](https://crates.io/api/v1/crates/vantadb)). El CLI `vanta-cli` se instala vía `scripts/install.sh|ps1`. `[INFERENCIA]` Metadata verificada no equivale a instalación probada por plataforma. `[HECHO]` `vanta-proxy` está congelado (sin canal de distribución).

## Operaciones mínimas

`[HECHO]` El contrato transversal: abrir cliente → `put` (namespace+key+payload+metadata+vector opcional) → `search` híbrido (vector y/o texto, `top_k`) → `memory.get`/`list`/`delete` → `flush`/`close`. Nombres por lenguaje en [api/](../api/rust.md) (ejemplos **no ejecutados** allí).

## Siguientes pasos

[runtime-selection](runtime-selection.md) para elegir transporte · [hybrid-search](hybrid-search.md) para parámetros de búsqueda · [persistence](persistence.md) para durabilidad · [operations](operations.md) para mantenimiento · [troubleshooting](troubleshooting.md) si algo falla.

## Fuentes

[QUICKSTART](../../user/QUICKSTART.md) · [getting-started](../../user/guide/getting-started.md) · [tutorials](../../user/tutorials/index.md) · [README](../../../README.md)
