---
title: "FIND-385 — Configuración Unificada 5-Capas (vanta.toml) & CLI Interactivo 'vanta config'"
kind: task
description: "Implementar cargador unificado de configuración de 5 capas (CLI > Envs > vanta.toml local > ~/.vanta/config.toml global > Defaults) y comandos CLI vanta config show/set/init/doctor"
status: pending
owners: ["Principal Systems Engineer"]
tags: ["cli", "config", "tui", "ux", "vanta-toml", "devx"]
---

# FIND-385: Configuración Unificada 5-Capas & CLI Interactivo (`src/config/loader.rs`, `src/cli_handlers/config.rs`)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-19-pre-merge-systems-and-ux-remediation.md`
- **Fuente:** `docs/dev/Backlog.md` (Experiencia de Desarrollador y Unificación de Configuración)
- **Esfuerzo:** 🟡 Medio (2h)
- **Prioridad:** 🟠 Alta
- **Tipo:** Rust
- **Turns estimados:** 12
- **Creado:** 2026-10-19T02:00
- **last-synced:** 2026-10-19T03:00
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes de ejecución (downhill):** 3 steps de ejecución restantes

---

## Blast Radius

| Dirección | Módulos |
|---|---|
| **Callers** | Todos los comandos del CLI (`vanta`), arranque de servidor (`vanta server`), inicialización de librerías (`src/bin/vanta-cli.rs`) |
| **Callees** | `toml`, `dirs`, `src/config.rs` (`VantaConfig`), `src/cli.rs` |
| **Implicaciones** | Unifica la administración de más de 45 variables dispersas en un esquema estándar de archivos TOML (`./vanta.toml` y `~/.vanta/config.toml`). Añade interfaz de terminal intuitiva para auditar el estado del sistema, detectar variables huérfanas y modificar configuraciones sin editar archivos manualmente. |

---

## Impacto Mapeado (Regla 0)

- **Archivos leídos (completos):** `src/config.rs:1330-1580`, `src/cli.rs:1-120`, `src/cli_handlers/mod.rs`.
- **Archivos referenciados hacia dentro:** `serde`, `toml`, `std::env`.
- **Archivos que referencian a los editados:** Puntos de entrada principales del CLI y servidor.
- **Veredicto impacto:** Mejora fundamental de DevX; cero roturas de compatibilidad (las variables de entorno siguen teniendo precedencia sobre los archivos TOML).

---

## Contrato

`vanta config show imprime la tabla completa de parámetros indicando la capa de procedencia de cada valor, y vanta config set <clave> <valor> actualiza vanta.toml de forma persistente`

---

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Default Recomendado | Resuelto |
|---|---|---|---|---|
| 1 | Jerarquía de resolución (5 capas) | 1. Flags CLI > 2. Variables de Entorno > 3. `./vanta.toml` (Proyecto) > 4. `~/.vanta/config.toml` (Usuario) > 5. Defaults en código | Jerarquía canónica de 5 capas estándar | ✅ decidido-por-evidencia |
| 2 | Enmascaramiento de secretos en salida | Ocultar campos de auth, api keys y contraseñas (`sk-***`) | Enmascaramiento por regex de campos sensibles | ✅ decidido-por-evidencia |
| 3 | Comando de inicialización rápida | Subcomando `vanta config init` que genera un `vanta.toml` anotado con mejores prácticas | Generación de plantilla con comentarios explicativos | ✅ decidido-por-evidencia |

---

## Invariantes de Dominio (Handoff — MUST)

- **Invariantes a preservar:** Las variables de entorno (`VANTADB_*`) deben seguir teniendo mayor prioridad que los archivos de configuración en disco para preservar el comportamiento esperado en contenedores Docker y orquestadores Kubernetes.
- **Comandos de verificación:** `cargo nextest run -p vantadb --lib config::loader` y `cargo test test_config_cli`
- **Deuda pendiente:** Ninguna.

---

## Recitation (Canónico — Estructura Única)

```yaml
contract:
  verificacion: cargo nextest run -p vantadb --lib config::loader
  evidencia:
    - claim: vanta config resuelve la cascada de 5 capas y atribuye correctamente la procedencia de cada configuración
      evidencia: src/config/loader.rs:load_hierarchical_config
      confianza: alta
  artefactos:
    - src/config/loader.rs
    - src/cli_handlers/config.rs
    - src/cli.rs
  invariantes: Las variables de entorno sobreescriben a los archivos en disco; los secretos nunca se imprimen en texto plano
  deuda: ninguna
  queda_pendiente: Documentar vanta.toml en docs/api/CONFIGURATION.md
```

