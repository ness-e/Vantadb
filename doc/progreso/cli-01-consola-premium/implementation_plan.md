# Plan de Implementación: Fase CLI-01 (Modernización y Profesionalización de la Interfaz de Línea de Comandos)

Este plan detalla el diseño e implementación para transformar la interfaz de línea de comandos actual `vanta-cli` en una herramienta moderna, interactiva y robusta que cumpla con los estándares premium de DevEx (Developer Experience). Se reemplazará el análisis de argumentos manual por `clap` v4 con soporte de autocompletado multi-shell, y se mejorará la presentación visual con tablas, colores HSL legibles y barras/spinners de progreso (`indicatif` / `console`).

---

## User Review Required

> [!IMPORTANT]
> **Adición de Dependencias de CLI en el Core:**
> Añadiremos `clap` (con la feature `derive` y `env`) y `clap_complete` como dependencias opcionales bajo la feature `cli` del core. Esto mantiene la biblioteca libre de dependencias no deseadas cuando se utiliza sin la interfaz de comandos.
> 
> **Nuevo Esquema de Subcomandos Globales:**
> Se unificará el parámetro `--db <path>` como bandera global opcional. Si no se provee, la CLI buscará la variable de entorno `VANTA_DB`. Esto optimiza la productividad del desarrollador al evitar repetir la ruta de base de datos en cada invocación.

---

## Open Questions

> [!NOTE]
> **Autocompletado Automatizado o Manual:**
> ¿Prefieres que los scripts de autocompletado se generen al vuelo mediante un subcomando dedicado (ej. `vanta-cli completions --shell zsh`) o mediante un script de build (`build.rs`) automático al compilar? 
> Recomendamos un subcomando dedicado por portabilidad y facilidad de instalación por parte del usuario final.

---

## Proposed Changes

### [Component: Core Manifest]

#### [MODIFY] [Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/Cargo.toml)
*   Agregar `clap` y `clap_complete` a la sección `[dependencies]` de forma opcional.
*   Actualizar la feature `cli` para habilitarlas.

```toml
[dependencies]
clap = { version = "4.4", features = ["derive", "env"], optional = true }
clap_complete = { version = "4.4", optional = true }

[features]
cli = ["dep:indicatif", "dep:console", "dep:clap", "dep:clap_complete"]
```

---

### [Component: CLI Entrypoint]

#### [MODIFY] [src/bin/vanta-cli.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/bin/vanta-cli.rs)
*   Reescribir el archivo completo utilizando la macro `clap::Parser`.
*   Definir la estructura de subcomandos:
    *   `put`: Guarda un valor. Argumentos: `--namespace <ns>`, `--key <key>`, `--payload <text>`.
    *   `get`: Recupera un valor. Argumentos: `--namespace <ns>`, `--key <key>`.
    *   `list`: Lista llaves/valores. Argumentos: `--namespace <ns>`.
    *   `rebuild-index`: Reconstruye todos los índices de la base de datos (con feedback visual de progreso).
    *   `audit-index`: Valida la integridad del índice de texto. Argumentos: `--namespace <ns>` [opcional], `--json` [flag], `--deep` [flag].
    *   `repair-text-index`: Repara el índice de texto si hay desviaciones.
    *   `export`: Exporta registros a un archivo JSON. Argumentos: `--namespace <ns>` [opcional], `--out <file>`.
    *   `import`: Importa registros desde un archivo JSON. Argumentos: `--in <file>`.
    *   `query`: Ejecuta una consulta estructurada directa en IQL/híbrida. Argumento posicional: `<query>`.
    *   `status`: Diagnóstico de salud (backend, almacenamiento en bytes, número de nodos, capacidades del hardware).
    *   `completions`: Genera scripts de autocompletado en consola para `bash`, `zsh`, `fish`, `powershell`. Argumento: `--shell <shell>`.

#### Diseño de Experiencia Visual (UX Premium):
*   **Status Dashboard (`status`)**:
    *   Mostrará un panel estético con colores HSL (`console`), indicando el backend activo (RocksDB/Fjall), la ruta, si es de sólo lectura y los recursos del sistema disponibles.
*   **Formateo Tabular (`list` / `query`)**:
    *   Las salidas se alinearán en tablas limpias con cabeceras formateadas y divisores de celdas visuales.
*   **Spinners de Progreso (`rebuild-index` / `audit-index`)**:
    *   Se inyectará un spinner de progreso interactivo de `indicatif` durante la reconstrucción o auditoría profunda para mejorar la retroalimentación visual.

---

## Verification Plan

### Manual Verification (Solicitado al Usuario)

Para verificar el correcto funcionamiento y profesionalismo de la nueva CLI, te solicitaré ejecutar los siguientes comandos y verificar su comportamiento visual y sintáctico:

1.  **Validar menú de ayuda autogenerado**:
    `cargo run --features cli --bin vanta-cli -- --help`

2.  **Verificar estado del motor**:
    `cargo run --features cli --bin vanta-cli -- --db ./db status`

3.  **Probar consulta con salida tabular**:
    `cargo run --features cli --bin vanta-cli -- --db ./db list --namespace test`

4.  **Generar script de autocompletado para PowerShell / Zsh**:
    `cargo run --features cli --bin vanta-cli -- --db ./db completions --shell powershell`
