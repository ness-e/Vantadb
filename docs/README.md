---
title: VantaDB Documentation
type: docs-index
status: active
last_reviewed: 2026-09-02
language: en
aliases: [Documentation Home, Docs Root, Vault Root]
tags: [vantadb, documentation, index]
---

# VantaDB Documentation

Welcome to the VantaDB documentation vault.

This directory (`docs/`) is the root of the project's Obsidian vault. All documentation is centralized here.

## Main Index

The primary entry point to all documentation is the [master-index](master-index.md) (Master Index), which organizes the content into:

- **MPTS** — Complete technical specification (architecture, SDK, operations, roadmap, glossary)
- **End Users** — Quickstart, migration guides, case studies, technical articles, GraphRAG
- **Developers** — API reference, architecture, ADRs, experiments, implementation plans
- **Operators** — Configuration, CI/CD, benchmarks, monitoring, governance
- **Project Tracking** — Backlog, progress, changelog, devlog

## Files and Directories

| Path | Description |
|------|-------------|
| [glosario/](user/glosario/README.md) | Glossary of technical terms (concepts, engines, metrics) |
| [api/](api/EMBEDDED_SDK.md) | Python and Rust SDK reference |
| [architecture/](dev/architecture/ARCHITECTURE.md) | Core engine architecture, ADRs, audits |
| [operations/](user/operations/CONFIGURATION.md) | CI/CD, benchmarks, configuration, monitoring |
| [tutorials/](user/tutorials/index.md) | Learning path: agent memory, RAG, hybrid search, migrations |
<!-- | [articles/](../web/content/blog/why-i-built-vantadb-local-memory-engine.md) | Published technical articles (planned) | -->
| [graphrag/](dev/graphrag/README.md) | GraphRAG architecture and design |
| [iql/](api/IQL.md) | Interactive Query Language reference |
| [avance/](dev/avance/README.md) | Project progress dashboard |
| [Backlog.md](dev/Backlog.md) | Active task backlog |
| [CHANGELOG.md](CHANGELOG.md) | Project changelog |
| [QUICKSTART.md](user/QUICKSTART.md) | 5-minute quickstart guide |
| [avance/historial/backlog-history.md](dev/avance/historial/backlog-history.md) | Development log and daily notes |
| [README.md](README.md) | This overview file |

## Conventions

- Documentation is primarily written in English.
- Spanish is permitted in `glosario/` (bilingual glossary terms) and `web/` (market research, Spanish-language user research).
- Internal vault links use standard relative markdown links (`[label](path.md)`) for GitHub-compatible navigation.
- Public-facing documentation retains GitHub-compatible markdown links where required.
