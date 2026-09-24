# Task API-STD-13 — CONJUNTO: arquitectura + orquestador + IPC (responder con código)

> **Plan:** `docs/dev/plans/2026-09-24-api-estandarizacion.md`
> **Estado:** ✅ DONE (inline, sin subagentes)
> **Fecha:** 2026-09-24

## 1. Objetivo + contrato

Responder con código: ¿quién orquesta? ¿cómo se comunican? Fijar single-ownership + fronteras IPC.

## 2. Respuestas verificadas (código, 2026-09-24)

**P1 — NO hay orquestador único.** Cada runtime carga el core in-process:
- Python: PyO3 cdylib `*.abi3.so` (`vantadb-python/`, excluido providers del workspace por MSVC `Cargo.toml:757`).
- Node: NAPI-RS `.node` (`vantadb-node/`).
- TS: **doble backend** — WASM `pkg/` (`vantadb.ts`, `serde_wasm_bindgen`) Y napi-node (`native.ts:113` "powered by vantadb-node napi-rs", `import("vantadb-node")` en `:67,133,167`, fallback plataforma `:127`).
- CLI: SDK directo, salvo `--mcp` que hace `Command::new("vantadb-server").arg("--mcp").spawn()` con fallback PATH (`server.rs:262-324`).
- Entre procesos: HTTP TCP localhost (server/proxy) + MCP stdio. **Sin** Unix sockets/SHM/WebSockets (grep `UnixStream|uds|WebSocket|shm` vacío salvo `.gitignore:42` + nota MSVC).
- MCP local opencode: launcher `vanta-mcp-local.ps1 -DbPath C:/Users/Eros/.vantadb` (`opencode.jsonc:76-88`), "87 tools perfil full" — ⚠️ deriva vs `AGENTS.md` (15 tools): anotado para 16/18.

**P2 — Vías de comunicación hoy:** (a) in-process lib (`.so`/`.node`/`.wasm`), (b) hijo spawn (CLI→server), (c) TCP localhost (server/proxy), (d) stdio (MCP). No hay `.so` cargado por Node ni WebSockets entre componentes.

## 3. Hallazgo nuevo (para 15/16)

- `native.ts:315`: "napi only takes numbers" — el backend napi **también** pierde `u128` >2^53 (tercer sitio tras TS-wasm traversals y `QueryResult::Write`). El estándar `u128→string` debe cubrir napi sí o sí.
- `native.ts:127`: fallback plataforma documentado (binario `.node` platform-specific).

## 4. Decisiones propuestas (para 15)

Single-owner Rust (estado en ficheros+WAL+VFile; bindings vistas sin lifecycle); Unix-socket SOLO si benchmark Regla 9 lo justifica (hoy TCP basta); unificar backends TS (wasm vs napi divergen: `u128` string vs number — ver `types.ts:58-65`).

## 5. DoD

- [x] Contrato ✅ (diagrama + citas) · Task file sync · Recitation: conjunto respondido

## Context Save Point

API-STD-13 DONE. Next: API-STD-14 (web-checklist). Deuda: ninguna. WIP: ninguno.
