# Plan de Implementación: T4.4 — Gobernanza de Comunidad y Contribuciones

## Contexto

Para cerrar la **Fase 4 (Community Launch)**, debemos formalizar la gobernanza del proyecto y las vías de contribución externas (T4.4).
Este plan aborda dos aspectos ejecutables:
1. **ST4.4.2 (Publicación de Issues):** En lugar de hacer que el usuario copie y pegue manualmente los borradores de `docs/operations/PUBLIC_ISSUE_DRAFTS.md`, implementaremos el script `dev-tools/create_github_issues.ps1` que automatiza la creación de las 7 issues comunitarias directamente en el repositorio GitHub usando el CLI oficial de GitHub (`gh`).
2. **ST4.4.3 (Gobernanza y SLAs):** Crearemos la política formal de gobernanza y contribución comunitaria en `docs/operations/COMMUNITY_GOVERNANCE.md` que detalla los SLAs de triage (<48h), la gestión de etiquetas y las normas de conducta.

## Cambios Propuestos

---

### Herramientas de Automatización de Comunidad

#### [NEW] [create_github_issues.ps1](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/dev-tools/create_github_issues.ps1)
* **Objetivo:** Un script interactivo de PowerShell para crear las issues en GitHub.
* **Características:**
  * Verifica si el CLI de GitHub (`gh`) está instalado y autenticado (`gh auth status`).
  * Contiene los títulos, etiquetas y cuerpos estructurados de las 7 issues basados en `docs/operations/PUBLIC_ISSUE_DRAFTS.md`.
  * Llama a `gh issue create --title $title --body $body --label $labels` para cada una de ellas de forma ordenada.

---

### Documentación de Políticas y SLAs

#### [NEW] [COMMUNITY_GOVERNANCE.md](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/operations/COMMUNITY_GOVERNANCE.md)
* **Objetivo:** Formalizar la gobernanza de la comunidad de VantaDB.
* **Secciones:**
  * Flujo de Triage de Issues y Pull Requests.
  * Niveles de severidad y prioridad.
  * Política de respuesta SLA (< 48 horas para revisión de PRs e issues externas).
  * Estructura de moderación de canales (preparatorio para Discord).

---

### Actualizaciones de Control

#### [MODIFY] [VantaDB_Plan_Maestro_Unificado.md](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/VantaDB_Plan_Maestro_Unificado.md)
* Cambiar el estado de `T4.4 — Gobernanza de Comunidad y Contribuciones` a `✅ COMPLETADA` al finalizar la implementación del script y la política de gobernanza.

## Plan de Verificación

1. **Linter de Markdown:** Validar que los archivos de documentación no contengan errores sintácticos.
2. **Prueba Estática del Script:** Ejecutar el script con un flag de simulación (`-DryRun`) para imprimir en consola qué issues se crearían sin realizar las llamadas de API de GitHub reales.

## Estado Final

✅ COMPLETADA — Commit `b4a9080`
