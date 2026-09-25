# Task API-STD-10 — INDIVIDUAL (9/11) CLI

> **Plan:** `docs/dev/plans/2026-09-24-api-estandarizacion.md`
> **Estado:** ✅ DONE (inline, sin subagentes)
> **Fecha:** 2026-09-24

## 1. Objetivo + contrato

Ficha individual CLI: funcionamiento + uso + código + veredicto.

## 2. Funcionamiento

API humana/terminal (`vanta-cli`, Clap): CRUD, search/similar/multi, IQL query, import/export, server (`--mcp`/HTTP), count. Consumen: humanos + agentes terminal.

## 3. Uso (ejemplos mínimos)

```bash
vanta-cli --db /tmp/v.db put --namespace docs --key a --payload "hi"
vanta-cli --db /tmp/v.db search --query "hello" --limit 5 --json
```

## 4. Código (re-verificado 2026-09-24, grep `cli.rs`)

- **C1 Flags inconsistentes CONFIRMADO:** Import `#[arg(long, name="in")]` (`:123`) vs Export `--out`-style (`:107-111`); `Query.query` posicional (`:134`); `Search.limit` (`:219`) vs `SimilarToKey.top_k` (`:270`).
- **C2 `--json` parcial CONFIRMADO + MATIZADO:** existe en varios comandos (`:91,192,222,257,273`) pero NO global (put/get/list fuera — auditoría previa `cli.rs:43-79`).
- **C3 Salida humana lossy** (auditoría previa): cajas `╭│╰` (`crud.rs:178-235`), `payload[..80]` (`search.rs:140-142`), JSON recortado (`:70-80`), `count` crudo (`crud.rs:551-553`) y `0` exit 0 sin DB (`:522-524`).
- **C4 Mezcla concerns** (auditoría previa): spawn `vantadb-server --mcp` (`server.rs:253-349`), lecturas RW (`search.rs:33-36`), `cmd_put` bypass SDK (`crud.rs:53-55`).

## 5. Veredicto + implicaciones

4/4 (C2 matizado: `--json` existe, falta globalizar). Propuestas 15: POSIX/GNU + Clap (ya en uso), `--in/--out` simétricos, `limit` vs `top_k` unificados, `--json` global con salida COMPLETA, humano truncado solo en TTY, `count` exit≠0 sin DB, `cmd_put` vía SDK, lecturas RO. Blast radius: `cli.rs` + `cli_handlers/` + scripts que parseen salida.

## 6. DoD

- [x] Contrato ✅ · Task file sync · Recitation: CLI ficha completa

## Context Save Point

API-STD-10 DONE. Next: API-STD-11 (proxy). Deuda: ninguna. WIP: ninguno.
