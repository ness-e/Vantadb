# ADR-042: Migrar 6 firmas `pub` a traits del trait-split storage↔index (major)

> **Estado:** Aceptado (decisión humana vía Gate V Q-F3X-impl, opción A, 2026-09-13) ·
> **Tarea:** F3X-impl (diseño en `docs/dev/tasks/F3X.md`, hallazgo H4)

## Contexto

La hoja neutral `src/index_port.rs` (F3X-X1a, verde y sin uso aún) rompe el ciclo
storage↔index solo si los consumidores migran a ella. Seis funciones `pub`
(`traverse_graph`, `reindex_nodes`, `compact_layout` en `src/storage/archive.rs` y
otras tres que nombran `CPIndex` en concreto) exponen el tipo concreto en su firma;
cambiarlas a los traits sellados (`IndexPort`/`MmapBackend`/`VectorStoreRef`) es
breaking para consumidores externos del crate (v0.5.0). Séptimo ítem del mismo
cambio (B2 del review): `release_mmap_vector` se mueve de módulo con cuerpo idéntico
(fallo `semver-checks` documentado, sin cambio semántico).

## Decisión

**ADR + major:** migrar las 6 firmas a los traits en este cambio, documentado como
breaking en el changelog (mismo precedente que ADR-041). No se deja la hoja a medias:
o el ciclo se rompe de verdad o no se hace (la opción parcial B dejaba el gate en rojo).

## Consecuencias

- **Costos:** bump major; consumidores externos de esas 6 funciones migran firmas;
  `cargo semver-checks` debe correrse antes de merge (se espera rojo major documentado).
- **Riesgos:** bajo — cambio mecánico de firmas, verificado por compilación + suites
  storage/index verdes; sin cambio semántico.
- **Deuda evitada:** la hoja sin consumidores sería código muerto con `allow(dead_code)`.
- **Alternativas descartadas:** parcial X1a (gate en rojo, deuda con dueño),
  DEFER explícito (la mudanza seguiría sin cotizar).
