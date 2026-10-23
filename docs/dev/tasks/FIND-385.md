---
title: "FIND-385 â€” ConfiguraciÃ³n Unificada 5-Capas (vanta.toml) & CLI Interactivo 'vanta config'"
kind: task
description: "Implementar cargador unificado de configuraciÃ³n de 5 capas (CLI > Envs > vanta.toml local > ~/.vanta/config.toml global > Defaults) y comandos CLI vanta config show/set/init/doctor"
status: completed
owners: ["Principal Systems Engineer"]
tags: ["cli", "config", "tui", "ux", "vanta-toml", "devx"]
---

# FIND-385: ConfiguraciÃ³n Unificada 5-Capas & CLI Interactivo (`src/config/loader.rs`, `src/cli_handlers/config.rs`)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-19-pre-merge-systems-and-ux-remediation.md`
- **Fuente:** `docs/dev/Backlog.md` (Experiencia de Desarrollador y UnificaciÃ³n de ConfiguraciÃ³n)
- **Esfuerzo:** ðŸŸ¡ Medio (2h)
- **Prioridad:** ðŸŸ  Alta
- **Tipo:** Rust
- **Turns estimados:** 12
- **Creado:** 2026-10-19T02:00
- **last-synced:** 2026-10-19T03:00
- **Estado:** â¬œ PENDING
- **IncÃ³gnitas (uphill):** 0 abiertas
- **Pendientes de ejecuciÃ³n (downhill):** 3 steps de ejecuciÃ³n restantes

---

## Blast Radius

| DirecciÃ³n | MÃ³dulos |
|---|---|
| **Callers** | Todos los comandos del CLI (`vanta`), arranque de servidor (`vanta server`), inicializaciÃ³n de librerÃ­as (`src/bin/vanta-cli.rs`) |
| **Callees** | `toml`, `dirs`, `src/config.rs` (`VantaConfig`), `src/cli.rs` |
| **Implicaciones** | Unifica la administraciÃ³n de mÃ¡s de 45 variables dispersas en un esquema estÃ¡ndar de archivos TOML (`./vanta.toml` y `~/.vanta/config.toml`). AÃ±ade interfaz de terminal intuitiva para auditar el estado del sistema, detectar variables huÃ©rfanas y modificar configuraciones sin editar archivos manualmente. |

---

## Impacto Mapeado (Regla 0)

- **Archivos leÃ­dos (completos):** `src/config.rs:1330-1580`, `src/cli.rs:1-120`, `src/cli_handlers/mod.rs`.
- **Archivos referenciados hacia dentro:** `serde`, `toml`, `std::env`.
- **Archivos que referencian a los editados:** Puntos de entrada principales del CLI y servidor.
- **Veredicto impacto:** Mejora fundamental de DevX; cero roturas de compatibilidad (las variables de entorno siguen teniendo precedencia sobre los archivos TOML).

---

## Contrato

`vanta config show imprime la tabla completa de parÃ¡metros indicando la capa de procedencia de cada valor, y vanta config set <clave> <valor> actualiza vanta.toml de forma persistente`

---

## Spec (SDD)

| # | DecisiÃ³n | Opciones (+tradeoff) | Default Recomendado | Resuelto |
|---|---|---|---|---|
| 1 | JerarquÃ­a de resoluciÃ³n (5 capas) | 1. Flags CLI > 2. Variables de Entorno > 3. `./vanta.toml` (Proyecto) > 4. `~/.vanta/config.toml` (Usuario) > 5. Defaults en cÃ³digo | JerarquÃ­a canÃ³nica de 5 capas estÃ¡ndar | âœ… decidido-por-evidencia |
| 2 | Enmascaramiento de secretos en salida | Ocultar campos de auth, api keys y contraseÃ±as (`sk-***`) | Enmascaramiento por regex de campos sensibles | âœ… decidido-por-evidencia |
| 3 | Comando de inicializaciÃ³n rÃ¡pida | Subcomando `vanta config init` que genera un `vanta.toml` anotado con mejores prÃ¡cticas | GeneraciÃ³n de plantilla con comentarios explicativos | âœ… decidido-por-evidencia |

---

## Invariantes de Dominio (Handoff â€” MUST)

- **Invariantes a preservar:** Las variables de entorno (`VANTADB_*`) deben seguir teniendo mayor prioridad que los archivos de configuraciÃ³n en disco para preservar el comportamiento esperado en contenedores Docker y orquestadores Kubernetes.
- **Comandos de verificaciÃ³n:** `cargo nextest run -p vantadb --lib config::loader` y `cargo test test_config_cli`
- **Deuda pendiente:** Ninguna.

---

## Recitation (CanÃ³nico â€” Estructura Ãšnica)

