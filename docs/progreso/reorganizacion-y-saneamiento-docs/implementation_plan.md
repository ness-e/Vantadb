# Plan de Saneamiento y Reorganización Documental — VantaDB

## Objetivo
El objetivo de esta fase es estructurar de forma limpia, consistente e intuitiva la carpeta `docs/` en 5 pilares fundamentales, eliminando la deuda técnica documental, stubs y archivos obsoletos, y manteniendo un único mapa maestro en `docs/README.md` como Single Source of Truth (SSoT).

## Pilares de Estructura Propuestos

1. **API & Interfaces** (`docs/api/`): Referencias sobre interfaces de desarrollo y protocolos de comunicación del motor.
2. **Architecture & Design** (`docs/architecture/`): Documentación técnica del diseño del core, formato binario, índices y decisiones de diseño históricas (ADRs).
3. **Operations & Releases** (`docs/operations/`): Políticas operativas, configuraciones, telemetría de memoria, backups, releases y guías de resiliencia (fuzzing/caos).
4. **Case Studies** (`docs/case_studies/`): Casos de uso prácticos e integraciones en entornos locales o distribuidos.
5. **Experimental** (`docs/experimental/`): Documentación de funcionalidades en cuarentena o descartadas para el MVP (ej: IQL).

---

## Cambios Realizados

### Componente Documentación (`docs/`)

#### [NEW] [docs/operations/EXECUTIVE_TECHNICAL_AUDIT.md](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/operations/EXECUTIVE_TECHNICAL_AUDIT.md)
* Unificación del reporte técnico ejecutivo e informes de auditoría del estado del repositorio.

#### [NEW] [docs/operations/PILOT_PROGRAM.md](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/operations/PILOT_PROGRAM.md)
* Unificación de las guías de onboarding y outreach del programa de pilotos en un solo documento operativo.

#### [NEW] [docs/operations/HISTORIAL_PROGRESO_UNIFICADO.md](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/operations/HISTORIAL_PROGRESO_UNIFICADO.md)
* Consolidación de los 43 walkthroughs históricos de progreso en un único archivo cronológico ordenado por categoría temática para evitar saturar el repositorio.

#### [DELETE] [docs/api/IQL.md](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/api/IQL.md)
* Eliminación de la redirección redundante obsoleta para que `docs/experimental/IQL.md` sea la única fuente del parser en cuarentena.

#### [MODIFY] [docs/README.md](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/README.md)
* Actualización completa de la navegación de docs para reflejar la distribución en los 5 pilares, actualizando enlaces y agregando resúmenes explicativos de los archivos unificados.

#### [MOVE]
* Mover `docs/adr/` a `docs/architecture/adr/`.
* Mover `docs/ai/agent.md` a `docs/operations/AGENT_INSTRUCTIONS.md`.
* Mover `docs/audits/` a `docs/architecture/audits/`.
* Mover `docs/snapshots/` a `docs/operations/snapshots/`.
* Mover `docs/ADVANCED_TOKENIZER.md` a `docs/architecture/ADVANCED_TOKENIZER.md`.
* Mover `docs/BENCHMARKS.md` a `docs/operations/BENCHMARKS.md`.
* Mover `docs/EDITOR_INTEGRATIONS.md` a `docs/operations/EDITOR_INTEGRATIONS.md`.
* Mover `docs/MCP.md` a `api/MCP.md`.

---

## Plan de Verificación

### Verificación Manual
- Validar que todos los enlaces relativos de `docs/README.md` sean correctos y apunten a los archivos existentes sin romper la navegación.
- Verificar que el script `dev-tools/scripts/collect_code.ps1` genere los snapshots correctamente en `docs/operations/snapshots/`.
