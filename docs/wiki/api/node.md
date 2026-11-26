---
title: "API de Node.js nativo (`vantadb-node`)"
kind: reference
status: active
description: "Binding napi-rs para Node.js: estado de publicación, superficie frente al wrapper TS y diferencias verificadas."
---

# API de Node.js nativo (`vantadb-node`)

Audiencia: usuarios Node que evalúan el binding nativo. Referencia canónica: [NODE_SDK](../../api/NODE_SDK.md).

## Estado

`[HECHO]` El crate `vantadb-node` (napi-rs, [`vantadb-node/src/lib.rs`](../../../vantadb-node/src/lib.rs)) publica el paquete npm `vantadb-node` 0.9.0 — publicación comprobada contra [registry.npmjs.org/vantadb-node](https://registry.npmjs.org/vantadb-node) el 2026-11-06 (`dist-tags.latest = 0.9.0`). `[HECHO]` El [SURFACE_MANIFEST](../../api/SURFACE_MANIFEST.md) está **desactualizado** en este punto: clasifica el paquete como "planeado, nunca publicado"; EXPERIMENTAL queda como clasificación de madurez declarada (canal vs madurez son ejes distintos). `[HECHO]` No es miembro del workspace cargo (ausente de `Cargo.toml:780-792`). La decisión napi-rs está en ADR-0050; el canal de release es `release-npm-node.yml` sobre tags `node-v*`. `[HECHO]` `vantadb-ts` lo consume como dependencia `file:../vantadb-node` — dependencia normal (no `optionalDependencies`) que habilita la ruta nativa alternativa (`vantadb-ts/package.json:63-66`, export `./native`).

## Superficie frente a WASM/TS

`[HECHO]` Comparte la hoja `vantadb-ffi-core` (`OpGate`/`OpGuard` para write-after-close, `clamp_top_k` para ERR-022) junto con los bindings Python y WASM (`vantadb-ffi-core/src/lib.rs:1-42`). `[HECHO]` En la matriz W1/API-02 expone `insertNode({id:string})`, `getNode(id:string)`, `deleteNode(id,reason?)`, `putBatch([{...}])`, `searchMulti(namespaces,request)`, `graphragSearch(...)`; ids `u128` viajan como string decimal y los traversals devuelven `bigint[]` ([BINDINGS_NAMESPACES](../../api/BINDINGS_NAMESPACES.md)).

`[HECHO]` Diferencias declaradas: Node NO expone un método IQL `query` (el flag de `capabilities` reporta el motor, no la superficie — FIND-268, la exclusión del runner de paridad DIST-17) y no expone ANN crudo `searchVector`. `[HECHO]` `MemorySearchHit.score` está documentado como mayor-es-mejor en la matriz de paridad; `[INFERENCIA]` la igualdad completa de shapes con el wrapper TS no se afirma aquí — las diferencias declaradas son las de la tabla de BINDINGS_NAMESPACES.

## Diferencia frente al wrapper TypeScript

`[INFERENCIA]` La distinción práctica: `vantadb` (npm, TS) es la superficie pública documentada que por defecto corre sobre el artefacto WASM; `vantadb-node` es la implementación nativa alternativa — misma semántica de métodos camelCase pero ejecutando el motor nativo, sin las limitaciones wasm32 (fs, tiempo, tamaño de artefacto).

## Fuentes

`vantadb-node/` · `vantadb-ts/src/native.ts` · [NODE_SDK](../../api/NODE_SDK.md) · [BINDINGS_NAMESPACES](../../api/BINDINGS_NAMESPACES.md) · ADR-0050 · [paridad](parity.md)
