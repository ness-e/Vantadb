---
title: "Anexo Técnico Forense: Verificación de Aseveraciones Externas"
kind: review
status: active
description: "Contraste forense y verificación línea por línea de claims externos frente a la evidencia del repositorio."
tags: [vantadb, review, forensic, audit, claims]
---

# Anexo Técnico Forense: Verificación de Aseveraciones Externas

## Alcance y método

**[Hecho Comprobado]** Contraste realizado el 2026-11-06 sobre `develop`, commit
`0512116e6db9867c731b8881181cc40a8840906d`. La fecha del nombre de este archivo
identifica el entregable solicitado, no una verificación retrospectiva al 3 de
noviembre. Las referencias de código corresponden a ese commit.

**[Hecho Comprobado]** Las cinco formulaciones siguientes proceden del encargo de
verificación. No se aportó una fuente externa primaria que permita atribuirlas
como citas literales a un auditor concreto. Este anexo contrasta su contenido;
no inventa autoría, URL ni fecha de publicación externa.

**[Hecho Comprobado]** Se inspeccionaron implementación, documentación canónica,
historial Git y metadata pública de npm. La inspección estática demuestra
existencia y mecanismos; no sustituye pruebas de crash, auditoría jurídica,
instalación por plataforma ni una comparación de rendimiento.

### Clasificación epistémica

- **Hecho Comprobado:** observación directa, con referencia verificable.
- **Inferencia Lógica:** conclusión derivada, limitada por las premisas indicadas.
- **Afirmación Refutada:** la formulación concreta contradice evidencia positiva;
  no implica que una afirmación más amplia quede también refutada.

## Matriz de contraste

