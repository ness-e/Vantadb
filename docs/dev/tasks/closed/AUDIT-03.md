---
title: "AUDIT-03: API Unificada Embedded::forget() (Soft, Hard, Certified)"
kind: task
status: completed
created_at: "2026-10-31T06:00:00Z"
updated_at: "2026-10-31T06:00:00Z"
owner: "vanta-lead"
category: "api-governance"
tags: [api, forget, certified-delete, rust, python, mcp]
description: "API Unificada Embedded::forget() (Soft, Hard, Certified) en Rust, Python y MCP."
---

# AUDIT-03: API Unificada Embedded::forget() (Soft, Hard, Certified)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-31-audit-remediation-master.md`
- **Fuente:** Hallazgos FIND-504, VER-02, revisión multi-lenguaje
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🔴 Alta
- **Tipo:** Multi-lenguaje (Rust SDK, Python SDK, MCP)
- **Turns estimados:** 12-16
- **Creado:** 2026-10-31T06:00:00Z
- **last-synced:** 2026-10-31T06:00:00Z
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 4

## Objetivo

Unificar y formalizar el ciclo de vida de olvido y eliminación de memoria bajo una API canónica `Embedded::forget()` disponible de forma homogénea en el crate Rust principal, en los bindings de Python y en el protocolo MCP. La API debe admitir tres modalidades estrictas:
1. `ForgetMode::Soft`: Inserción de lápida (tombstone) lógica que oculta el registro de búsquedas y consultas sin alterar el almacenamiento físico inmediato.
2. `ForgetMode::Hard`: Desindexación completa de HNSW y BM25, supresión del vector y sobreescritura/purga física del registro en el vstore.
3. `ForgetMode::Certified`: Ejecución de borrado físico profundo con cálculo de recibo criptográfico SHA-256 (Purge Certificate) que acredita la eliminación del registro, su metadata, sus aristas de grafo y entradas en índices, conforme al estándar VER-02.

## Archivos en scope

- `src/sdk/api/mod.rs` y `src/sdk/api/memory.rs`: definición del método `forget()` y tipos `ForgetMode`, `ForgetReport`.
- `src/storage/engine/delete.rs`: ejecución de lápida, desindexación y purga.
- `vantadb-python/src/lib.rs` / `vantadb-python/src/types.rs`: exposición de `client.memory.forget(key, mode="soft"|"hard"|"certified")`.
- `vantadb-mcp/src/tools/memory.rs`: tool `vanta_forget` en perfiles correspondientes.
- `docs/api/CERTIFIED_DELETE.md`: documentación de certificación y reporte.

## Blast Radius

| Dirección | Módulos |
|---|---|
| Callers | Agentes MCP, SDK Python, SDK Rust, suites de gobernabilidad y privacidad |
| Callees | `HnswIndex::remove`, `BM25Index::remove`, `VFile::zeroize_record`, `PurgeCertificate` |
| Implicaciones | Reemplaza y depreca llamadas dispersas a `delete_node` o `purge_records`; centraliza la emisión de certificados de borrado. |

## Criterios de aceptación

1. Tipo `enum ForgetMode { Soft, Hard, Certified }` y `struct ForgetReport` expuestos públicamente en Rust.
2. `Embedded::forget(&self, key: &str, mode: ForgetMode) -> Result<ForgetReport, VantaError>` implementado y testeado en todos sus modos.
3. Python SDK expone `db.memory.forget(key: str, mode: str = "soft") -> Dict[str, Any]` retornando dict con timestamp, hashes y modo.
4. Tool MCP `vanta_forget` registrada con schema JSON validando argumentos `key` y `mode` (`["soft", "hard", "certified"]`).

## Contrato

```bash
cargo test -p vantadb test_forget_modes_soft_hard_certified
cargo test -p vantadb-mcp test_mcp_tool_vanta_forget
pytest vantadb-python/tests/test_forget_api.py
```

## Steps

### Step 1: Implementar tipos y lógica central en Rust
- Declarar `ForgetMode` y `ForgetReport` en `src/sdk/types.rs`.
- Implementar `Embedded::forget()` delegando en `StorageEngine::purge_node` según el modo seleccionado.
- Generar certificado firmado/hasheado cuando el modo sea `Certified`.

### Step 2: Conectar bindings de Python en `vantadb-python`
- Crear función PyO3 que mapee strings `"soft"`, `"hard"`, `"certified"` a `ForgetMode`.
- Devolver diccionario con metadatos estructurados del reporte de eliminación.

### Step 3: Implementar handler de tool MCP en `vantadb-mcp`
- Definir herramienta `vanta_forget` con su schema JSON Schema.
- Conectar handler al despacho de memoria del motor.

### Step 4: Pruebas cruzadas y documentación
- Redactar tests unitarios en Rust y pytest en Python.
- Actualizar `docs/api/CERTIFIED_DELETE.md` reflejando la nueva API unificada.

## Invariantes de dominio

- Un registro con `Soft` forget no debe aparecer en búsquedas híbridas ni vectoriales, pero puede ser inspeccionado por auditorías administrativas si no ha sido purgado.
- Un registro con `Hard` o `Certified` forget jamás puede ser recuperable tras la finalización de la llamada.

