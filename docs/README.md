---
title: VantaDB Documentation
type: docs-index
status: active
last_reviewed: 2026-07-01
language: en
aliases: [Documentation Home, Docs Root, Vault Root]
tags: [vantadb, documentation, index]
---

# VantaDB Documentation

Welcome to the VantaDB documentation vault.

This directory (`docs/`) is the root of the project's Obsidian vault. All documentation is centralized here.

## Main Index

The primary entry point to all documentation is the [[master-index|Master Index]], which organizes the content into:

- **MPTS** — Complete technical specification (architecture, SDK, operations, roadmap, glossary)
- **End Users** — Quickstart, migration guides, case studies, technical articles, GraphRAG
- **Developers** — API reference, architecture, ADRs, experiments, implementation plans
- **Operators** — Configuration, CI/CD, benchmarks, monitoring, governance
- **Project Tracking** — Backlog, progress, changelog, devlog

## Files and Directories

| Path | Description |
|------|-------------|
| [[glosario/README.md\|glosario/]] | Glossary of technical terms (concepts, engines, metrics) |
| [[api/EMBEDDED_SDK.md\|api/]] | Python and Rust SDK reference |
| [[architecture/ARCHITECTURE.md\|architecture/]] | Core engine architecture, ADRs, audits |
| [[operations/CONFIGURATION.md\|operations/]] | CI/CD, benchmarks, configuration, monitoring |
| [[tutorials/03-migrating-from-chromadb.md\|tutorials/]] | Migration guides (ChromaDB, LanceDB) |
| [[articles/why_i_built_local_memory_engine.md\|articles/]] | Published technical articles |
| [[case_studies/rag_edge_device.md\|case_studies/]] | Deployment case studies |
| [[graphrag/README.md\|graphrag/]] | GraphRAG architecture and design |
| [[experimental/IQL.md\|experimental/]] | Interactive Query Language and experimental features |
| [[progreso/README.md\|progreso/]] | Project progress dashboard |
| [[Backlog.md]] | Active task backlog |
| [[CHANGELOG.md]] | Project changelog |
| [[QUICKSTART.md]] | 5-minute quickstart guide |
| [[bitacora.md]] | Development log and daily notes |
| [[README.md]] | This overview file |

## Conventions

- Documentation is primarily written in English.
- Spanish is permitted in `glosario/` (bilingual glossary terms) and `web/` (market research, Spanish-language user research).
- Internal vault links use Obsidian wikilinks format (`[[Link]]`) for seamless navigation.
- Public-facing documentation retains GitHub-compatible markdown links where required.
