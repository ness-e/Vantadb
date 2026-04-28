# Tareas de Optimización del Repositorio en GitHub

Este documento contiene las recomendaciones y configuraciones pendientes para preparar el repositorio para su lanzamiento oficial final (v0.1.0), asegurando una presentación profesional y máxima seguridad.

## 1. Documentación (Para incluir en el próximo commit oficial)
- [x] **`CONTRIBUTING.md`**: Añadir un enlace a `docs/operations/CI_POLICY.md` para que los nuevos colaboradores entiendan la política de pruebas (Fast Gate vs Heavy Certification) y no incluyan dependencias de red en el CI rápido.
- [x] **`SECURITY.md`**: Publicar un canal realista de reporte sin inventar correos; preferir GitHub Security Advisories si el repo tiene private reporting habilitado y usar Issues con etiqueta `security` solo para reportes no sensibles.
- [x] **`README.md`**: Añadir badges de CI / Release / License y evitar prometer publicación en PyPI mientras el paquete siga en modo install-from-source.
- [x] **`.github/dependabot.yml`**: Mantener actualizaciones semanales para Cargo y, cuando aplique, para el paquete Python.

## 2. Descubrimiento y SEO (Para configurar en la UI de GitHub)
En la página principal del repositorio, haz clic en el ícono de engranaje (⚙️) junto a la sección "About" y añade los siguientes **Topics**:
- [ ] `database`
- [ ] `rust`
- [ ] `python`
- [ ] `vector-database`
- [ ] `vector-search`
- [ ] `hnsw`
- [ ] `embedded-database`
- [ ] `graph-database`
- [ ] `ai`
- [ ] `pyo3`
- [ ] `rag`

## 3. Pestaña "Settings" (Para configurar en la UI de GitHub)

### Code (Branches)
- [ ] Ir a **Branches** > **Add branch protection rule**.
- [ ] Escribir `main` como *Branch name pattern*.
- [ ] Marcar **Require a pull request before merging**.
- [ ] Marcar **Require status checks to pass before merging** y seleccionar el *Fast Gate* del repo (`VantaDB CI`, job `build`).

### General (Features)
- [ ] Desmarcar **Wikis** (la documentación ya vive en `docs/`).
- [ ] Marcar **Automatically delete head branches** (limpia las ramas huérfanas tras un PR).

### Security and quality (Code security)
- [ ] Activar **Dependabot alerts**.
- [ ] Activar **Dependabot security updates** (crucial para mantener actualizado `Cargo.lock` y `vantadb-python/pyproject.toml` contra vulnerabilidades).
- [ ] Verificar que las version updates de Dependabot queden operativas usando el archivo `/.github/dependabot.yml` del repo.

### Issues / Pull requests
- [x] (Opcional) Crear plantillas (Issue Templates) para Bug Reports y Feature Requests para estandarizar el feedback de la comunidad.

---
*Nota: La sección "Packages" en GitHub debe permanecer vacía. El camino oficial actual es Releases para binarios nativos y compilación desde fuente para los bindings de Python; no GHCR ni Docker como ruta principal del MVP.*
