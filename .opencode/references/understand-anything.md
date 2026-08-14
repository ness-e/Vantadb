# Understand-Anything

> Movido desde `.opencode/AGENTS.md` — referencia on-demand. Consultar para preguntas arquitectónicas, narrativa, onboarding y visualización. Si editas, actualiza también el puntero en AGENTS.md.

Understand-Anything produce un **knowledge graph LLM-powered** (1917 nodos, 1120 edges, 32 capas, 14 tour steps) en `.understand-anything/knowledge-graph.json`. Complementa a CodeGraph para preguntas arquitectónicas y narrativa humana.

## CodeGraph vs Understand-Anything — Guía de decisión

| Situación | Herramienta | Por qué |
|-----------|------------|---------|
| "¿Dónde está definida la función X?" | **CodeGraph** | Index pre-construido, respuesta en ms |
| "¿Qué llama a esta función?" | `codegraph_explore` | Call paths precisos, resuelve dispatch dinámico |
| "¿Qué se rompe si cambio X?" | `codegraph_explore "X"` | Blast radius vía código fuente |
| "¿Cómo está estructurada la arquitectura?" | **Understand-Anything** | 32 capas con descripciones narrativas |
| "Dame un tour del código base" | **Understand-Anything** | Tour guiado de 14 pasos desde entry point |
| "Explica este módulo en detalle" | `skill understand-explain` | Análisis narrativo contextual |
| "¿Qué tests ejecutar?" | `git diff --name-only \| codegraph_explore` | Conectado al git diff |
| "Onboarding para nuevo dev" | `skill understand-onboard` | Genera guía de onboarding interactiva |
| "¿Cuál es el dominio de negocio?" | `skill understand-domain` | Extrae flujos de dominio del grafo |
| "¿Qué cambió en este PR?" | `skill understand-diff` | Analiza diff contra el grafo existente |

**Regla general**: CodeGraph primero para todo lo que sea símbolos/código preciso. Understand-Anything para contexto arquitectónico, narrativa, onboarding y visualización.

## Slash Commands (Understand-Anything nativo)

El proyecto [Egonex-AI/Understand-Anything](https://github.com/Egonex-AI/Understand-Anything) expone estos comandos que el agente escribe directamente en la consola:

| Comando | Qué hace |
|---------|----------|
| `/understand` | Escanea repo, construye grafo en `.understand-anything/knowledge-graph.json` |
| `/understand --auto-update` | Activa hook post-commit para actualizaciones incrementales |
| `/understand --full` | Rebuild completo del grafo |
| `/understand-chat [pregunta]` | Chat contextualizado en la arquitectura del sistema |
| `/understand-dashboard` | Panel visual interactivo en navegador |
| `/understand-explain [ruta]` | Análisis aislado de un archivo específico |
| `/understand-diff` | Examina cambios staged/unstaged y predice impacto |
| `/understand-onboard` | Genera Guided Tours para onboarding |
| `/understand-domain` | Agrupa código por entidades de negocio |
| `/understand-knowledge [ruta]` | Analiza documentación Markdown externa |

## Alternativa: Agent Skills

Los skills en `C:\Users\Eros\.agents\skills\` envuelven la misma funcionalidad vía `skill <nombre>`:

| Skill | Comando OpenCode | Qué hace |
|-------|-----------------|----------|
| `understand` | `skill understand` | Pipeline completo: escanea, analiza y genera grafo |
| `understand-chat` | `skill understand-chat` | Chat contextual sobre el codebase |
| `understand-explain` | `skill understand-explain` | Explicación profunda de archivo/módulo |
| `understand-diff` | `skill understand-diff` | Analiza git diff contra el grafo |
| `understand-domain` | `skill understand-domain` | Extrae conocimiento de dominio de negocio |
| `understand-knowledge` | `skill understand-knowledge` | Analiza wikis Markdown → grafo |
| `understand-onboard` | `skill understand-onboard` | Guía de onboarding interactiva |
| `understand-dashboard` | `skill understand-dashboard` | Visor web interactivo del grafo |

## Estado actual

El grafo ya está generado en `.understand-anything/knowledge-graph.json`:

```
/understand --auto-update        # incremental post-commit
/understand --full               # rebuild completo
skill understand                 # misma funcionalidad vía skill
```

## Flujo recomendado: CodeGraph + Understand-Anything sin conflictos

1. **Para navegación diaria** → usa CodeGraph (`codegraph_explore`). Es más rápido, determinístico, y no gasta tokens LLM en re-análisis.
2. **Para entender arquitectura** → `/understand-chat "pregunta"` o `skill understand-chat`. El grafo ya existe, no necesita regenerarse.
3. **Para onboarding/review** → `/understand-explain` o `skill understand-explain`. Usan el grafo existente.
4. **Solo regenera si**: cambia la estructura del proyecto (nuevos módulos grandes) o quieres un análisis más fresco.
5. **NUNCA** ejecutes `/understand --full` a menos que sea necesario — el pipeline actual ya cubre 790 archivos y consumió ~158s de subagentes.

## Referencia del grafo

```json
{
  "nodes": [{"id": "file:src/engine.rs", "type": "file", "name": "engine.rs", "summary": "In-memory storage engine", "tags": ["storage", "core"]}],
  "edges": [{"source": "file:src/engine.rs", "target": "file:src/storage/mod.rs", "type": "imports", "direction": "directed", "weight": 0.7}],
  "layers": [{"id": "layer:core-engine", "name": "Core Engine", "description": "In-memory engine and storage backends", "nodeIds": ["file:src/engine.rs", ...]}],
  "tour": [{"order": 1, "title": "Project Overview", "description": "Start with README", "nodeIds": ["document:README.md"]}]
}
```

## Capas arquitectónicas (32 total)

Las principales: `core-engine`, `storage-backends`, `vector-index`, `web-frontend`, `python-bindings`, `typescript-sdk`, `integration-wrappers`, `dev-tooling`, `tests`, `documentation`, `ci-cd`, `wasm`, `enterprise`, `mcp`.