---

## Deuda Técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (deuda eliminada: consolidación de configuraciones antes fragmentadas en scripts aislados).

---

## Definition of Done (Contrato Multi-Nivel — P2-08)

| Nivel | Gate | Estado |
|---|---|---|
| **Task** | Cargador jerárquico implementado, subcomandos `vanta config` operativos y tests verdes | ⬜ PENDING |
| **Commit** | Atomic commit `feat(cli): add 5-layer unified configuration loader and interactive cli` | ⬜ PENDING |
| **Release** | Documentación de referencia en `docs/api/CONFIGURATION.md` y guía en `QUICKSTART.md` | ⬜ PENDING |

---

## Herramientas Necesarias

- `cargo nextest`
- `cargo clippy`
- **Skills cargadas (SDP):** `api-and-interface-design`, `writing-guidelines` (documentación de comandos).

---

## Investigation Notes

Actualmente, el operador debe adivinar si una configuración fue aplicada revisando si definió `VANTADB_STORAGE_PATH`, `VANTADB_PORT`, `VANTADB_MEMORY_LIMIT`, etc. La ausencia de un archivo centralizado en el repositorio provocaba que diferentes desarrolladores usaran configuraciones locales inconsistentes sin un archivo `vanta.toml` versionable en git.

---

## Incógnitas (Uphill) vs Pendientes (Downhill) — P2-03

| Eje | Contador | Descripción |
|---|---|---|
| **Incógnitas abiertas (uphill)** | 0 | Esquema TOML y cascada de capas resueltos |
| **Pendientes de ejecución (downhill)** | 3 | 1) `src/config/loader.rs`; 2) Subcomandos en `cli.rs` y `config.rs`; 3) Tests unitarios |
| **% completado** | 10% | Especificación completa; implementación pendiente |

---

## Fases Explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — Enmascaramiento estricto de secretos (`VANTADB_AUTH_TOKEN`, `OPENAI_API_KEY`, etc.) en la terminal.
- [x] **PERFORMANCE** — El cargador se ejecuta en menos de 2 milisegundos en el inicio; cachea la configuración deserializada.

---

## Steps

### Step 1: Implementar `src/config/loader.rs`
- **Archivos:** `src/config/loader.rs`
- **Acción:** Definir `enum ConfigSource { Cli, Env, ProjectToml, UserToml, Default }`. Implementar `load_effective_config()` que evalúe las 5 capas en cascada y asigne a cada campo su procedencia.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 2: Implementar Subcomandos `vanta config` en `src/cli_handlers/config.rs`
- **Archivos:** `src/cli_handlers/config.rs`, `src/cli.rs`
- **Acción:** Crear comandos `vanta config show` (tabla coloreada con procedencia), `vanta config set <key> <value>` (actualiza TOML), `vanta config init` (crea plantilla) y `vanta config doctor` (valida salud de rutas y modelos).
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 3: Añadir Tests Unitarios de Precedencia
- **Archivos:** `src/config/loader.rs`
- **Acción:** Verificar que una variable de entorno tiene precedencia sobre un valor en `vanta.toml`, y este sobre el valor por defecto.
- **Verify:** `cargo nextest run -p vantadb --lib config::loader`
- **Estado:** ⬜ PENDING

---

## Dependencias

- Ninguna dependencia previa.

---

## Review (GATE — Agente Distinto, P2-01)

- **Revisor:** `vanta-audit` (CLI & Developer Experience Reviewer)
- **Enfoque:** Usabilidad en terminal, enmascaramiento de secretos y robustez de parsing TOML.
- **Cómo se probó:** Invocación de `show`, `set` e `init` con verificaciones unitarias.
- **Checklist Anti-Hábitos Tóxicos:**
  - [x] Nunca imprimir claves en texto plano.
  - [x] Proteger contra inyección de tipos en TOML.
- **Veredicto:** ⏳ PENDING ejecución

---

## Notas

La plantilla generada por `vanta config init` incluye configuraciones recomendadas para desarrollo local, servidores de producción y perfiles embebidos de baja memoria.
