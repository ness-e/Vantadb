---
title: Gate Fase A — kit del tester (stranger-test + checklist todo-SÍ)
type: operations
status: active
tags: [vantadb, fase-a, gate, early-access, stranger-test]
last_reviewed: 2026-09-19
aliases: []
---

# Gate Fase A — kit del tester

> Kit fuente-única del gate que desbloquea el anuncio. La **ejecución con
> humanos es owner-side** (conseguir 5 personas, correr los tests, veredicto
> GO/NO-GO). Este doc deja todo listo para que el owner solo tenga que
> conseguir las personas. Spec fuente: `docs/research/archive/Investigacion-plan.md`
> Fase A (A.1/A.2/A.3) + fila `EXE-03` en `docs/Backlog.md`. Anuncios pausados
> hasta Fase A en verde (decisión 2026-09-08, vigente).

## 0. Objetivo medible (A.3)

> **Que 5 personas instalen, entiendan, usen un flujo real y digan qué les
> impidió seguir.** Solo entonces se decide qué terminar, separar o comunicar.

- Stranger-test: 2-3 personas siguiendo **solo** el README/QUICKSTART, sin ayuda.
- Flujo real: instalar en limpio → QUICKSTART §3-§5 → 1 búsqueda (vector/texto/híbrida) → export.
- Salida: 5 fichas `usuario-01..05` (§1) + veredicto GO/NO-GO del anuncio.

## 1. Plantilla por persona — `usuario-0N` (A.1, una ficha por tester)

Copiar este bloque por persona (fuente: `Investigacion-plan.md:20-24`, ampliada):

```text
# Prueba de instalación: usuario-01

Fecha: / OS: / Python-Node: / Tiempo hasta instalación: / Tiempo hasta primer resultado:
Bloqueos: / Preguntas: / Errores: / Cambios derivados: [ ]

1. ¿Instalaste sin ayuda? (comando exacto usado):
2. ¿Corrió QUICKSTART §3 (put/get/list)? (pega el output de `get`):
3. ¿Corrió QUICKSTART §5 (vector/texto/híbrida)? (pega las 3 líneas `vector:/text:/hybrid:`):
4. ¿Qué te impidió seguir? (nada / cita el bloqueo):
5. Del 1 al 5, ¿entendiste qué es VantaDB? (marca + 1 frase):
```

Reglas: sin ayuda del owner (solo docs linkeadas en §3); cada error se pega
literal (no se resume); tiempo se mide con reloj (no se estima).

## 2. Checklist Fase A — todo SÍ o no se anuncia (A.2, imprimible)

### Producto

- [ ] Flujo principal desde cero reproducido por un tercero (§1 fichas).
- [ ] Instala en limpio (sin clone: `scripts/install.sh` / `install.ps1`, o `pip install vantadb-py`).
- [ ] QUICKSTART ejecutado por un tercero (`get` imprime `local durable memory`, `list` muestra `memory-1`).
- [ ] 0 críticos abiertos (bugs críticos = NO-GO automático).
- [ ] Versión idéntica en código, paquete y docs — verificado 2026-09-19: `0.5.0` en `Cargo.toml:728` + `vantadb-python/pyproject.toml:7` + `vantadb-ts/package.json:3`.

### Calidad

- [ ] Unitarios en ops críticas + 1 integración en verde.
- [ ] CI verde en limpio (Fast Gate + docs-checks del PR).
- [ ] Sin secretos en código ni en git (`git diff --staged | grep -i "password\\|secret\\|api_key\\|token"` vacío).
- [ ] `LICENSE` (Apache-2.0) presente + changelog de la versión exacta (`docs/CHANGELOG.md` vía release-plz, no a mano).

### Documentación

- [ ] README con estado Early Access + instalación + quickstart PROBADO + limitaciones + cómo reportar + cómo contribuir.
  - ⚠️ Gap honesto 2026-09-19 (ver §4): falta banner explícito "Early Access" y sección "cómo reportar" → acción owner antes del GO.
- [ ] `docs/QUICKSTART.md` probado (§3-§5 corren en limpio).
- [ ] Límites declarados (`Product Boundary` + `docs/operations/EXPERIMENTAL_FEATURES.md`).

### Comunidad

- [ ] 1 canal soporte + 1 anuncios + bienvenida (Discord canónico `https://discord.gg/g8nqB3NtXt`, verificado en `README.md:41`).
- [ ] 1 invite válido (canónico `g8nqB3NtXt`).

### Comunicación

- [ ] Anuncio listo pero NO publicado (ver `docs/DISTRIBUTION.md` §4 — se referencia, no se duplica: checklist de 6 ítems PREPARE-only, publishing pausado hasta Fase A).
- [ ] Cero funciones no existentes (cada claim del anuncio apunta a símbolo/bench real).
- [ ] Cero métricas no verificadas (Regla 11: número sin bench+comando se quita).

## 3. Instrucciones para el tester (15 min)

1. **Instalar** (elige 1, sin clonar): `pip install vantadb-py` · o one-liner `scripts/install.sh` / `install.ps1` (ver `README.md` §Installation + `docs/QUICKSTART.md` §0).
2. **Quickstart** (`docs/QUICKSTART.md` §3 + §5): `put/get/list` por CLI y `quickstart_memory.py` (vector/texto/híbrida). Éxito = `get` imprime el payload + las 3 líneas de hits.
3. **Reportar**: 1 ficha §1 por persona + issues en GitHub (título `[fase-a] <bloqueo>`, con OS, comando exacto y output literal). Sin cuenta: manda la ficha al owner por el canal soporte.

## 4. README — veredicto de honestidad (auditado 2026-09-19, `tools/list` real 87)

**Veredicto: HONESTO, sin fix al README** (`fix solo si deshonesto` = no editar).

| Claim | Evidencia | Veredicto |
|-------|-----------|-----------|
| Cero features v1.0.0 (`entity resolution`, `conflict detection`, `production-ready`) | grep en `README.md` = 0 hits; `Product Boundary` (`README.md:169-184`) clasifica IQL/MCP/remoto como experimental y cloud/HA/RBAC como deferred, idéntico a `EXPERIMENTAL_FEATURES.md` | ✅ |
| Cero conteos de superficie | `README.md` no afirma N tools (la superficie real es 87: 49 core en `handlers/tools.rs` + 37 extendidas scenes/threads + contexto, `docs/api/MCP.md:198,220`) | ✅ |
| Límites declarados | `Product Boundary` + `MVP = embedded memory + WAL + vector/BM25/hybrid + export/import + CLI/Python` (`README.md:173`) | ✅ |
| Versiones idénticas | `0.5.0` ×3 (ver §2 Producto) | ✅ |
| Benchmarks con fuente | hardware citado + comando reproductor (`benchmarks/vanta_benchmark_report.json` vía `vantadb_local_bench.py`, `README.md:282-328`) | ✅ |
| Gap (omisión, no falsedad) | sin banner "Early Access" ni "cómo reportar" (A.2 los exige) | ⚠️ acción owner en §2 Docs |

## 5. Handoff owner (qué falta, todo suyo)

1. Conseguir 5 personas (2-3 stranger-test sin ayuda + resto flujo real).
2. Añadir banner "Early Access" + "cómo reportar" al README (gap §4).
3. Recoger 5 fichas §1, contar bloqueos, firmar veredicto GO/NO-GO en la fila `EXE-03` del Backlog. Sin 5 fichas no hay anuncio.
