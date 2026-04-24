# Tareas de Optimización del Repositorio en GitHub

Este documento contiene las recomendaciones y configuraciones pendientes para preparar el repositorio para su lanzamiento oficial final (v0.1.0), asegurando una presentación profesional y máxima seguridad.

## 1. Documentación (Para incluir en el próximo commit oficial)
- [ ] **`CONTRIBUTING.md`**: Añadir un enlace a `docs/operations/CI_POLICY.md` para que los nuevos colaboradores entiendan la política de pruebas (Fast Gate vs Heavy Certification) y no incluyan dependencias de red en el CI rápido.
- [ ] **`SECURITY.md`**: Actualizar la sección de reporte de vulnerabilidades para incluir un correo electrónico de contacto oficial (ej. `security@tudominio.com` o el correo del mantenedor), ya que GitHub no permite mensajes privados directos sin un Issue público.
- [ ] **`README.md`**: (Opcional) Una vez que el SDK de Python se publique, agregar un badge de PyPI en la cabecera.

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
- [ ] Marcar **Require status checks to pass before merging** y seleccionar los workflows del *Fast Gate* (ej. `Rust CI`).

### General (Features)
- [ ] Desmarcar **Wikis** (la documentación ya vive en `docs/`).
- [ ] Marcar **Automatically delete head branches** (limpia las ramas huérfanas tras un PR).

### Security and quality (Code security)
- [ ] Activar **Dependabot alerts**.
- [ ] Activar **Dependabot security updates** (crucial para mantener actualizado el `Cargo.lock` contra vulnerabilidades).

### Issues / Pull requests
- [ ] (Opcional) Crear plantillas (Issue Templates) para Bug Reports y Feature Requests para estandarizar el feedback de la comunidad.

---
*Nota: La sección "Packages" en GitHub debe permanecer vacía, ya que VantaDB (versión embebida) se distribuye como binarios nativos en Releases y vía PyPI, no como contenedores de Docker.*
