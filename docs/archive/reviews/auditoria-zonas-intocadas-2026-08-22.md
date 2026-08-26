---
title: "Auditoría zonas intocadas — segunda ola (GOV-F2)"
type: audit
status: final
date: 2026-08-22
method: vanta-research read-only (ses_fd427f217ffecyvZuj3ypx2app)
---

# Auditoría zonas intocadas — GOV-F2 (2026-08-22)

> Alcance: Manual Estratégico 164KB · SKILLS-MANIFEST · .opencode/ (AGENTS/agents/rules/references) · integrations/providers READMEs · workflows CI profundos · plans/archive muestreo. Read-only estricto; hallazgos van a tickets.

## Tabla de hallazgos

| Zona | Finding | Severidad | Evidencia | Recomendación |
|---|---|---|---|---|
| Manual Estratégico | Claims de tracción congelados en v0.4.0 beta / 1.075 commits / 2 estrellas; realidad v0.5.0 con P26-P31 cerradas | 🔴 ALTA | `Manual:355,456,567,989` vs `Cargo.toml:646` + plan cierre `:217` | **DIVIDIR** (ver abajo) |
| Manual Estratégico | Anexo VII manda "bump v0.2.0" y congela backlog hasta enero 2027 — contradice estado real | 🟡 | `Manual:1058` vs providers Cargo.toml (todos 0.5.0) | Marcar Parte D/C como snapshot histórico con banner |
| Manual Estratégico | Sin TOC navegable ni fecha de revisión; claims crates.io "v0.1.4" | 🟡 | `Manual:1033`, cabecera | Banner snapshot + TOC al dividir |
| SKILLS-MANIFEST | Proyecto-local: declara 193 (162+31) → disk hoy **193 ✅ match** (la premisa "154" del ticket era incorrecta — manifiesto actualizado 08-19) | 🟢 OK | `SKILLS-MANIFEST.md:3-9` + conteo disk | Ninguna |
| SKILLS-MANIFEST | Globales drift: ~/.agents 153→**160**, ~/.claude 26→**33** | 🟡 | `SKILLS-MANIFEST.md:10` | Re-auditar o marcar informativos |
| .opencode/AGENTS.md | Autocontradicción: :11 y :102 dicen "111 skills (82+29)" vs manifiesto 193 | 🔴 ALTA | `AGENTS.md:11,102` | Actualizar a "193 (162+31), ver SKILLS-MANIFEST" |
| .opencode/AGENTS.md | Regla 2 dice "7 instancias" continue-on-error; hay **8 reales** (todas con # CATEGORY ✓ Regla 2 cumple) | 🟡 | ci-rust-10.yml:302,394,503,537,573; heavy-bench:148; adapters-62:142; wheels-60:172 | Corregir contador en AGENTS.md |
| Workflows docs | 17 workflows; docs/workflow cubre 15. Sin doc: `release.yml`, `ci-examples-12.yml` | 🟡 | glob vs glob | Crear los 2 docs |
| .opencode consistencia | Rutas/agentes/referencias citados existen (10 agents, 15 rules, references OK) | 🟢 OK | globs | — |
| Learnings AGENTS.md | Bloques fechados hasta GOV-D2 hoy; técnicos y accionables | 🟢 OK | AGENTS.md:607-664 | Mantener política |
| integrations/providers | READMEs declaran pip install vantadb-* con código local presente; PyPI 404 ya trackeado MKT-18f | 🟡 conocida | langchain README:8 | Cerrar vía MKT-18f |
| plans/archive muestreo ×10 | Cierres por-task coherentes ✅; minor: sin bloque consolidado único "Estado final" | 🟢 minor | varios | Plantilla opcional |

**Severidades:** 🔴 2 · 🟡 6 · 🟢 5 (incluye 2 OK que corrigen premisas de la auditoría V1).

## Recomendación Manual Estratégico (resuelve uphill #4)

**DIVIDIR**: (a) extraer partes de negocio + anexos a `docs/business/` como plan fundacional congelado con banner de fecha; (b) reescribir/eliminar el anexo de estado técnico (los docs técnicos ya lo cubren mejor); (c) archivar el monolito original. Razón: mezcla estrategia de negocio vigente con claims técnicos falsos — canonizar propagaría muertos, archivar perdería la estrategia.

## Tickets derivados (al Backlog cuando owner apruebe)
1. AGENTS.md skills count fix (🔴, 10 min)
2. Split Manual Estratégico según recomendación (🟠, medio día)
3. Docs workflow para release.yml + ci-examples-12.yml (🟡)
4. SKILLS-MANIFEST conteos globales (🟡)
5. Regla 2 contador 7→8 (🟡)
