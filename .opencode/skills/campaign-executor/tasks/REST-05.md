# REST-05 — `namespace_stats` en bridge desktop (gap VS-CORE-02)

## Metadata
- **Plan file:** `docs/plans/2026-08-19-vanta-studio-fase4.md`
- **Creado:** 2026-08-19
- **last-synced:** 2026-08-19
- **Estado:** ✅ COMPLETO (vanta-worker; verify lead: 71 desktop tests + 22 TS + build verde)

## Contrato
- Comando Tauri `vanta_namespace_stats` (espejo core) + wrapper `vanta.ts` `namespaceStats()`.
- Sidebar/HOME consumen stats reales; fallback local solo si el backend no lo soporta.
- Build desktop verde.

## Resultado
- `desktop/src-tauri/src/connections/`: DTO `NamespaceStats` + `NamespaceStatsMap` (types.rs), método `namespace_stats` en trait (default Unsupported), impl native (spawn_blocking) + server (`/api/v2/metrics` vía `timeout_ops`) + server_client (GET metrics, parsea `{namespaces}`), read path en manager, comando `vanta_namespace_stats` (commands/data.rs), registro invoke (lib.rs).
- `desktop/src/vanta.ts`: `namespaceStats(expiringSoonWindowMs?)`.
- `desktop/src/vanta-http-map.ts`: mapping → GET `/api/v2/metrics`, `transform: d.namespaces` (+test).
- `WorkspaceShell.tsx`: `useNamespaceCounts` stats primero, fallback `list({limit:500})`; `HomeOverview.tsx`: totals reales (suma de counts, incluye expirados).
- Verify: `cargo test --manifest-path desktop/src-tauri/Cargo.toml` 71 passed / 0 failed; `node --test` 22 pass; `npm run build` exit 0.