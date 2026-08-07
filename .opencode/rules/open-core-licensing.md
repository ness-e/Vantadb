# Open Core Licenciamiento — Reglas

> **Scope:** modelo Open Core (decisión 2026-08-06): core `vantadb` Apache-2.0 + ediciones Pro/Enterprise propietarias en `vantadb-pro`. Separación de features, licenciamiento, distribución de artefactos, validación de licencia por nodo.
> **No tocar aquí:** CI/versionado (`release-ci.md`), API pública (`api-contract.md`), bindings (`python-bindings.md`, `js-ecosystem.md`).
> **Status:** 🟢 Vigente
> **Fuentes:** `docs/plans/2026-08-06-oc-vantadb-pro.md`, C3 `VantaDB_Manual_Estrategico_Unificado.md`, investigación de licencias 2026-08-06 (SurrealDB BSL vs Open Core).

## Reglas

### 1 — El core `vantadb` permanece Apache-2.0 y sin features Pro

- **Must:** mantener `LICENSE` (Apache-2.0) y `license = "Apache-2.0"` en el `Cargo.toml` raíz.
- **Must not:** relicenciar el core, ni mover features comerciales existentes (`encryption`, `wal-shipping`, `pitr`, `prometheus`, `server`, `tls`) fuera del core sin una decisión humana explícita (D4).
- **Must not:** agregar al core Apache features nuevas diseñadas para ser exclusivas de Pro.
- **Por qué:** el moat comercial no es la licencia sino las features + marca (decisión D1/D4). Mover código del core rompe adopción y contribuidores; mantener el core como Apache-2.0 maximiza la adopción del motor embebido.

### 2 — El crate/edición Pro vive FUERA del workspace

- **Must:** el repo/crate Pro (`vantadb-pro`) residir en directorio separado y **no** aparecer en `[workspace] members` ni `default-members` del `Cargo.toml` raíz (hoy: `members = [".", "vantadb-python", "vantadb-server", "vantadb-mcp", "vantadb-wasm"]`). Esa lista NO se toca para añadir Pro.
- **Must not:** empaquetar `vantadb-pro` en el build, `cargo package`, `deny.toml` ni publishes del core.
- **Por qué:** el core no debe depender de, ni arrastrar, código propietario (D3). Un fallo en Pro no puede bloquear CI del core.

### 3 — Entrega: solo artefactos compilados, nunca el source

- **Must:** Pro se entrega compilado (`.crate`/`.whl`/binario vía registro privado a token por cliente, o artefacto firmado on-prem para Enterprise).
- **Must not:** exponer el source de Pro en público, ni en el repo del core ni en registros públicos.
- **Must not:** embeber el secreto/capitacón del registro o de firma de licencias en VCS.
- **Por qué:** la licencia propietaria protege la renta; exponer el source la anula (D3).

### 4 — Validación de licencia por nodo, offline

- **Must:** cada feature Pro valida su licencia (`vantadb.license`) antes de activarse: formato, expiración (`yyyy-mm-dd`) y límite de nodos. Sin servidor de licencias ni `call-home`.
- **Must not:** relajar la verificación en builds `release`/compilados.
- **Por qué:** la licencia declara cuántos nodos puede usar el cliente; el conteo real (multi-node con archivo touch) queda como ceiling documentado, no se simula.

<!-- Referencias cruzadas: → release-ci.md, api-contract.md, docs/plans/2026-08-06-oc-vantadb-pro.md -->