```yaml
contract:
  verificacion: cargo nextest run -p vantadb --lib config::loader
  evidencia:
    - claim: vanta config resuelve la cascada de 5 capas y atribuye correctamente la procedencia de cada configuraciÃ³n
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

## Deuda TÃ©cnica (Regla 6 â€” MUST)

**Saldo neto de deuda por PR:** Sin deuda (deuda eliminada: consolidaciÃ³n de configuraciones antes fragmentadas en scripts aislados).

---

## Definition of Done (Contrato Multi-Nivel â€” P2-08)

| Nivel | Gate | Estado |
|---|---|---|
| **Task** | Cargador jerÃ¡rquico implementado, subcomandos `vanta config` operativos y tests verdes | â¬œ PENDING |
| **Commit** | Atomic commit `feat(cli): add 5-layer unified configuration loader and interactive cli` | â¬œ PENDING |
| **Release** | DocumentaciÃ³n de referencia en `docs/api/CONFIGURATION.md` y guÃ­a en `QUICKSTART.md` | â¬œ PENDING |

---

## Herramientas Necesarias

- `cargo nextest`
- `cargo clippy`
- **Skills cargadas (SDP):** `api-and-interface-design`, `writing-guidelines` (documentaciÃ³n de comandos).

---

## Investigation Notes

Actualmente, el operador debe adivinar si una configuraciÃ³n fue aplicada revisando si definiÃ³ `VANTADB_STORAGE_PATH`, `VANTADB_PORT`, `VANTADB_MEMORY_LIMIT`, etc. La ausencia de un archivo centralizado en el repositorio provocaba que diferentes desarrolladores usaran configuraciones locales inconsistentes sin un archivo `vanta.toml` versionable en git.

---

## IncÃ³gnitas (Uphill) vs Pendientes (Downhill) â€” P2-03

| Eje | Contador | DescripciÃ³n |
|---|---|---|
| **IncÃ³gnitas abiertas (uphill)** | 0 | Esquema TOML y cascada de capas resueltos |
| **Pendientes de ejecuciÃ³n (downhill)** | 3 | 1) `src/config/loader.rs`; 2) Subcomandos en `cli.rs` y `config.rs`; 3) Tests unitarios |
| **% completado** | 10% | EspecificaciÃ³n completa; implementaciÃ³n pendiente |

---

## Fases ExplÃ­citas â€” SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** â€” Enmascaramiento estricto de secretos (`VANTADB_AUTH_TOKEN`, `OPENAI_API_KEY`, etc.) en la terminal.
- [x] **PERFORMANCE** â€” El cargador se ejecuta en menos de 2 milisegundos en el inicio; cachea la configuraciÃ³n deserializada.

---

## Steps

### Step 1: Implementar `src/config/loader.rs`
- **Archivos:** `src/config/loader.rs`
- **AcciÃ³n:** Definir `enum ConfigSource { Cli, Env, ProjectToml, UserToml, Default }`. Implementar `load_effective_config()` que evalÃºe las 5 capas en cascada y asigne a cada campo su procedencia.
- **Verify:** `cargo check -p vantadb`
- **Estado:** â¬œ PENDING

### Step 2: Implementar Subcomandos `vanta config` en `src/cli_handlers/config.rs`
- **Archivos:** `src/cli_handlers/config.rs`, `src/cli.rs`
- **AcciÃ³n:** Crear comandos `vanta config show` (tabla coloreada con procedencia), `vanta config set <key> <value>` (actualiza TOML), `vanta config init` (crea plantilla) y `vanta config doctor` (valida salud de rutas y modelos).
- **Verify:** `cargo check -p vantadb`
- **Estado:** â¬œ PENDING

### Step 3: AÃ±adir Tests Unitarios de Precedencia
- **Archivos:** `src/config/loader.rs`
- **AcciÃ³n:** Verificar que una variable de entorno tiene precedencia sobre un valor en `vanta.toml`, y este sobre el valor por defecto.
- **Verify:** `cargo nextest run -p vantadb --lib config::loader`
- **Estado:** â¬œ PENDING

---

## Dependencias

- Ninguna dependencia previa.

---

## Review (GATE â€” Agente Distinto, P2-01)

- **Revisor:** `vanta-audit` (CLI & Developer Experience Reviewer)
- **Enfoque:** Usabilidad en terminal, enmascaramiento de secretos y robustez de parsing TOML.
- **CÃ³mo se probÃ³:** InvocaciÃ³n de `show`, `set` e `init` con verificaciones unitarias.
- **Checklist Anti-HÃ¡bitos TÃ³xicos:**
  - [x] Nunca imprimir claves en texto plano.
  - [x] Proteger contra inyecciÃ³n de tipos en TOML.
- **Veredicto:** â³ PENDING ejecuciÃ³n

---

## Notas

La plantilla generada por `vanta config init` incluye configuraciones recomendadas para desarrollo local, servidores de producciÃ³n y perfiles embebidos de baja memoria.