| # | Formulación sometida a contraste | Evidencia primaria | Dictamen y límite |
|---|---|---|---|
| 1 | «VantaDB no soporta borrado real ni derecho al olvido». | [`ForgetMode::Certified` y despacho de `forget`](../../../src/sdk/api/memory.rs#L1668-L1773), [`delete_certified`](../../../src/sdk/api/memory.rs#L1597-L1613), [contrato de borrado certificado](../../api/CERTIFIED_DELETE.md). | **Afirmación Refutada**, respecto a la inexistencia de borrado y certificado. **No queda demostrada** una garantía de secure erase ni conformidad jurídica GDPR integral. |
| 2 | «El motor requiere un servicio o demonio HTTP externo para operar». | [`Embedded::open`](../../../src/sdk/builder.rs#L120-L126), [`open_with_config` abre `StorageEngine` directamente](../../../src/sdk/builder.rs#L188-L214), [operación `put`](../../../src/sdk/api/memory.rs#L1045-L1075). | **Afirmación Refutada** como requisito del motor embebido. Los servidores y proveedores de embeddings que se elijan pueden usar procesos o red. |
| 3 | «El binding de Node.js no existe o está despublicado». | [implementación napi-rs](../../../vantadb-node/src/lib.rs#L1-L37), [dependencias y `cdylib`](../../../vantadb-node/Cargo.toml#L15-L30), [metadata de `vantadb-node@0.9.0` en npm](https://registry.npmjs.org/vantadb-node/0.9.0). | **Afirmación Refutada** para la existencia del binding y disponibilidad de la versión 0.9.0 en la consulta. No certifica instalación, cobertura funcional completa ni disponibilidad futura. |
| 4 | «La búsqueda híbrida es un simple plugin externo sin fusión real». | [ejecución híbrida nativa](../../../src/sdk/search/hybrid.rs#L13-L54), [fusión de ambas listas](../../../src/sdk/search/fusion.rs#L112-L156), [contribución RRF por rango](../../../src/sdk/search/fusion.rs#L212-L234). | **Afirmación Refutada**: existe implementación Rust de fusión. No demuestra superioridad de recall, latencia ni equivalencia entre todas las rutas de búsqueda. |
| 5 | «El almacenamiento es susceptible a corrupción silenciosa ante crash». | [formatos WAL](../../../src/wal.rs#L11-L55), [CRC32C y deserialización](../../../src/wal.rs#L1457-L1472), [políticas de sync](../../../src/wal.rs#L999-L1028), [recovery transaccional](../../../src/storage/engine/init.rs#L617-L707). | **No refutada en su amplitud**. Hay controles comprobables, pero su existencia no prueba ausencia universal de pérdida, corrupción o errores de recuperación. |

## 1. Borrado certificado: capacidad implementada, perímetro explícito

**[Hecho Comprobado]** `forget` admite `Soft`, `Hard` y `Certified`. El último
delega en `delete_certified`, que ejecuta el borrado compartido y construye un
`PurgeCertificate`. Su inventario incluye superficies, residuos observados y
límites declarados; puede indicar `purged`, `residues` o `not_found`, en lugar de
declarar éxito incondicional.

**[Hecho Comprobado]** El
[borrado del SDK](../../../src/sdk/api/memory.rs#L1540-L1569) llama a
`StorageEngine::delete`. El motor registra el tombstone WAL y ejecuta
[`backend.delete` sobre la partición de datos](../../../src/storage/engine/delete.rs#L35-L55);
además, marca el vector como tombstone y lo retira de HNSW y caché en
[`apply_delete_inner`](../../../src/storage/engine/delete.rs#L128-L156).
Es eliminación de entradas vivas, no sobreescritura certificada del medio físico.

**[Hecho Comprobado]** El recibo se serializa con un campo de integridad SHA-256.
El hash se calcula sobre el cuerpo canónico con el propio digest vaciado:
[`canonical_bytes`](../../../src/attestation.rs#L362-L368),
[`finalize`](../../../src/attestation.rs#L438-L443) y
[`build_certificate`](../../../src/attestation.rs#L451-L485).

**[Inferencia Lógica]** Esto refuta «no existe borrado certificado», pero no
«todo rastro físico desaparece». El
[contrato canónico](../../api/CERTIFIED_DELETE.md#scope-and-limits-declared-not-silent)
excluye medios físicos, backups, snapshots externos, segmentos WAL archivados,
exports, retención de logs y unlearning de pesos. Un hash de integridad no es una
firma de autenticidad del emisor ni una certificación de cumplimiento GDPR.

**[Hecho Comprobado]** Para verificar desde otro proceso debe persistirse la
purga mediante `flush()` o `close()`; el
[SDK lo documenta expresamente](../../../src/sdk/api/memory.rs#L1590-L1595).

## 2. Motor embebido frente a transportes opcionales

**[Hecho Comprobado]** La apertura del SDK instancia `StorageEngine` y mantiene
su handle dentro de `Embedded`; no delega la apertura en un cliente HTTP.
`put` entra en la implementación local del SDK. Esto acredita un camino de
almacenamiento in-process, no la necesidad de levantar un servicio.

**[Inferencia Lógica]** «No requiere un demonio HTTP» no significa «ningún uso de
VantaDB puede abrir sockets». El servidor HTTP, el proceso MCP y proveedores como
Ollama u OpenAI son superficies/configuraciones distintas. Incluso con transporte
MCP stdio puede existir un listener HTTP loopback interno; no debe confundirse
el transporte del cliente con el proceso que posee el writer.

## 3. Node.js: fuente, publicación y alcance del gate

**[Hecho Comprobado]** La consulta de npm del 2026-11-06 devolvió metadata para
`name=vantadb-node`, `version=0.9.0`, con artefacto `dist.tarball`, integridad y
exports CommonJS/ESM. La fuente declara dependencias `napi` y `napi-derive`.
`publish=false` en el Cargo manifest afecta a crates.io, no refuta una publicación
del paquete npm.

**[Hecho Comprobado]** El
[conciliador documental](../../../scripts/docs/verify-surface-manifest.mjs#L21-L84)
ejecuta el generador en modo check y comprueba patrones de estado `PUBLIC` en el
manifiesto local. **No consulta el registro npm**: su mensaje final de
conciliación no constituye, por sí solo, evidencia independiente de publicación.
La consulta al endpoint público anterior aporta esa evidencia separada.

**[Inferencia Lógica]** Existencia y publicación no equivalen a paridad completa
con Python/WASM, éxito de instalación en todas las plataformas declaradas o
ausencia de errores de lifecycle. Esas cuestiones necesitan gates específicos.

## 4. Fusión híbrida implementada en Rust

**[Hecho Comprobado]** La ruta híbrida obtiene candidatos léxicos y vectoriales.
La implementación de fusión acumula por identidad `(namespace, key)` una
contribución `weight / (rrf_k + rank + 1)` y ordena los resultados. Las dos listas
se consumen en el mismo módulo Rust; no se requiere un plugin de fusión externo.

**[Inferencia Lógica]** La evidencia refuta la ausencia de fusión, pero no hace
que cualquier llamada sea híbrida automáticamente: se necesitan las modalidades
de entrada correspondientes y vectores de un espacio de embeddings coherente.
Los scores RRF no son probabilidades calibradas de relevancia.

## 5. Crash consistency: controles no equivalen a una prueba universal

**[Hecho Comprobado]** El formato corriente sin cifrar es
`WAL_FORMAT_VERSION = 3`. La variante cifrada usa
`WAL_ENCRYPTED_FORMAT_VERSION = 4`, cuya lectura depende de la feature
`encryption`. Por tanto, «todo el almacenamiento usa WAL v4» no describe el
código inspeccionado.

**[Hecho Comprobado]** `maybe_sync` distingue `Always`, `Periodic` y `Never`.
`Periodic` dispara sync por un umbral de registros; no es una garantía temporal
universal de fsync cada cierto número de milisegundos. `sync` vacía el buffer y
llama a `sync_data()`. `Never` no hace sync automático. Agrupar escrituras no
elimina estas distinciones ni convierte cualquier confirmación en durable.

**[Hecho Comprobado]** Hay comparación CRC32C, validación de frames y un recovery
que reconoce `Begin`, `Commit` y `Abort`. Esto prueba mecanismos de detección y
recuperación, no que cubran todos los estados de un crash.

**[Hecho Comprobado]** En el protocolo inspeccionado se aplican operaciones antes
de escribir el marcador final `Commit`:
[`txn.rs:295–382`](../../../src/storage/engine/txn.rs#L295-L382).
El backlog local mantiene `CRASH-01` y `TXN-01` abiertos para investigar/corregir
replay de segmentos rotados y estados transaccionales interrumpidos.

**[Inferencia Lógica]** Un frame perdido que nunca se reproduce puede no presentar
un CRC incorrecto; una mutación del backend sin commit durable tampoco se resuelve
por comprobar CRC. Las reproducciones necesarias deben forzar rotación,
checkpoint y crashes antes/después del punto de commit con un oracle externo de
registros confirmados. Este paso documental no ejecutó esas reproducciones ni
demostró pérdida de datos.

**Dictamen:** no se declara este claim «refutado». Sí queda refutada una
formulación más estrecha: «el WAL carece de mecanismos de detección de corrupción».
La aceptación de producción requiere evidencia adicional de crash consistency.

## Trazabilidad Git

**[Hecho Comprobado]** Último commit que toca cada archivo en el snapshot
inspeccionado; no se presenta como fecha de introducción de toda su funcionalidad:

| Archivo | Commit | Fecha Git |
|---|---|---|
| `src/attestation.rs` | `3b06218fb` | 2026-10-06 |
| `src/sdk/api/memory.rs` | `0488c7dc8` | 2026-11-03 |
| `src/sdk/search/fusion.rs` | `928df0970` | 2026-10-31 |
| `src/wal.rs` | `928df0970` | 2026-10-31 |
| `vantadb-node/package.json` | `679636f28` | 2026-10-19 |
| `scripts/docs/verify-surface-manifest.mjs` | `0512116e6` | 2026-11-06 |

```bash
git show 0512116e6:src/wal.rs
git show 0512116e6:src/sdk/search/fusion.rs
git show 0512116e6:src/sdk/api/memory.rs
git log -1 --format="%h %cs %s" 0512116e6 -- src/attestation.rs
git log -1 --format="%h %cs %s" 0512116e6 -- scripts/docs/verify-surface-manifest.mjs
```

**[Inferencia Lógica]** La conclusión defendible es acotada: existen borrado con
recibo verificable, operación embebida, binding Node publicado y fusión RRF
nativa. No se deriva de ello borrado físico universal, cumplimiento jurídico
automático, compatibilidad total entre runtimes ni inmunidad ante crash.
