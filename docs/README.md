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

The primary entry point to all documentation is the [master-index](dev/master-index.md) (Master Index), which organizes the content into:

- **MPTS** — Complete technical specification (architecture, SDK, operations, roadmap, glossary)
- **End Users** — Quickstart, migration guides, case studies, technical articles, GraphRAG
- **Developers** — API reference, architecture, ADRs, experiments, implementation plans
- **Operators** — Configuration, CI/CD, benchmarks, monitoring, governance
- **Project Tracking** — Backlog, progress, changelog, devlog

## Files and Directories

| Path | Description |
|------|-------------|
| [user/](user/QUICKSTART.md) | End-user docs: quickstart, tutorials, FAQ, blog, operations, benchmarks, book, desktop, discord |
| [dev/](dev/Backlog.md) | Contributor docs: plans, tasks, avance, research, reviews, architecture, strategy, workflow |
| [api/](api/EMBEDDED_SDK.md) | Python and Rust SDK reference (stable path — consumed by CI version check) |
| [CHANGELOG.md](CHANGELOG.md) | Project changelog (stable path — consumed by release-plz) |
| [README.md](README.md) | This overview file |

## Conventions

- Documentation is primarily written in English.
- Spanish is permitted in `glosario/` (bilingual glossary terms) and `web/` (market research, Spanish-language user research).
- Internal vault links use standard relative markdown links (`[label](path.md)`) for GitHub-compatible navigation.
- Public-facing documentation retains GitHub-compatible markdown links where required.
