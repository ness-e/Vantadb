# Roadmap de Integración: VantaDB + Web

> Plan maestro para integrar `web/` dentro del monorepo VantaDB.
> Creado: 2026-07-02

---

## Fases

### Fase 0 — Setup del plan
- [x] Explorar estructura completa de VantaDB/ y web/
- [x] Auditar archivos basura, duplicados, configs obsoletas
- [x] Comparar docs/ vs web/docs/ y backlogs
- [x] Crear plan/ con roadmap, tareas, cleanup list
- [ ] Decidir unificación de docs

### Fase 1 — Cleanup del repositorio
- [ ] Eliminar `ghost_out/` (~128 MB)
- [ ] Eliminar `_test_async/`, `_test_import/`, `_test_listfmt/` (~201 MB)
- [ ] Eliminar `test/` (~128 MB)
- [ ] Eliminar logs vacíos (4 archivos)
- [ ] Eliminar `vector_index.bin`
- [ ] Eliminar `db/`, `quickstart_db/`, `vantadb_data/` (~192 MB)
- [ ] Consolidar certificados duplicados (3 → 1 copia)
- [ ] Limpiar `web/.playwright-mcp/`, `web/.tanstack/tmp/`
- [ ] Eliminar `job_log.txt` (328 KB — stale)

### Fase 2 — CI/CD Integration
- [ ] Mover `web/.github/workflows/deploy.yml` → `.github/workflows/web-deploy.yml`
- [ ] Filtrar workflows existentes con `paths-ignore: ["web/**"]`
- [ ] Configurar `rootDirectory: "web"` en Vercel dashboard
- [ ] Eliminar GitHub Pages workflow (obsoleto — solo Vercel)
- [ ] Verificar deploy

### Fase 3 — Docs Unification
- [ ] Mover `docs/REPORTE_INVESTIGACION_Y_DECISIONES.md` → `docs/archive/`
- [ ] Agregar cross-links recíprocos entre backlogs
- [ ] Actualizar `docs/master-index.md` (v0.2.0, repo URL, link a web/docs)
- [ ] Agregar referencia al web backlog en `docs/Backlog.md`

### Fase 4 — Verification
- [ ] `cargo build` (todo el workspace Rust)
- [ ] `npx tsc --noEmit` (TypeScript web)
- [ ] Verificar que Vercel deploy funciona desde `web/`
