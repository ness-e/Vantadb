# `sec-codeql-30.yml` — SEC: CodeQL — Analysis

## ¿Qué hace?

Ejecuta el análisis estático de seguridad CodeQL de GitHub sobre el código Rust del proyecto para detectar vulnerabilidades.

## ¿Cómo lo hace?

Un solo job `analyze`:

1. Inicializa CodeQL con lenguaje `rust` en modo `manual` (build manual)
2. Build del workspace completo: `cargo build --workspace`
3. Ejecuta el análisis CodeQL: `github/codeql-action/analyze`

CodeQL aplica queries de seguridad predefinidas que buscan patrones de vulnerabilidades conocidas.

## ¿Qué tests usa?

No ejecuta tests. Usa el motor de análisis estático CodeQL.

## ¿Qué verifica?

- Vulnerabilidades de seguridad en Rust: desbordamientos, inyecciones, uso de memoria insegura, etc.
- Bugs detectables mediante análisis estático
- Malas prácticas de seguridad

## Funcionalidad final

Detección automatizada de vulnerabilidades de seguridad en el código fuente mediante análisis estático semántico con CodeQL.

## ¿Cuándo se ejecuta?

- **Push** a `main`
- **Pull Request** a `main`
- **Semanal** (domingo 00:00 UTC) vía `schedule`
- **Workflow dispatch** manual
