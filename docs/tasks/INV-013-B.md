# Task File: INV-013-B

> **Plan:** `docs/plans/2026-08-05-backlog-validation-actions.md` — Task 36 (Fase 5 Web Frontend)
> **Archivo clave:** `web/src/app/layout.tsx`
> **Premisa:** web/src emite 0 JSON-LD; Metadata API de Next.js 16 NO genera JSON-LD (solo espejo de tags HTML). Schema.org/SoftwareApplication es el tipo apropiado para VantaDB.

## Contexto verificado
- **Version workspace:** `0.5.0` (Cargo.toml:602 `[workspace.package]`)
- **Description producto:** Cargo.toml:6 → "VantaDB: An embedded persistent memory and vector retrieval engine for local-first AI applications."
- **License:** Apache-2.0 (Cargo.toml:7)
- **Python bindings:** `>=3.11` (vantadb-python/pyproject.toml:10)
- **Rust MSRV:** 1.94.1 (Cargo.toml:604)
- **Logo asset:** `web/public/assets/avatar_gato.png` existe (usado en icons)
- **Layout:** RootLayout es Server Component (sin `"use client"`) — script cae en HTML estático.
- **Patrón:** docs oficiales Next.js 16 (`nextjs.org/docs/app/guides/json-ld`) → `<script type="application/ld+json">` nativo en server component + `<head>` manual soportado en App Router (vercel/next.js #80725). Se incluye `.replace(/</g, "\\u003c")` anti-XSS.

## Implementación
- `web/src/app/layout.tsx`:
  - `const jsonLd = {...}` (module scope, schema.org/SoftwareApplication) con name, applicationCategory=DatabaseApplication, applicationSubCategory="Vector Database", operatingSystem, description, version=0.5.0, url (repo github), logo (raw.githubusercontent .../avatar_gato.png), offers price="0"/USD, softwareRequirements (Python >=3.11, Rust MSRV 1.94.1, 64-bit OS), license (Apache-2.0 URL), featureList.
  - `<head>` agregado dentro de `<html>` con `<script type="application/ld+json" dangerouslySetInnerHTML>`.
- **Contrato:** `grep application/ld+json web/src/app/layout.tsx` = 1 ✅; JSON válido ✅.

## JSON-LD literal emitido (tras JSON.stringify + scrub `<`→`\u003c`)
```json
{"@context":"https://schema.org","@type":"SoftwareApplication","name":"VantaDB","applicationCategory":"DatabaseApplication","applicationSubCategory":"Vector Database","operatingSystem":"Windows, macOS, Linux, WebAssembly","description":"VantaDB: An embedded persistent memory and vector retrieval engine for local-first AI applications.","version":"0.5.0","url":"https://github.com/ness-e/Vantadb","logo":"https://raw.githubusercontent.com/ness-e/Vantadb/main/web/public/assets/avatar_gato.png","offers":{"@type":"Offer","price":"0","priceCurrency":"USD"},"softwareRequirements":"Python >= 3.11 bindings; Rust core MSRV 1.94.1; 64-bit OS","license":"https://www.apache.org/licenses/LICENSE-2.0","featureList":["Embedded in-process database","Crash-safe WAL recovery (CRC32C)","Native hybrid search (BM25 + HNSW via RRF)","PyO3 Python bindings","WASM build","MCP server"]}
```

## Verificación
- `grep -c "application/ld+json" web/src/app/layout.tsx` → 1
- JSON.parse del objeto → válido (2026-08-05)
- Rich Results Test: Google acepta JSON-LD en head O body; el script va en `<head>` real.

## Estado
- ⬜ PENDING → (marcar cuando lead confirme build + commit)