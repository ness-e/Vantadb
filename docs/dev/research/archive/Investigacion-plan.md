> ## Informe de validación (2026-09-08, verificado en código + internet + APIs)
> Este documento fue auditado contra: `vantadb-python/src/lib.rs` (firmas PyO3), `vantadb-ts/src/{vantadb.ts,types.ts}`, `src/wal.rs` + `src/gds.rs`, PyPI/npm registries, `git tag v0.5.0` (2026-08-01), GitHub API (labels, releases, issues, discussions) y docs oficiales (HN, Reddit, PyPI, npm, GitHub, shields.io).
> **Regla de marcas del documento:** [REAL v0.5.0] = existe y corre hoy · [PROPUESTA] = especificación futura, no publicar como existente · [BLOQUEADO ness-e] = requiere editar `ness-e/Vantadb` (prohibido sin orden del owner).
> Hallazgo crítico: 9 ejemplos de código estaban rotos (8 corregidos aquí, 1 imposible) + 1 bug silencioso (TTL en metadata nunca expira). Ver Apéndice Z con la tabla API real.

# Plan Detallado de Lanzamiento Híbrido para SyntropyOS + VantaDB

---


## Fase A: Validación privada + Gate de Early Access (criterio de salida)

> Trasplantado de la corrección auditada: es el gate que decide si se anuncia o no. Sin estos checks en verde, no hay anuncio público.

### A.1 Stranger-test (2-3 personas, sin tu ayuda)

Pide que sigan solo el README y registren por persona:

```
# Prueba de instalación: usuario-01

Fecha: / OS: / Python-Node: / Tiempo hasta instalación: / Tiempo hasta primer resultado:
Bloqueos: / Preguntas: / Errores: / Cambios derivados: [ ]
```

### A.2 Gate (todo SI o no se anuncia)

**Producto:** flujo principal desde cero - instala en limpio - quickstart ejecutado por un tercero - 0 críticos - versión idéntica en código, paquete y docs.
**Calidad:** unitarios en ops críticas + 1 integración + CI verde en limpio + sin secretos + LICENSE + changelog de la versión exacta.
**Documentación:** README con estado Early Access + instalación + quickstart PROBADO + limitaciones + cómo reportar + cómo contribuir.
**Comunidad:** 1 canal soporte + 1 anuncios + bienvenida + 1 invite válido (canónico `g8nqB3NtXt`).
**Comunicación:** anuncio listo + cero funciones no existentes + cero métricas no verificadas.

### A.3 Objetivo medible

> **Que 5 personas instalen, entiendan, usen un flujo real y digan qué les impidió seguir.** Solo entonces se decide qué terminar, separar o comunicar.

## Fase 0: Preparación (Semana 0)

### Objetivo:

Dejar TODO listo antes de anunciar públicamente. Sin cabos sueltos.

---

### ✅ Tareas Críticas (DEBES terminar antes de publicar)

---

#### 1. **VantaDB - Estado del Código**

**Por qué importa:**

Si alguien instala VantaDB y no funciona, se va y no vuelve. La primera impresión es crítica.

**Tareas Detalladas:**

#### 1.1. README de `ness-e/Vantadb` Actualizado

**Formato de README (copia y adapta):**

```
# VantaDB

> **Estado:** v0.5.0 - Early Access (Beta)

VantaDB es el holón de memoria de SyntropyOS. Memoria persistente gobernada para agentes de IA.

**⚠️ Importante:** Esto es Early Access. Funciona para casos de uso básicos, pero algunas features avanzadas (entity resolution, conflict detection) están en desarrollo. Si buscas algo production-ready, espera a v1.0.0 (TBD).

---

## ¿Qué funciona?

- ✅ Persistencia durable (WAL + fsync + CRC32C)
- ✅ Búsqueda híbrida (BM25 + HNSW + RRF)
- ✅ Filtrado por metadatos y namespaces
- ✅ TTL y expiración automática
- ✅ Grafos de entidades (BFS, DFS, PageRank)
- ✅ Supersession (marcar registros como obsoletos)
- ✅ Bindings Python (`pip install vantadb-py`) y TypeScript (`npm install vantadb`)

## ¿Qué NO funciona aún?

- ❌ Entity resolution automática (detectar duplicados como "Acme Corp" = "Acme Corporation")
- ❌ Conflict detection (detectar contradicciones automáticamente)
- ❌ Skills extract (memoria procedimental)
- ❌ Consolidación automática L2→L3
- ❌ Auditoría automática de vigencia

## Quickstart

### Python

```bash
pip install vantadb-py
```

```python
import vantadb

db = vantadb.VantaDB("./my_brain")
db.put(namespace="prefs", key="editor", payload="usa tabs")
result = db.search_memory("prefs", [], text_query="formateo")
```

### TypeScript

```bash
npm install vantadb
```

```typescript
import { VantaDB } from "vantadb";

const db = VantaDB.create();
db.put({ namespace: "prefs", key: "editor", payload: "usa tabs" });
const result = db.search({ namespace: "prefs", query_vector: [], text_query: "formateo" });
```

## Roadmap

- **v0.5.0** (actual): Early Access - persistencia + búsqueda híbrida
- **v0.7.0** (TBD): Governance - entity resolution, conflict detection
- **v1.0.0** (TBD): Production-ready - todas las features de governance

## ¿Quieres contribuir?

Buscamos ayuda con:
- 📖 Docs (tutorials, ejemplos, FAQs)
- 🧪 Tests (unitarios, de integración)
- 🐛 Reportar bugs
- 💡 Feedback de uso real

Únete a Discord o abre un issue en GitHub.

## Licencia

Apache 2.0. Uso personal y comercial permitido.

---

**SyntropyOS** — Orden desde el caos.
```

**Checklist para README:**

- Estado claro ("v0.5.0 - Early Access")
- Qué funciona (lista concreta con ✅)
- Qué NO funciona (lista honesta con ❌)
- Quickstart (código copiable en Python y TS)
- Roadmap (v0.7.0, v1.0.0 con fechas aproximadas)
- Llamado a contribuir (docs, tests, feedback)
- Enlace a Discord
- Licencia clara (Apache 2.0)

---

#### 1.2. Docs Básicas en `vantadb.vercel.app`

**Estructura de Docs (mínima viable):**

```
vantadb.vercel.app/
├── index.md (Homepage)
├── installation.md
├── quickstart.md
├── api-reference.md
├── faq.md
└── roadmap.md
```

**Contenido de cada página:**

#### `index.md` (Homepage)

```
# VantaDB

> Memoria persistente gobernada para agentes de IA.

**Estado:** v0.5.0 - Early Access

VantaDB es el holón de memoria de SyntropyOS. Provee persistencia durable, búsqueda híbrida (BM25 + HNSW + RRF), y gobernanza del ciclo de vida de la memoria.

## Empezar

1. Instalación
2. Quickstart
3. API Reference
4. FAQ

## Estado del Proyecto

- ✅ Persistencia durable (WAL + fsync)
- ✅ Búsqueda híbrida (BM25 + HNSW + RRF)
- ✅ TTL y expiración
- ❌ Entity resolution (v0.7.0)
- ❌ Conflict detection (v0.7.0)

Ver Roadmap

## Únete a la Comunidad

- GitHub
- Discord
- SyntropyOS
```

#### `installation.md`

```
# Instalación

## Python

```bash
pip install vantadb-py
```

Requiere Python 3.11+.

## TypeScript

```bash
npm install vantadb
```

Requiere Node 18+ o Bun.

## Verificar Instalación

### Python

```python
import vantadb
print(vantadb.__version__)  # Debería imprimir "0.5.0"
```

### TypeScript

```typescript
import { VantaDB } from "vantadb";
console.log("VantaDB OK");  // Sin API de versión en runtime TS — ver package.json
```

## Problemas Comunes

### Error: "No module named 'vantadb'"

Asegúrate de estar usando Python 3.11+ y de haber instalado en el environment correcto.

### Error: "Cannot find module 'vantadb'"

Asegúrate de estar usando Node 18+ o Bun.

Reportar un problema
```

#### `quickstart.md`

```
# Quickstart (5 minutos)

## Python

```python
import vantadb

# 1. Crear base de datos
db = vantadb.VantaDB("./my_brain")

# 2. Guardar un registro
db.put(
    namespace="prefs",
    key="editor",
    payload="usa tabs"
)

# 3. Buscar registros (`query_vector` requerido; `[]` = solo texto BM25)
result = db.search_memory(
    "prefs",
    [],
    text_query="formateo"
)
print(result)

# 4. Usar TTL (`ttl_ms` es parámetro top-level, NO dentro de metadata)
db.put(
    namespace="sesion",
    key="token-xyz",
    payload="Bearer abc",
    ttl_ms=90*24*3600*1000  # 3 meses
)

# 5. Usar grafos (nodos por ID numérico u128, no por namespace+key)
db.add_edge(1, 2, "decidio")
result = db.graph_bfs([1], 2)
print(result)
```

## TypeScript

```typescript
import { VantaDB } from "vantadb";

// 1. Crear base de datos
const db = VantaDB.create();

// 2. Guardar un registro
db.put({
    namespace: "prefs",
    key: "editor",
    payload: "usa tabs"
});

// 3. Buscar registros (query_vector requerido; [] = solo texto BM25)
const result = db.search({
    namespace: "prefs",
    query_vector: [],
    text_query: "formateo"
});
console.log(result);

// 4. Usar TTL (top-level, NO dentro de metadata)
db.put({
    namespace: "sesion",
    key: "token-xyz",
    payload: "Bearer abc",
    ttl_ms: 90*24*3600*1000  // 3 meses
});

// 5. Usar grafos (IDs numéricos, no namespace+key)
db.addEdge(1, 2, "decidio");
const result = db.graphBfs([1], 2);
console.log(result);
```

## Siguiente Paso

- API Reference para más detalles
- FAQ para preguntas comunes
```

#### `api-reference.md`

```
# API Reference

## VantaDB Class

### `constructor(path: string)`

Crea una nueva base de datos.

```python
db = vantadb.VantaDB("./my_brain")
```

### `put(namespace, key, payload, metadata=None, vector=None, ttl_ms=None)`

Guarda un registro. `ttl_ms` es top-level (si va dentro de `metadata` queda como campo inerte y el registro NUNCA expira).

```python
db.put(
    namespace="prefs",
    key="editor",
    payload="usa tabs",
    ttl_ms=90*24*3600*1000  # Opcional: TTL en ms, top-level
)
```

### `search_memory(namespace, query_vector, filters=None, text_query=None, top_k=10)`

Busca registros. `query_vector` es posicional requerido; pasa `[]` para búsqueda solo-texto (BM25).

```python
result = db.search_memory(
    "prefs",
    [],
    text_query="formateo",
    top_k=5
)
```

### `add_edge(source_id, target_id, label, weight=None, created_at_ms=None)`

Agrega una arista al grafo. Los nodos se identifican por ID numérico (u128), no por namespace+key.

```python
db.add_edge(1, 2, "decidio")
```

### `graph_bfs(roots, max_depth=999999, direction="Forward")`

BFS en el grafo desde IDs raíz.

```python
result = db.graph_bfs([1], 2)
```

Ver API completa en GitHub
```

#### `faq.md`

```
# FAQ

## ¿Qué es VantaDB?

VantaDB es memoria persistente gobernada para agentes de IA. Es el holón de memoria de SyntropyOS.

## ¿Está listo para producción?

No aún. v0.5.0 es Early Access. Funciona para casos de uso básicos, pero features avanzadas (entity resolution, conflict detection) están en desarrollo. Espera a v1.0.0 (TBD) para producción.

## ¿Cómo se compara con Qdrant/Chroma?

VantaDB es local-first (sin servidor), con gobernanza del ciclo de vida de la memoria. Qdrant/Chroma son vector stores pasivos.

## ¿Puedo usarlo comercialmente?

Sí. Licencia Apache 2.0 permite uso comercial.

## ¿Cómo contribuyo?

Buscamos ayuda con docs, tests, y feedback. Únete a Discord o abre un issue en GitHub.

## ¿Qué sigue?

v0.7.0 (TBD): entity resolution, conflict detection. v1.0.0 (TBD): production-ready.

Ver Roadmap
```

#### `roadmap.md`

```
# Roadmap

## v0.5.0 (Actual - Early Access)

- ✅ Persistencia durable (WAL + fsync)
- ✅ Búsqueda híbrida (BM25 + HNSW + RRF)
- ✅ TTL y expiración
- ✅ Grafos de entidades

## v0.7.0 (TBD - Governance)

- 🔵 Entity resolution automática
- 🔵 Conflict detection y resolución
- 🔵 Skills extract (memoria procedimental)
- 🔵 Consolidación automática L2→L3

## v1.0.0 (TBD - Production-Ready)

- 🔵 Todas las features de governance completas
- 🔵 Docs completas
- 🔵 Tests de integración
- 🔵 Benchmarks de performance
- 🔵 Casos de uso en producción

## Más Allá

- 🔵 Iris (visión)
- 🔵 Cardinal (orientación)
- 🔵 Sage (aprendizaje)
- 🔵 Execute (ejecución)
- 🔵 Plan (planificación)

Ver Roadmap de SyntropyOS
```

**Checklist para Docs:**

- Homepage clara con estado del proyecto
- Installation (Python y TS)
- Quickstart (código copiable)
- API reference (aunque sea mínima)
- FAQ (al menos 5 preguntas)
- Roadmap (v0.7.0, v1.0.0)
- Enlaces a GitHub, Discord, SyntropyOS

---

#### 1.3. Tests Mínimos

**Tareas:**

- Tests unitarios para funciones críticas:
    - `put()`, `get_memory()`, `search_memory()`, `add_edge()`, `graph_bfs()`
    - Al menos 1 test por función
- Tests de integración:
    - Flujo completo: put → search → delete
    - Al menos 1 test de integración
- CI en GitHub Actions:
    - Tests corren en cada push
    - Badge de CI en README
- Tests locales antes de commit:
    - Documenta en CONTRIBUTING.md cómo correr tests

**Formato de CI (referencia, NO copiar literal — el repo es Rust+maturin, no Python puro):**

> Estado real verificado: `ness-e/Vantadb` ya tiene **24 workflows** (`ci-rust-10.yml`, `ci-gate.yml`, `release-*.yml`…) con actions pineadas por SHA (`setup-python v7`, `setup-node v4.4.0`). No existe `requirements.txt` en raíz ni `tests/python/` (los tests son `tests/*.rs` + `vantadb-python/tests/test_sdk.py`). Antes de escribir un `ci.yml` nuevo, auditar lo existente. Si se crea un job mínimo para bindings, forma correcta:

```
name: CI-bindings

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test-python:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: actions/setup-python@v5
        with:
          python-version: '3.11'
      - run: pip install maturin
      - run: maturin develop --release
        working-directory: vantadb-python
      - run: pytest vantadb-python/tests/test_sdk.py

  test-typescript:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: actions/setup-node@v5
        with:
          node-version: '20'
      - run: npm install
        working-directory: vantadb-ts
      - run: npm test
        working-directory: vantadb-ts
```

> Nota: `checkout@v4`/`setup-node@v4` obsoletos (latest v7, runtime Node 24). `engines` en package.json es solo advisory salvo `engine-strict`.

**Checklist para Tests:**

- Tests unitarios para funciones críticas
- Al menos 1 test de integración
- CI corriendo en cada push
- Badge de CI en README
- Docs de cómo correr tests localmente

---

#### 1.4. Issues Templates

**Formato de Templates (`.github/ISSUE_TEMPLATE/`):**

#### `bug_report.md`

```
---
name: Bug Report
about: Reportar un bug
title: '[BUG] '
labels: bug
---

**Descripción del bug**

Qué pasó, qué esperabas que pasara.

**Para reproducir**

Pasos para reproducir el bug:

1. ...
2. ...
3. ...

**Código de ejemplo**

```python
# Tu código aquí
```

**Comportamiento esperado**

Qué debería haber pasado.

**Screenshots**

Si aplica, agrega screenshots.

**Environment**

- OS: [e.g. Ubuntu 22.04]
- Python/Node version: [e.g. 3.11, 18]
- VantaDB version: [e.g. 0.5.0]

**Contexto adicional**

Cualquier otra cosa que ayude a debuggear.
```

#### `feature_request.md`

```
---
name: Feature Request
about: Sugerir una feature
title: '[FEATURE] '
labels: enhancement
---

**¿Tu feature request está relacionado con un problema?**

Describe el problema. Ej: "Siempre que quiero hacer X, tengo que hacer Y manualmente."

**Describe la solución que te gustaría**

Qué debería pasar después de tu feature.

**Describe alternativas que consideraste**

Otras formas de resolver el problema.

**Contexto adicional**

Cualquier otra cosa (screenshots, ejemplos de uso, etc.).
```

#### `question.md`

```
---
name: Question
about: Hacer una pregunta
title: '[QUESTION] '
labels: question
---

**Tu pregunta**

Escribe tu pregunta aquí.

**Contexto**

Qué estás intentando hacer, por qué tienes esta pregunta.

**Qué intentaste**

Qué ya intentaste para resolver tu duda.
```

**Checklist para Issues Templates:**

- Bug report template
- Feature request template
- Question template
- Labels configurados en GitHub (bug, enhancement, question, etc.)

---

**Recomendación Final para VantaDB:**

No necesita ser perfecto, pero sí funcional. Si alguien instala VantaDB, debería funcionar sin errores críticos. Las docs deberían ser claras aunque sean mínimas. Los tests deberían correr sin fallar.

---

*(Continuaré con las demás tareas en el siguiente mensaje para no exceder el límite de tokens...)*

---

#### 2. **SyntropyOS - Organización GitHub**

**Por qué importa:**

La org de GitHub es tu "casa oficial". Si está desordenada, la gente asume que el proyecto está muerto o es amateur.

---

#### 2.1. `profile/README.md` Actualizado

**Formato de README de Organización (copia y adapta):**

```
# SyntropyOS

> **Orden desde el caos.**

SyntropyOS es un sistema operativo para agentes de IA. Construido con holones modulares: memoria (VantaDB), visión (Iris), orientación (Cardinal), aprendizaje (Sage), ejecución (Execute), planificación (Plan).

---

## Holones

| Holón | Estado | Descripción | Repo |
|-------|--------|-------------|------|
| **VantaDB** | 🟡 v0.5.0 (Early Access) | Memoria persistente gobernada | ness-e/Vantadb |
| **Iris** | 🔵 Planificado | Visión y percepción | - |
| **Cardinal** | 🔵 Planificado | Orientación y navegación | - |
| **Sage** | 🔵 Planificado | Aprendizaje y adaptación | - |
| **Execute** | 🔵 Planificado | Ejecución de acciones | - |
| **Plan** | 🔵 Planificado | Planificación de tareas | - |

---

## Empezar

1. VantaDB Quickstart
2. Roadmap
3. Manifiesto

---

## Comunidad

- 💬 Discord - Chat, soporte, anuncios
- 📚 Docs - Documentación oficial
- 🐛 Issues - Reportar bugs, sugerir features

---

## Contribuir

Buscamos ayuda con:
- 📖 Docs (tutorials, ejemplos, FAQs)
- 🧪 Tests (unitarios, de integración)
- 🐛 Reportar bugs
- 💡 Feedback de uso real
- 🌍 Traducciones (ES, PT, etc.)

Ver CONTRIBUTING.md

---

## Licencia

Apache 2.0. Uso personal y comercial permitido.

---

**SyntropyOS** — Orden desde el caos.
```

**Checklist para `profile/README.md`:**

- Tagline claro ("Orden desde el caos")
- Tabla de holones con estado (🟡 Early Access, 🔵 Planificado, 🟢 Production)
- Enlaces a VantaDB, Roadmap, Manifiesto
- Enlaces a Discord, Docs, Issues
- Llamado a contribuir (docs, tests, feedback, traducciones)
- Licencia clara (Apache 2.0)

---

#### 2.2. `profile/README_ES.md` Sincronizado

**Tareas:**

- Traduce `profile/README.md` al español
- Mantén misma estructura y enlaces
- Asegúrate de que enlaces funcionen (Discord, VantaDB, etc.)

**Formato de `README_ES.md`:**

```
# SyntropyOS

> **Orden desde el caos.**

SyntropyOS es un sistema operativo para agentes de IA. Construido con holones modulares: memoria (VantaDB), visión (Iris), orientación (Cardinal), aprendizaje (Sage), ejecución (Execute), planificación (Plan).

---

## Holones

| Holón | Estado | Descripción | Repo |
|-------|--------|-------------|------|
| **VantaDB** | 🟡 v0.5.0 (Acceso Temprano) | Memoria persistente gobernada | ness-e/Vantadb |
| **Iris** | 🔵 Planificado | Visión y percepción | - |
| **Cardinal** | 🔵 Planificado | Orientación y navegación | - |
| **Sage** | 🔵 Planificado | Aprendizaje y adaptación | - |
| **Execute** | 🔵 Planificado | Ejecución de acciones | - |
| **Plan** | 🔵 Planificado | Planificación de tareas | - |

---

## Empezar

1. VantaDB Quickstart
2. Roadmap
3. Manifiesto

---

## Comunidad

- 💬 Discord - Chat, soporte, anuncios
- 📚 Docs - Documentación oficial
- 🐛 Issues - Reportar bugs, sugerir features

---

## Contribuir

Buscamos ayuda con:
- 📖 Docs (tutorials, ejemplos, FAQs)
- 🧪 Tests (unitarios, de integración)
- 🐛 Reportar bugs
- 💡 Feedback de uso real
- 🌍 Traducciones (EN, PT, etc.)

Ver CONTRIBUTING.md

---

## Licencia

Apache 2.0. Uso personal y comercial permitido.

---

**SyntropyOS** — Orden desde el caos.
```

**Checklist para `README_ES.md`:**

- Misma estructura que `README.md`
- Traducción clara (no Google Translate)
- Enlaces funcionan
- Estado de holones actualizado

---

#### 2.3. Docs Canónicos en `.github`

**Estructura de `.github`:**

```
.github/
├── MANIFESTO.md
├── CONTRIBUTING.md
├── ROADMAP.md
├── SECURITY.md
├── CODE_OF_CONDUCT.md
├── CONTRIBUTORS.md
├── SUPPORT.md
└── FUNDING.yml (opcional)
```

**Contenido de cada archivo:**

#### `MANIFESTO.md`

```
# Manifiesto de SyntropyOS

## Visión

Un sistema operativo para agentes de IA que escala desde un agente personal hasta una flota global, manteniendo coherencia, trazabilidad y control.

## Principios

1. **Holones Modulares**: Memoria, visión, orientación, aprendizaje, ejecución, planificación. Cada holón es independiente pero interoperable.

2. **Gobernanza del Ciclo de Vida**: La memoria no es pasiva. Tiene TTL, supersession, auditoría de vigencia, entity resolution, conflict detection.

3. **Local-First**: Sin servidor. Tus datos, tu infra. Opcionalmente distribuido cuando es necesario.

4. **Open Source**: Apache 2.0. Uso personal y comercial permitido.

5. **Comunidad-Driven**: Construido con y para la comunidad. Feedback, contribuciones, traducciones bienvenidas.

## Por Qué

Los agentes de IA actuales son frágiles. No tienen memoria persistente, no pueden razonar sobre su propio estado, no pueden colaborar. SyntropyOS resuelve esto con holones modulares y gobernanza del ciclo de vida.

## Únete

- Discord
- GitHub
- Docs

---

**SyntropyOS** — Orden desde el caos.
```

#### `CONTRIBUTING.md`

```
# Contribuir a SyntropyOS

¡Gracias por querer contribuir! 🎉

## Cómo Contribuir

### 1. Reportar Bugs

Abre un issue en ness-e/Vantadb/issues con el template "Bug Report".

### 2. Sugerir Features

Abre un issue en ness-e/Vantadb/issues con el template "Feature Request".

### 3. Mejorar Docs

- Edita archivos `.md` en `ness-e/Vantadb/docs/`
- O sugiere cambios en Discord

### 4. Escribir Tests

- Tests unitarios en `ness-e/Vantadb/tests/`
- Tests de integración en `ness-e/Vantadb/tests/integration/`

### 5. Traducciones

- Traduce `profile/README.md` a tu idioma
- Abre un PR con la traducción

### 6. Código

- Fork `ness-e/Vantadb`
- Crea una rama `feature/tu-feature`
- Abre un PR

## Estándares de Código

- Python: PEP 8, type hints
- TypeScript: ESLint, TypeScript strict mode
- Tests: pytest (Python), vitest (TypeScript)

## Correr Tests

### Python

```bash
pip install maturin
maturin develop --release  # dentro de vantadb-python/
pytest vantadb-python/tests/test_sdk.py
```

### TypeScript

```bash
npm install
npm test
```

## Preguntas?

Únete a Discord o abre un issue.

---

**SyntropyOS** — Orden desde el caos.
```

#### `ROADMAP.md`

```
# Roadmap de SyntropyOS

## Q3 2026 (Actual)

- ✅ VantaDB v0.5.0 (Early Access)
- 🔵 VantaDB v0.6.0 (mejoras menores)
- 🔵 Docs completas de VantaDB

## TBD (tras gate de 5 installs)

- 🔵 VantaDB v0.7.0 (Governance)
  - Entity resolution automática
  - Conflict detection y resolución
  - Skills extract (memoria procedimental)
- 🔵 Iris v0.1.0 (visión - planificado)

## TBD (tras gate de 5 installs)

- 🔵 VantaDB v1.0.0 (Production-Ready)
- 🔵 Cardinal v0.1.0 (orientación - planificado)

## TBD+ (tras gate de 5 installs)

- 🔵 Sage v0.1.0 (aprendizaje - planificado)
- 🔵 Execute v0.1.0 (ejecución - planificado)
- 🔵 Plan v0.1.0 (planificación - planificado)

## Más Allá

- 🔵 Integración entre holones
- 🔵 Casos de uso en producción
- 🔵 Comunidad activa

Ver Manifiesto | Ver CONTRIBUTING
```

#### `SECURITY.md`

```
# Política de Seguridad

## Reportar Vulnerabilidades

Envía un email a `syntropyos.ia@gmail.com` con:

- Descripción de la vulnerabilidad
- Pasos para reproducir
- Impacto potencial
- Posible fix (si tienes)

**No** reportes vulnerabilidades en issues públicos.

## Respuesta

Te responderemos en < 48 horas confirmando recepción.

Te mantendremos informado del progreso.

## Gratificación

No ofrecemos bounties monetarios aún, pero:

- Te creditaremos en `CONTRIBUTORS.md`
- Te daremos rol especial en Discord
- Mención en anuncios de release

---

**SyntropyOS** — Orden desde el caos.
```

#### `CODE_OF_CONDUCT.md`

```
# Código de Conducta

## Nuestro Compromiso

En SyntropyOS, nos comprometemos a mantener un espacio abierto, amigable y seguro para todos, independientemente de:

- Edad, tamaño corporal, discapacidad visible o invisible
- Etnia, características sexuales, identidad o expresión de género
- Nivel de experiencia, educación, estatus socio-económico
- Nacionalidad, apariencia personal, raza, religión
- Identidad u orientación sexual

## Estándares de Comportamiento

### Comportamientos Positivos

- Usar lenguaje acogedor e inclusivo
- Respetar puntos de vista y experiencias diferentes
- Aceptar críticas constructivas con gracia
- Enfocarse en lo que es mejor para la comunidad
- Mostrar empatía hacia otros miembros

### Comportamientos Inaceptables

- Uso de lenguaje o imágenes sexualizadas, atención o avances sexuales
- Trolling, comentarios insultantes o despectivos, ataques personales o políticos
- Acoso público o privado
- Publicar información privada de otros (dirección física, email) sin permiso
- Otras conductas que podrían considerarse inapropiadas en un entorno profesional

## Responsabilidades de Moderación

Los moderadores de la comunidad son responsables de:

- Clarificar y hacer cumplir nuestros estándares de comportamiento aceptable
- Tomar acciones correctivas apropiadas y justas ante comportamientos inaceptables
- Remover, editar o rechazar comentarios, commits, código, issues, y otras contribuciones que no se alineen con este Código de Conducta

## Alcance

Este Código de Conducta aplica en todos los espacios de la comunidad (GitHub, Discord, eventos) y también aplica cuando un individuo representa oficialmente a la comunidad en espacios públicos.

## Enforcement

Instancias de comportamiento abusivo, acosador o inaceptable pueden ser reportadas a los moderadores de la comunidad en Discord o por email a `syntropyos.ia@gmail.com`.

Todas las quejas serán revisadas e investigadas y resultarán en una respuesta necesaria y apropiada a las circunstancias. Los moderadores de la comunidad están obligados a respetar la privacidad y seguridad de quienes reportan incidentes.

## Atribución

Este Código de Conducta está adaptado del Contributor Covenant, versión 2.0.

---

**SyntropyOS** — Orden desde el caos.
```

#### `CONTRIBUTORS.md`

```
# Contribuidores

¡Gracias a todos los que hacen SyntropyOS posible! 🎉

## Core Team

- @ness-e - Fundador, VantaDB

## Contribuidores Tempranos

*(Espacio para futuros contribuidores)*

## Cómo Ser Listado Aquí

1. Contribuye con código, docs, tests, traducciones, o feedback valioso
2. Abre un PR o issue en `SyntropyOS/.github`
3. Te agregaremos a esta lista

Ver CONTRIBUTING.md

---

**SyntropyOS** — Orden desde el caos.
```

#### `SUPPORT.md`

```
# Soporte

## ¿Necesitas Ayuda?

### Docs

- VantaDB Docs
- SyntropyOS Roadmap

### Discord

Únete a Discord para:

- Hacer preguntas
- Compartir ideas
- Reportar bugs
- Sugerir features

### GitHub Issues

- VantaDB Issues
- SyntropyOS Issues

### Email

Para temas sensibles (seguridad, privacidad): `syntropyos.ia@gmail.com`

## Tiempos de Respuesta Esperados

- Discord: < 24 horas (días hábiles)
- GitHub Issues: < 48 horas
- Email: < 72 horas

---

**SyntropyOS** — Orden desde el caos.
```

> ## Decisión ORG-14 — Dual license y donaciones: evaluado, NO ejecutar (2026-09-09, referencia completa)
>
> **Decisión:** mantener Apache 2.0. No `LICENSE.COMMERCIAL` casera, no dual anunciado, no headers masivos, no CLA ahora, no cambios en `ness-e/Vantadb`, `FUNDING.yml` pausado, sin enlaces de pago en repos.
> **Motivos:** sin migración (ORG-02), sin cliente que justifique asesoría legal (una comercial casera sería frágil), plataformas/retiros VE sin verificar por plataforma, tiers con beneficios crearían obligaciones (consultoría/SLA implícitos).
> **Rescatable:** Apache vigente; donación pura futura sin contraprestaciones (sin prioridad, soporte, consultoría, acceso, SLA ni reconocimiento obligatorio); tiers solo como cantidades sugeridas ($3-5/$10-15/$25-50/libre) con descargo explícito de no-compra; licencia comercial solo ante cliente real (conservar versiones Apache, revisar titularidad, asesoría, CLA previo).
> **Regla:** donación ≠ membresía ≠ soporte ≠ consultoría ≠ licencia comercial ≠ SaaS.
> **Reapertura:** tras migración o demanda comercial real, nunca antes del Early Access Gate. Ver `docs/dev/Backlog.md` ORG-14.

#### `FUNDING.yml` (Opcional — [PAUSADO hasta migrar VantaDB a Syntropy, decisión 2026-09-08])

```
github: ness-e
custom: ["https://buymeacoffee.com/ness"]  # Si tienes (confirmar titularidad de /ness antes)
```

**Checklist para `.github`:**

- `MANIFESTO.md` con visión y principios
- `CONTRIBUTING.md` con cómo contribuir
- `ROADMAP.md` con timeline claro
- `SECURITY.md` con cómo reportar vulnerabilidades
- `CODE_OF_CONDUCT.md` con estándares de comportamiento
- `CONTRIBUTORS.md` con lista de contribuidores
- `SUPPORT.md` con canales de ayuda
- `FUNDING.yml` ([PAUSADO hasta migración] — ver arriba)

---

#### 2.4. Discord Configurado

**Tareas Detalladas:**

#### 2.4.1. Canales Básicos

> Estado real: el servidor VantaDB Community YA tiene estructura (👋 WELCOME con rules/roles/announcements, 💬 COMMUNITY con general/off-topic/showcase/Stage, 🛠️ DEV con help/bug-reports/dev-chat/ideas, 🛡️ STAFF). Con 3 miembros, NO crear los 15 canales de abajo: aplicar la regla de 6 canales de la corrección §3 y crecer por demanda. Lo de abajo queda como referencia de estado final, no como tarea inmediata.

**Estructura de Canales:**

```
📌 INFORMACIÓN
├── #👋bienvenida (solo lectura, bienvenidas automáticas)
├── #📢anuncios (solo staff escribe)
├── #📜reglas (solo lectura)

💬 COMUNIDAD
├── #💬general (discusión libre)
├── #🧠vantadb-memoria (soporte técnico de VantaDB)
├── #🛠️soporte (ayuda general)
├── #💡ideas-holones (RFCs, feature requests)
├── #🎉logros (usuarios comparten lo que construyeron)

🔧 TÉCNICO
├── #📚docs (discusión de docs, PRs)
├── #🧪tests (discusión de tests, CI)
├── #🐛bugs (reporte de bugs)

🌍 INTERNACIONAL
├── #🇪🇸español (chat en español)
├── #🇧🇷português (chat en portugués)

🎮 OFF-TOPIC
├── #☕café (chat casual)
├── #🎮gaming (juegos)
├── #🎵música (música)
```

**Configuración de Cada Canal:**

- `#👋bienvenida`:
    - Solo lectura para todos excepto staff
    - Bot de bienvenidas automáticas (ej: MEE6, WelcomeBot)
    - Mensaje de bienvenida:
        
        ```
        ¡Bienvenido/a a SyntropyOS! 🎉
        
        - Lee las #📜reglas
        - Mira #📢anuncios para novedades
        - Presentate en #💬general
        - ¿Problemas con VantaDB? #🧠vantadb-memoria
        
        ¡Disfruta!
        ```
        
- `#📢anuncios`:
    - Solo staff escribe
    - Todos pueden leer
    - Usa para: releases, eventos, hitos
- `#📜reglas`:
    - Solo lectura
    - Contenido:
        
        ```
        # Reglas de SyntropyOS Discord
        
        1. **Sin spam/NSFW**: Mantén el canal limpio y profesional.
        2. **Vulnerabilidades → Email**: Reporta bugs de seguridad a `syntropyos.ia@gmail.com`, no aquí.
        3. **ES/EN bienvenidos**: Habla en el idioma que quieras. Usa canales internacionales si prefieres.
        4. **Respeto**: Sigue el Code of Conduct.
        5. **No autopromoción**: Sin spam de tus proyectos a menos que sea relevante.
        
        ¿Preguntas? Pregunta en #💬general o DM a un mod.
        ```
        
- `#💬general`:
    - Chat libre para todos
    - Moderación: medium (filtro de spam, links)
- `#🧠vantadb-memoria`:
    - Soporte técnico de VantaDB
    - Solo temas de VantaDB (no otros holones aún)
- `#🛠️soporte`:
    - Ayuda general (instalación, configuración, etc.)
- `#💡ideas-holones`:
    - RFCs, feature requests, discusiones de diseño
- `#🎉logros`:
    - Usuarios comparten lo que construyeron con VantaDB/SyntropyOS
- Canales técnicos (`#📚docs`, `#🧪tests`, `#🐛bugs`):
    - Para contribuidores activos
    - Discusiones específicas de código
- Canales internacionales (`#🇪🇸español`, `#🇧🇷português`):
    - Chat en idiomas específicos
    - Opcional, pero bueno para comunidad global
- Canales off-topic (`#☕café`, `#🎮gaming`, `#🎵música`):
    - Para comunidad, no solo código
    - Ayuda a retener usuarios

---

#### 2.4.2. Roles

> [MÁS ADELANTE, decisión 2026-09-08 — servidor intacto por ahora]

**Roles Sugeridos:**

```
@Founder       - @ness-e (tú)
@Admin         - Co-fundadores, si hay
@Mod           - Moderadores de confianza
@Contributor   - Gente que contribuyó código/docs/tests
@Early Adopter - Primeros 50 usuarios
@Member        - Todos los demás
@Bot           - Bots (MEE6, WelcomeBot, etc.)
```

**Permisos por Rol:**

- `@Founder`, `@Admin`:
    - Todos los permisos
- `@Mod`:
    - Gestionar canales, mensajes, roles
    - Ban/kick usuarios
    - No pueden borrar canales o cambiar configuración crítica
- `@Contributor`:
    - Acceso a canales técnicos
    - Mención en `@Contributor` para anuncios de contribuciones
- `@Early Adopter`:
    - Rol especial para primeros 50 usuarios
    - Mención en `@Early Adopter` para anuncios exclusivos
- `@Member`:
    - Permisos básicos (leer, escribir en canales públicos)
- `@Bot`:
    - Permisos específicos para cada bot

---

#### 2.4.3. Bots Recomendados

> [MÁS ADELANTE, decisión 2026-09-08 — servidor intacto por ahora. Estado MEE6/WelcomeBot 2026 no verificado]

**Bots Esenciales:**

- **WelcomeBot** o **MEE6**:
    - Bienvenidas automáticas en `#👋bienvenida`
    - Mensaje personalizado con enlaces a reglas, anuncios, general
- **Simple Poll**:
    - Para encuestas en `#💡ideas-holones`
- **GitHub Bot** (opcional):
    - Notificaciones de issues, PRs, releases en `#📢anuncios`
- **Role Bot** (opcional):
    - Usuarios pueden auto-asignarse roles (ej: `@Python`, `@TypeScript`, `@Español`)

**Configuración de WelcomeBot:**

```
Canal: #👋bienvenida
Mensaje:
¡Bienvenido/a {user} a SyntropyOS! 🎉

- Lee las #📜reglas
- Mira #📢anuncios para novedades
- Presentate en #💬general
- ¿Problemas con VantaDB? #🧠vantadb-memoria

¡Disfruta!
```

---

#### 2.4.4. Moderación

**Configuración de Moderación:**

- **Moderation Level**: Medium
- **2FA Requerida para Mods**: Sí (si tienes mods)
- **Filtro Multimedia**: Para todos (prevenir spam de imágenes)
- **Filtro de Links**: Para no-miembros (prevenir spam)
- **Slowmode**: En `#💬general` (1 mensaje cada 5 segundos) si hay mucho spam

**Comandos de Moderación (para mods):**

- `!kick @usuario razón` - Expulsar usuario
- `!ban @usuario razón` - Banear usuario
- `!timeout @usuario 10m razón` - Timeout por 10 minutos
- `!clear 10` - Borrar últimos 10 mensajes

---

#### 2.4.5. Invite Permanente

**Tareas:**

- Crea invite permanente (sin expiración, sin límite de usos)
- Actualiza invite en todos los READMEs:
    - `SyntropyOS/profile/README.md`
    - `SyntropyOS/profile/README_ES.md`
    - `ness-e/Vantadb/README.md`
    - `SyntropyOS/.github/CONTRIBUTING.md`
    - `SyntropyOS/.github/SUPPORT.md`
- Testea el invite (asegúrate de que funciona)

**Formato de Invite:**

```
https://discord.gg/g8nqB3NtXt  # Tu invite real
```

---

**Checklist para Discord:**

- Canales básicos creados (bienvenida, anuncios, reglas, general, vantadb, soporte, ideas)
- Roles configurados (Founder, Admin, Mod, Contributor, Early Adopter, Member, Bot)
- Bots esenciales (WelcomeBot, Simple Poll)
- Moderación configurada (level medium, filtros)
- Invite permanente creado y actualizado en todos los READMEs

---

#### 2.5. Badges Funcionando

**Tareas:**

- License badge en READMEs (formato verificado shields.io):

    ```
    [![License: Apache 2.0](https://img.shields.io/github/license/ness-e/Vantadb)](https://opensource.org/licenses/Apache-2.0)
    ```

- Discord badge (requiere Server ID + widget habilitado por admin):

    ```
    [![Discord](https://img.shields.io/discord/TU_SERVER_ID)](https://discord.gg/g8nqB3NtXt)
    ```

    > Pendiente: extraer Server ID (Widget → Enable) y sustituir `TU_SERVER_ID`.

- CI badge (con el nombre real del workflow existente):

    ```
    [![CI](https://img.shields.io/github/actions/workflow/status/ness-e/Vantadb/ci-rust-10.yml)](https://github.com/ness-e/Vantadb/actions)
    ```

- PyPI version badge:

    ```
    [![PyPI](https://img.shields.io/pypi/v/vantadb-py)](https://pypi.org/project/vantadb-py/)
    ```

- npm version badge:

    ```
    [![npm](https://img.shields.io/npm/v/vantadb)](https://www.npmjs.com/package/vantadb)
    ```
    

**Checklist para Badges:**

- License badge
- Discord badge
- CI badge (si hay CI)
- PyPI version badge
- npm version badge
- Todos los badges en READMEs principales

---

**Recomendación Final para SyntropyOS Org:**

La org debe parecer "viva" pero honesta sobre el estado temprano. No ocultes que es early stage, pero tampoco debe parecer abandonada. READMEs actualizados, docs canónicos completos, Discord funcional, badges trabajando.

---

*(Continuaré con Paquetes, Docs, y Fases 1-3 en el siguiente mensaje...)*

---

#### 3. **Paquetes (PyPI, npm)**

**Por qué importa:**

Si los paquetes no están actualizados o no funcionan, la gente no puede instalar VantaDB. Es la primera barrera de entrada.

---

#### 3.1. `vantadb-py` en PyPI

**Tareas Detalladas:**

#### 3.1.1. Actualizar `setup.py` o `pyproject.toml`

**Formato de `pyproject.toml`:**

```
[build-system]
requires = ["setuptools>=61.0", "wheel"]
build-backend = "setuptools.build_meta"

[project]
name = "vantadb-py"
version = "0.5.0"
description = "Memoria persistente gobernada para agentes de IA"
readme = "README.md"
requires-python = ">=3.11"
license = {text = "Apache-2.0"}
# NOTA (PEP 639): NO agregar classifier "License :: ..." — deprecado, da warning. Solo la expresión SPDX de arriba.
authors = [
    {name = "ness-e", email = "syntropyos.ia@gmail.com"}
]
keywords = ["ai", "memory", "vector", "database", "syntropyos"]
classifiers = [
    "Development Status :: 4 - Beta",
    "Intended Audience :: Developers",
    "License :: OSI Approved :: Apache Software License",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
]
dependencies = [
    # Tus dependencias aquí
]

[project.urls]
Homepage = "https://vantadb.vercel.app"
Documentation = "https://vantadb.vercel.app"
Repository = "https://github.com/ness-e/Vantadb"
Issues = "https://github.com/ness-e/Vantadb/issues"
Discord = "https://discord.gg/g8nqB3NtXt"
```

**Checklist para `pyproject.toml`:**

- `version = "0.5.0"` (claro que es beta)
- `Homepage` apunta a `vantadb.vercel.app` o `github.com/ness-e/Vantadb`
- `Documentation` apunta a `vantadb.vercel.app`
- `Repository` apunta a `github.com/ness-e/Vantadb`
- `Issues` apunta a `github.com/ness-e/Vantadb/issues`
- `Discord` apunta a tu invite
- `requires-python = ">=3.11"` (o tu versión mínima)
- `license = "Apache-2.0"`

---

#### 3.1.2. README en PyPI

**Tareas:**

- `README.md` en el repo de `vantadb-py` debe ser el que se muestra en PyPI
- Asegúrate de que incluye:
    - Estado claro ("v0.5.0 - Early Access")
    - Quickstart (código copiable)
    - Enlaces a docs, GitHub, Discord
    - Licencia (Apache 2.0)

**Formato de README para PyPI:**

```
# VantaDB (Python)

> **Estado:** v0.5.0 - Early Access

VantaDB es memoria persistente gobernada para agentes de IA.

**⚠️ Importante:** Esto es Early Access. Funciona para casos de uso básicos, pero algunas features avanzadas (entity resolution, conflict detection) están en desarrollo.

## Instalación

```bash
pip install vantadb-py
```

## Quickstart

```python
import vantadb

db = vantadb.VantaDB("./my_brain")
db.put(namespace="prefs", key="editor", payload="usa tabs")
result = db.search_memory("prefs", [], text_query="formateo")
print(result)
```

## Docs

- Documentación completa
- GitHub
- Discord

## Licencia

Apache 2.0. Uso personal y comercial permitido.
```

---

#### 3.1.3. Publicar en PyPI

**Comandos:**

```
# Instalar herramientas
pip install build twine

# Build del paquete
python -m build

# Subir a PyPI
twine upload dist/*
```

**Tareas:**

- Crea cuenta en pypi.org (si no tienes)
- Genera API token en PyPI
- Configura `~/.pypirc` con tu token:
    
    ```
    [pypi]
    username = __token__
    password = pypi-XXXXXXXXXXXXXXXXXXXXXXXX
    ```
    
- Sube el paquete (comandos arriba)
- Verifica en pypi.org/project/vantadb-py que:
    - Versión es 0.5.0
    - README se ve bien
    - Enlaces funcionan (Homepage, Docs, Repo, Issues, Discord)

---

#### 3.1.4. Verificar Instalación

**Tareas:**

- En un environment limpio, prueba:
    
    ```
    pip install vantadb-py
    python -c "import vantadb; print(vantadb.__version__)"
    ```
    
- Debería imprimir `0.5.0` sin errores
- Prueba el quickstart:
    
    ```
    import vantadb
    db = vantadb.VantaDB("./test_brain")
    db.put(namespace="test", key="foo", payload="bar")
    result = db.search_memory("test", [], text_query="foo")
    print(result)
    ```
    

**Checklist para `vantadb-py`:**

- `pyproject.toml` actualizado con versión, enlaces, licencia
- README en PyPI claro y completo
- Paquete subido a PyPI
- Instalación funciona en environment limpio
- Quickstart funciona sin errores

---

#### 3.2. `vantadb` en npm

**Tareas Detalladas:**

#### 3.2.1. Actualizar `package.json`

**Formato de `package.json`:**

```
{
  "name": "vantadb",
  "version": "0.5.0",
  "description": "Memoria persistente gobernada para agentes de IA",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "scripts": {
    "build": "tsc",
    "test": "vitest",
    "prepublishOnly": "npm run build"
  },
  "keywords": [
    "ai",
    "memory",
    "vector",
    "database",
    "syntropyos"
  ],
  "author": "ness-e <syntropyos.ia@gmail.com>",
  "license": "Apache-2.0",
  "repository": {
    "type": "git",
    "url": "https://github.com/ness-e/Vantadb.git"
  },
  "bugs": {
    "url": "https://github.com/ness-e/Vantadb/issues"
  },
  "homepage": "https://vantadb.vercel.app",
  "engines": {
    "node": ">=18"
  },
  "files": [
    "dist",
    "README.md",
    "LICENSE"
  ]
}
```

**Checklist para `package.json`:**

- `version = "0.5.0"` (claro que es beta)
- `homepage` apunta a `vantadb.vercel.app`
- `repository` apunta a `github.com/ness-e/Vantadb`
- `bugs` apunta a `github.com/ness-e/Vantadb/issues`
- `engines.node = ">=18"` (o tu versión mínima)
- `license = "Apache-2.0"`

---

#### 3.2.2. README en npm

**Tareas:**

- `README.md` en el repo de `vantadb` debe ser el que se muestra en npm
- Asegúrate de que incluye:
    - Estado claro ("v0.5.0 - Early Access")
    - Quickstart (código copiable)
    - Enlaces a docs, GitHub, Discord
    - Licencia (Apache 2.0)

**Formato de README para npm:**

```
# VantaDB (TypeScript)

> **Estado:** v0.5.0 - Early Access

VantaDB es memoria persistente gobernada para agentes de IA.

**⚠️ Importante:** Esto es Early Access. Funciona para casos de uso básicos, pero algunas features avanzadas (entity resolution, conflict detection) están en desarrollo.

## Instalación

```bash
npm install vantadb
```

## Quickstart

```typescript
import { VantaDB } from "vantadb";

const db = VantaDB.create();
db.put({ namespace: "prefs", key: "editor", payload: "usa tabs" });
const result = db.search({ namespace: "prefs", query_vector: [], text_query: "formateo" });
console.log(result);
```

## Docs

- Documentación completa
- GitHub
- Discord

## Licencia

Apache 2.0. Uso personal y comercial permitido.
```

---

#### 3.2.3. Publicar en npm

**Comandos:**

```
# Build del paquete
npm run build

# Login a npm (si no estás logueado)
npm login

# Publicar
npm publish
```

**Tareas:**

- Crea cuenta en npmjs.com (si no tienes)
- Login en terminal (`npm login`)
- Build del paquete (`npm run build`)
- Publica (`npm publish`)
- Verifica en npmjs.com/package/vantadb que:
    - Versión es 0.5.0
    - README se ve bien
    - Enlaces funcionan (Homepage, Repo, Issues)

---

#### 3.2.4. Verificar Instalación

**Tareas:**

- En un environment limpio, prueba:
    
    ```
    npm install vantadb
    node -e "import('vantadb').then(m => console.log(typeof m.VantaDB.create === 'function' ? 'VantaDB OK' : 'fail'))"
    ```
    
- Debería imprimir `0.5.0` sin errores
- Prueba el quickstart:
    
    ```
    import { VantaDB } from "vantadb";
    const db = VantaDB.create();
    db.put({ namespace: "test", key: "foo", payload: "bar" });
    const result = db.search({ namespace: "test", query_vector: [], text_query: "foo" });
    console.log(result);
    ```
    

**Checklist para `vantadb` npm:**

- `package.json` actualizado con versión, enlaces, licencia
- README en npm claro y completo
- Paquete publicado en npm
- Instalación funciona en environment limpio
- Quickstart funciona sin errores

---

**Recomendación Final para Paquetes:**

Si los paquetes no están actualizados, mejor no anuncies aún. Arregla esto primero. Un usuario que no puede instalar es un usuario perdido.

---

#### 4. **Docs de VantaDB (vantadb.vercel.app)**

**Por qué importa:**

Las docs son tu "producto" tanto como el código. Si son confusas, la gente asume que el código también lo es.

---

#### 4.1. Estructura de Docs

**Estructura Recomendada:**

```
vantadb.vercel.app/
├── index.md (Homepage)
├── installation.md
├── quickstart.md
├── api-reference.md
├── faq.md
├── roadmap.md
├── contributing.md
└── changelog.md
```

**Contenido de cada página:**

Ya detallé `index.md`, `installation.md`, `quickstart.md`, `api-reference.md`, `faq.md`, `roadmap.md` en la sección 1.2. Agrego aquí `contributing.md` y `changelog.md`:

#### `contributing.md`

```
# Contribuir

¡Gracias por querer contribuir! 🎉

## Cómo Contribuir

### 1. Reportar Bugs

Abre un issue en GitHub con el template "Bug Report".

### 2. Sugerir Features

Abre un issue en GitHub con el template "Feature Request".

### 3. Mejorar Docs

- Edita archivos `.md` en `docs/`
- O sugiere cambios en Discord

### 4. Escribir Tests

- Tests unitarios en `vantadb-python/tests/test_sdk.py` o `vantadb-ts/` (`npm test`)
- Tests de integración en `tests/*.rs` (ej: `tests/api/`, `cargo test --test …`)

### 5. Traducciones

- Traduce docs a tu idioma
- Abre un PR con la traducción

## Correr Tests

### Python

```bash
pip install maturin
maturin develop --release  # dentro de vantadb-python/
pytest vantadb-python/tests/test_sdk.py
```

### TypeScript

```bash
npm install
npm test
```

## Preguntas?

Únete a Discord o abre un issue.
```

#### `changelog.md`

```
# Changelog

## v0.5.0 (2026-08-01)

### Features

- ✅ Persistencia durable (WAL + fsync + CRC32C)
- ✅ Búsqueda híbrida (BM25 + HNSW + RRF)
- ✅ Filtrado por metadatos y namespaces
- ✅ TTL y expiración automática
- ✅ Grafos de entidades (BFS, DFS, PageRank)
- ✅ Supersession (marcar registros como obsoletos)
- ✅ Bindings Python y TypeScript

### Known Issues

- ❌ Entity resolution no implementada
- ❌ Conflict detection no implementado

Ver releases en GitHub
```

**Checklist para Docs:**

- Homepage clara con estado del proyecto
- Installation (Python y TS)
- Quickstart (código copiable)
- API reference (aunque sea mínima)
- FAQ (al menos 5 preguntas)
- Roadmap (v0.7.0, v1.0.0)
- Contributing (cómo contribuir)
- Changelog (v0.5.0 actual)
- Enlaces a GitHub, Discord, SyntropyOS

---

#### 4.2. Deploy en Vercel

**Tareas:**

- Crea repo en GitHub para docs (ej: `ness-e/vantadb-docs`)
- Sube archivos `.md` al repo
- Conecta repo a Vercel:
    - Ve a vercel.com
    - Importa el repo
    - Configura build (para VitePress, Docusaurus, etc.)
- Deploy automático en cada push a `main`
- Custom domain `vantadb.vercel.app` (o `docs.vantadb.io` si tienes dominio)

**Checklist para Deploy:**

- Repo de docs en GitHub
- Archivos `.md` subidos
- Conectado a Vercel
- Deploy automático configurado
- Custom domain configurado (opcional)

---

**Recomendación Final para Docs:**

Mejor docs mínimas pero claras que docs extensas pero confusas. Si no tienes tiempo para docs completas, al menos ten Quickstart, Installation, y FAQ claros.

---

*(Continuaré con Fases 1-3 en el siguiente mensaje...)*

---

## Fase 1: Lanzamiento Suave (Semanas 1-4)

### Objetivo:

Atraer **5-10 usuarios tempranos**, no 1000. Calidad sobre cantidad.

---

### 📅 Semana 1: Anuncio en GitHub + Discord

#### Día 1-2: Preparar Anuncios

**Tareas:**

#### 1.1. Publicar en GitHub Discussions de `SyntropyOS`

**URL:** `https://github.com/SyntropyOS/.github/discussions`

**Título:**

```
SyntropyOS + VantaDB v0.5.0 - Early Access
```

**Contenido (copia y adapta):**

```
# SyntropyOS + VantaDB v0.5.0 - Early Access

¡Hola comunidad! 👋

## ¿Qué es SyntropyOS?

SyntropyOS es un sistema operativo para agentes de IA. Construido con **holones** modulares:

- **VantaDB** (memoria) - ✅ v0.5.0 (Early Access)
- **Iris** (visión) - 🔵 Planificado
- **Cardinal** (orientación) - 🔵 Planificado
- **Sage** (aprendizaje) - 🔵 Planificado
- **Execute** (ejecución) - 🔵 Planificado
- **Plan** (planificación) - 🔵 Planificado

Ver Manifiesto | Ver Roadmap

---

## ¿Qué es VantaDB?

VantaDB es el **holón de memoria** de SyntropyOS. Memoria persistente gobernada para agentes de IA.

**Estado:** v0.5.0 - Early Access

**Qué funciona:**
- ✅ Persistencia durable (WAL + fsync + CRC32C)
- ✅ Búsqueda híbrida (BM25 + HNSW + RRF)
- ✅ Filtrado por metadatos y namespaces
- ✅ TTL y expiración automática
- ✅ Grafos de entidades (BFS, DFS, PageRank)
- ✅ Supersession (marcar registros como obsoletos)
- ✅ Bindings Python (`pip install vantadb-py`) y TypeScript (`npm install vantadb`)

**Qué NO funciona aún:**
- ❌ Entity resolution automática (detectar duplicados como "Acme Corp" = "Acme Corporation")
- ❌ Conflict detection (detectar contradicciones automáticamente)
- ❌ Skills extract (memoria procedimental)
- ❌ Consolidación automática L2→L3
- ❌ Auditoría automática de vigencia

Ver VantaDB | Ver Docs

---

## Roadmap

- **v0.5.0** (actual): Early Access - persistencia + búsqueda híbrida
- **v0.7.0** (TBD): Governance - entity resolution, conflict detection
- **v1.0.0** (TBD): Production-ready - todas las features de governance

---

## ¿Quieres contribuir?

Buscamos ayuda con:
- 📖 Docs (tutorials, ejemplos, FAQs)
- 🧪 Tests (unitarios, de integración)
- 🐛 Reportar bugs
- 💡 Feedback de uso real
- 🌍 Traducciones (ES, PT, etc.)

Ver CONTRIBUTING.md

---

## Únete a la Comunidad

- 💬 Discord - Chat, soporte, anuncios
- 📚 Docs - Documentación de VantaDB
- 🐛 Issues - Reportar bugs, sugerir features

---

**SyntropyOS** — Orden desde el caos.
```

**Tareas:**

- Publica en `SyntropyOS/.github/discussions` ([PAUSADO hasta migrar VantaDB a Syntropy, decisión 2026-09-08] — además hoy da 404, `has_discussions=false`; al reactivar: habilitar en Settings → Features o usar org-discussions)
- Fija el post (pin) para que sea lo primero que se ve
- Activa notificaciones para el post (para ver comentarios)

---

#### 1.2. Publicar en GitHub Discussions de `ness-e/Vantadb`

**URL:** `https://github.com/ness-e/Vantadb/discussions`

**Título:**

```
VantaDB v0.5.0 - Early Access (Lanzamiento)
```

**Contenido:**

Mismo contenido que el post de SyntropyOS, pero enfocado en VantaDB:

```
# VantaDB v0.5.0 - Early Access (Lanzamiento)

¡Hola! 👋

VantaDB v0.5.0 está disponible como **Early Access**.

## ¿Qué es VantaDB?

VantaDB es memoria persistente gobernada para agentes de IA. Es el holón de memoria de SyntropyOS.

**Estado:** v0.5.0 - Early Access

**Qué funciona:**
- ✅ Persistencia durable (WAL + fsync + CRC32C)
- ✅ Búsqueda híbrida (BM25 + HNSW + RRF)
- ✅ Filtrado por metadatos y namespaces
- ✅ TTL y expiración automática
- ✅ Grafos de entidades (BFS, DFS, PageRank)
- ✅ Supersession (marcar registros como obsoletos)
- ✅ Bindings Python (`pip install vantadb-py`) y TypeScript (`npm install vantadb`)

**Qué NO funciona aún:**
- ❌ Entity resolution automática
- ❌ Conflict detection
- ❌ Skills extract
- ❌ Consolidación automática L2→L3
- ❌ Auditoría automática de vigencia

Ver Docs | Ver Roadmap

---

## Quickstart

### Python

```bash
pip install vantadb-py
```

```python
import vantadb

db = vantadb.VantaDB("./my_brain")
db.put(namespace="prefs", key="editor", payload="usa tabs")
result = db.search_memory("prefs", [], text_query="formateo")
print(result)
```

### TypeScript

```bash
npm install vantadb
```

```typescript
import { VantaDB } from "vantadb";

const db = VantaDB.create();
db.put({ namespace: "prefs", key: "editor", payload: "usa tabs" });
const result = db.search({ namespace: "prefs", query_vector: [], text_query: "formateo" });
console.log(result);
```

---

## Roadmap

- **v0.5.0** (actual): Early Access
- **v0.7.0** (TBD): Governance
- **v1.0.0** (TBD): Production-ready

---

## ¿Quieres contribuir?

Buscamos ayuda con:
- 📖 Docs
- 🧪 Tests
- 🐛 Bugs
- 💡 Feedback

Ver CONTRIBUTING.md | Únete a Discord

---

**SyntropyOS** — Orden desde el caos.
```

**Tareas:**

- Publica en `ness-e/Vantadb/discussions`
- Fija el post (pin)
- Activa notificaciones

---

#### 1.3. Actualizar README de `SyntropyOS/.github`

**Tareas:**

- Agrega sección "Early Access - Únete a la comunidad" en `README.md` de `.github`:

```
## 📢 Early Access - Únete a la comunidad

VantaDB v0.5.0 está en **Early Access**. Buscamos 5-10 usuarios tempranos para feedback.

- Ver VantaDB
- Ver Docs
- Únete a Discord
```

- Agrega mismo contenido en `README_ES.md`

---

#### 1.4. Tweet Simple (Opcional, si tienes Twitter)

**Formato de Tweet:**

```
VantaDB v0.5.0 está en Early Access. Es el holón de memoria de SyntropyOS.

Busco 5-10 usuarios tempranos para feedback.

- Persistencia durable
- Búsqueda híbrida (BM25 + HNSW + RRF)
- TTL, grafos, supersession

Docs: https://vantadb.vercel.app
GitHub: https://github.com/ness-e/Vantadb
Discord: https://discord.gg/g8nqB3NtXt

#AI #OpenSource
```

**Tareas:**

- Publica tweet (si tienes Twitter)
- No hagas hype, solo informativo
- Responde a cualquier comentario en < 24 horas

---

#### Día 3-7: Primeros Usuarios

**Tareas:**

#### 1.5. Responder Issues y Mensajes

**Tareas:**

- Revisa GitHub Issues de `ness-e/Vantadb` cada día
- Responde TODOS los issues en < 48 horas
- Revisa Discord cada día
- Responde TODOS los mensajes en < 24 horas

**Formato de Respuesta a Issue:**

```
¡Gracias por reportar esto! 🙌

Investigaré este bug hoy/tomorrow. Te mantengo informado.

Mientras tanto, ¿podrías confirmar:
- ¿Qué OS estás usando?
- ¿Qué versión de Python/Node?
- ¿Puedes compartir el código que causa el error?

¡Gracias!
```

**Formato de Respuesta en Discord:**

```
¡Bienvenido! 🎉

¿En qué puedo ayudarte? ¿Problemas con VantaDB o solo curiosidad?

Si es un bug, abre un issue en GitHub: https://github.com/ness-e/Vantadb/issues

Si es una pregunta, pregunta aquí mismo. ¡Estoy para ayudar!
```

---

#### 1.6. Trackear Primeros Usuarios

**Tareas:**

- Crea un doc interno (`docs/early-adopters.md`):

```
# Early Adopters (v0.5.0)

## Usuarios Tempranos

| Usuario | GitHub/Discord | Fecha | Caso de Uso | Feedback |
|---------|----------------|-------|-------------|----------|
| @user1  | @user1         | 2026-09-XX | Agente personal | "Fácil de instalar, docs claras" |
| @user2  | @user2         | 2026-09-XX | RAG para docs   | "Búsqueda híbrida funciona bien" |

## Contribuciones

- @user1: PR #42 (fix en docs)
- @user2: Issue #43 (bug report)

## Agradecimientos

¡Gracias a todos los early adopters! Sin ustedes, VantaDB no sería posible.
```

- Actualiza este doc cada semana
- Cuando tengas 10 usuarios, dales rol `@Early Adopter` en Discord

---

**Checklist Semana 1:**

- Post en GitHub Discussions de SyntropyOS
- Post en GitHub Discussions de VantaDB
- README de `.github` actualizado
- Tweet (opcional)
- Todos los issues respondidos en < 48 horas
- Todos los mensajes de Discord respondidos en < 24 horas
- Doc de early adopters creado

---

### 📅 Semana 2: Primeros Usuarios (Continuación)

#### Día 8-14: Soporte Activo

**Tareas:**

#### 2.1. Responder Issues y Mensajes (Continuación)

**Tareas:**

- Revisa GitHub Issues cada día
- Responde TODOS los issues en < 48 horas
- Revisa Discord cada día
- Responde TODOS los mensajes en < 24 horas

**Tácticas:**

- **Sé rápido:** Responde en < 24 horas. Usuarios tempranos valoran atención rápida.
- **Sé amable:** Aunque el issue sea obvio, sé paciente. "Gracias por reportar esto" va lejos.
- **Sé honesto:** Si no sabes la respuesta, di "No sé, pero investigaré". No inventes.

---

#### 2.2. Fixear Bugs Críticos

**Tareas:**

- Si alguien reporta un bug crítico (ej: VantaDB no instala, crash al hacer `put()`):
    - Prioriza fixearlo en < 48 horas
    - Publica fix en GitHub
    - Notifica al usuario que reportó
    - Actualiza CHANGELOG.md

**Formato de Respuesta a Bug Fix:**

```
¡Bug fixado! 🎉

El issue estaba en [archivo/línea]. Lo fixeé en PR #XX.

Puedes actualizar a la última versión:

```bash
pip install --upgrade vantadb-py
# o
npm install vantadb@latest
```

¡Gracias por reportar esto! Ayudaste a mejorar VantaDB.
```

---

#### 2.3. Documentar Preguntas Comunes

**Tareas:**

- Si alguien pregunta lo mismo 2+ veces, agrégalo a FAQ en docs:

```
## FAQ

### P: ¿VantaDB funciona en Windows?

R: Sí, funciona en Windows, macOS, Linux. Python 3.11+ o Node 18+ requeridos.

### P: ¿Cómo uso TTL?

R: pasa `ttl_ms=90*24*3600*1000` como parámetro top-level del `put()` (NO dentro de `metadata`). Ver Quickstart.

### P: ¿Entity resolution ya funciona?

R: No aún. Está planeada para v0.7.0 (TBD). Mientras tanto, puedes hacer entity resolution manual con grafos.
```

- Actualiza `docs/faq.md` cada semana

---

**Checklist Semana 2:**

- Todos los issues respondidos en < 48 horas
- Todos los mensajes de Discord respondidos en < 24 horas
- Bugs críticos fixeados en < 48 horas
- FAQ actualizada con preguntas comunes

---

### 📅 Semana 3: Primer Feedback

#### Día 15-21: Pedir Feedback Explícito

**Tareas:**

#### 3.1. Pedir Feedback a Usuarios Tempranos

**Tareas:**

- En Discord, crea post en `#💬general`:

```
# 🔍 Feedback Request

¡Hola early adopters! 👋

Quiero mejorar VantaDB con su feedback. ¿Podrían responder estas preguntas?

1. ¿Para qué estás usando VantaDB?
2. ¿Qué te parece VantaDB hasta ahora? (1-10)
3. ¿Qué feature te gustaría ver en v0.7.0?
4. ¿Hay algo que no te queda claro en las docs?
5. ¿Recomendarías VantaDB a un colega? ¿Por qué sí/no?

Pueden responder aquí o en DM. ¡Gracias! 🙌
```

- Si tienes emails de usuarios (ej: de issues), envía email:

**Formato de Email:**

```
Asunto: Feedback de VantaDB v0.5.0

Hola [Nombre],

Gracias por usar VantaDB v0.5.0. Estoy construyendo SyntropyOS y tu feedback es invaluable.

¿Podrías responder estas 5 preguntas?

1. ¿Para qué estás usando VantaDB?
2. ¿Qué te parece VantaDB hasta ahora? (1-10)
3. ¿Qué feature te gustaría ver en v0.7.0?
4. ¿Hay algo que no te queda claro en las docs?
5. ¿Recomendarías VantaDB a un colega? ¿Por qué sí/no?

Puedes responder a este email o en Discord: https://discord.gg/g8nqB3NtXt

¡Gracias!

[Tu Nombre]
Fundador, SyntropyOS
```

---

#### 3.2. Crear Encuesta Simple

**Tareas:**

- En Discord, usa Simple Poll o encuesta nativa:

**Formato de Encuesta:**

```
¿Para qué estás usando VantaDB?

- [ ] Agente personal
- [ ] RAG para docs
- [ ] Grafos de conocimiento
- [ ] Experimentación
- [ ] Otro (comenta abajo)
```

```
¿Qué tan fácil fue instalar VantaDB?

- [ ] Muy fácil (5 min)
- [ ] Fácil (15 min)
- [ ] Medio (30 min)
- [ ] Difícil (1+ hora)
- [ ] No pude instalar
```

```
¿Qué holón te interesa más después de memoria?

- [ ] Iris (visión)
- [ ] Cardinal (orientación)
- [ ] Sage (aprendizaje)
- [ ] Execute (ejecución)
- [ ] Plan (planificación)
```

- Documenta resultados en `docs/feedback/semana-3.md`

---

#### 3.3. Documentar Feedback

**Tareas:**

- Crea doc interno (`docs/feedback/semana-3.md`):

```
# Feedback Semana 3

## Respuestas

### Usuario 1 (@user1)

1. **Caso de uso:** Agente personal
2. **Rating:** 8/10
3. **Feature deseada:** Entity resolution
4. **Docs confusas:** TTL (no entendí unidades)
5. **Recomendaría:** Sí, pero esperaría a v0.7.0

### Usuario 2 (@user2)

1. **Caso de uso:** RAG para docs
2. **Rating:** 9/10
3. **Feature deseada:** Conflict detection
4. **Docs confusas:** Nada, claras
5. **Recomendaría:** Sí, ya lo recomendé

## Insights

- 2/2 usuarios quieren entity resolution
- 1/2 tuvo confusión con TTL (unidades)
- Rating promedio: 8.5/10

## Acciones

- [ ] Aclarar unidades de TTL en docs
- [ ] Priorizar entity resolution en v0.7.0
- [ ] Agregar ejemplo de RAG en docs
```

- Actualiza este doc cada semana

---

**Checklist Semana 3:**

- Post de feedback en Discord
- Emails enviados a usuarios (si tienes emails)
- Encuestas creadas en Discord
- Feedback documentado en `docs/feedback/semana-3.md`
- Insights y acciones derivadas del feedback

---

### 📅 Semana 4: Primer Update Público

#### Día 22-28: Publicar Update

**Tareas:**

#### 4.1. Publicar Update en GitHub Discussions

**URL:** `https://github.com/ness-e/Vantadb/discussions`

**Título:**

```
VantaDB v0.5.0 - Update Semana 4
```

**Contenido:**

```
# VantaDB v0.5.0 - Update Semana 4

¡Hola comunidad! 👋

Gracias a los primeros usuarios por el feedback. Aquí el update de la semana 4:

## Qué se Mejoró

- ✅ Aclarado TTL en docs (unidades en ms)
- ✅ Agregado ejemplo de RAG en Quickstart
- ✅ Fixeado bug #42 (crash en Windows)
- ✅ Mejorado error messages en Python binding

## Qué Viene en v0.6.0

- 🔵 Entity resolution manual (API para marcar duplicados)
- 🔵 Mejoras en búsqueda híbrida (RRF tuning)
- 🔵 Más ejemplos en docs

## Stats

- 📦 Downloads PyPI: XX
- 📦 Downloads npm: XX
- ⭐ GitHub Stars: XX
- 💬 Discord Members: XX

## Agradecimientos

Gracias a @user1, @user2 por reportar bugs y sugerir mejoras. ¡Son increíbles!

---

**SyntropyOS** — Orden desde el caos.
```

**Tareas:**

- Publica en `ness-e/Vantadb/discussions`
- Fija el post (pin)
- Comparte en Discord (`#📢anuncios`)

---

#### 4.2. Actualizar CHANGELOG.md

**Tareas:**

- Actualiza `CHANGELOG.md` en repo de VantaDB:

```
# Changelog

## v0.5.1 (2026-09-XX)

### Fixes

- 🐛 Fixeado bug #42 (crash en Windows)
- 🐛 Mejorado error messages en Python binding

### Docs

- 📖 Aclarado TTL en docs (unidades en ms)
- 📖 Agregado ejemplo de RAG en Quickstart

## v0.5.0 (2026-08-01)

### Features

- ✅ Persistencia durable (WAL + fsync + CRC32C)
- ✅ Búsqueda híbrida (BM25 + HNSW + RRF)
- ✅ Filtrado por metadatos y namespaces
- ✅ TTL y expiración automática
- ✅ Grafos de entidades (BFS, DFS, PageRank)
- ✅ Supersession (marcar registros como obsoletos)
- ✅ Bindings Python y TypeScript
```

- Crea release en GitHub (`v0.5.1`):
    - Ve a `Releases` → `Draft a new release`
    - Tag: `v0.5.1`
    - Título: `v0.5.1 - Bug Fixes`
    - Descripción: Copia de CHANGELOG.md
    - Publica

---

#### 4.3. Tweet Simple (Opcional)

**Formato de Tweet:**

```
VantaDB v0.5.1:

- Fix bug en Windows
- Mejores error messages
- Docs más claras (TTL, RAG)

Gracias a los primeros usuarios por el feedback. 🙌

Changelog: https://github.com/ness-e/Vantadb/blob/main/CHANGELOG.md
Discord: https://discord.gg/g8nqB3NtXt

#AI #OpenSource
```

**Tareas:**

- Publica tweet (si tienes Twitter)
- Responde a comentarios en < 24 horas

---

**Checklist Semana 4:**

- Update publicado en GitHub Discussions
- CHANGELOG.md actualizado
- Release v0.5.1 creado en GitHub
- Tweet (opcional)
- Update compartido en Discord

---

**Checklist Fase 1 (Semanas 1-4):**

- Posts en GitHub Discussions (SyntropyOS + VantaDB)
- READMEs actualizados
- Todos los issues respondidos en < 48 horas
- Todos los mensajes de Discord respondidos en < 24 horas
- Bugs críticos fixeados en < 48 horas
- FAQ actualizada con preguntas comunes
- Feedback pedido y documentado
- Encuestas creadas
- Update semanal publicado
- CHANGELOG.md actualizado
- Release v0.5.1 creado
- 5-10 usuarios tempranos activos

---

*(Continuaré con Fases 2-3 en el siguiente mensaje...)*

---

## Fase 2: Crecimiento Orgánico (Semanas 5-12)

### Objetivo:

Llegar a **50-100 usuarios**, comunidad activa en Discord.

> Gate previo (no negociable): no entrar a Fase 2 sin 5 installs limpias documentadas y 0 críticos abiertos. Las métricas "50-100" son aspiracionales; el avance se mide con la tabla de la corrección §4 (valor, no vanidad).

---

### 📅 Seguimiento Semanal (Cada Semana, Semanas 5-12)

#### 📋 Rutina Semanal Fija

**Tareas Fijas (cada semana):**

---

#### 📍 Lunes: Revisión de Issues + PRs + Discord

**Checklist de Lunes:**

- **Revisar Issues de GitHub** (`ness-e/Vantadb/issues`):
    - Cierra issues que estén resueltos
    - Responde a issues nuevos (en < 48 horas)
    - Etiqueta issues (bug, enhancement, question, etc.)
    - Asigna issues a milestones (v0.6.0, v0.7.0, etc.)
- **Revisar PRs Abiertos** (`ness-e/Vantadb/pulls`):
    - Mergea PRs que estén listos
    - Pide cambios si falta algo
    - Agradece a contribuidores
- **Revisar Discord**:
    - Responde mensajes en `#💬general`, `#🧠vantadb-memoria`, `#🛠️soporte`
    - Anuncia si hay algo nuevo (release, doc update, etc.) en `#📢anuncios`
    - Revisa si hay bugs reportados en Discord (si sí, crea issue en GitHub)

**Formato de Respuesta a Issue Nuevo:**

```
¡Gracias por reportar esto! 🙌

Investigaré este bug hoy/tomorrow. Te mantengo informado.

Mientras tanto, ¿podrías confirmar:
- ¿Qué OS estás usando?
- ¿Qué versión de Python/Node?
- ¿Puedes compartir el código que causa el error?

¡Gracias!
```

**Formato de Respuesta a PR:**

```
¡Gracias por este PR! 🎉

Revisaré los cambios hoy/tomorrow. Si todo está bien, lo mergeo.

¿Hay algo específico que quieras que revise con atención?

¡Gracias por contribuir!
```

---

#### 📍 Miércoles: Trabajar en VantaDB

**Checklist de Miércoles:**

- **Trabajar en Nuevas Features**:
    - Implementa 1-2 features pedidas por usuarios
    - O fixea bugs reportados
    - O mejora docs basado en feedback
- **Revisar Contribuciones de Externos**:
    - Si hay PRs de contribuidores externos, revísalos
    - Da feedback claro y constructivo
    - Mergea cuando esté listo
- **Tests**:
    - Escribe tests para nuevas features
    - Asegúrate de que CI pasa

**Tácticas:**

- **Enfócate en 1 cosa:** No intentes hacer 10 cosas a la vez. Elige 1 feature o bug y termínalo.
- **Documenta mientras codificas:** Si agregas una feature, actualiza docs al mismo tiempo.
- **Tests primero (si puedes):** Escribe tests antes o durante, no después.

---

#### 📍 Viernes: Anuncio Semanal

**Checklist de Viernes:**

- **Publicar Update en Discord** (`#📢anuncios`):

**Formato de Anuncio:**

```
# 📅 Esta Semana en SyntropyOS/VantaDB

¡Hola comunidad! 👋

Resumen de la semana:

## Qué se Hizo

- ✅ [Feature/fix 1]
- ✅ [Feature/fix 2]
- ✅ [Doc update]

## Qué Viene la Próxima Semana

- 🔵 [Feature/fix planeado 1]
- 🔵 [Feature/fix planeado 2]

## Stats

- 📦 Downloads PyPI: XX
- 📦 Downloads npm: XX
- ⭐ GitHub Stars: XX
- 💬 Discord Members: XX

¡Gracias por ser parte de esto! 🙌

---

**SyntropyOS** — Orden desde el caos.
```

**Tareas:**

- Publica en `#📢anuncios` de Discord
- Responde a comentarios en < 24 horas

---

- **Tweet Simple** (si tienes Twitter):

**Formato de Tweet:**

```
Esta semana en VantaDB:

- [Feature/fix 1]
- [Feature/fix 2]
- [Doc update]

Próxima semana: [feature planeada]

Stats: XX downloads, XX stars, XX Discord members

GitHub: https://github.com/ness-e/Vantadb
Discord: https://discord.gg/g8nqB3NtXt

#AI #OpenSource
```

**Tareas:**

- Publica tweet (si tienes Twitter)
- Responde a comentarios en < 24 horas

---

#### 📍 Domingo (Opcional): Weekly Interno

**Checklist de Domingo (Opcional):**

- **Escribir Weekly Interno** (`docs/weekly/2026-WXX.md`):

**Formato de Weekly:**

```
# Weekly 2026-WXX (Sep XX - Sep XX)

## Qué se Hizo Esta Semana

### Código

- [Feature/fix 1]
- [Feature/fix 2]

### Docs

- [Doc update 1]
- [Doc update 2]

### Comunidad

- XX issues respondidos
- XX PRs mergeados
- XX nuevos Discord members

## Qué No se Hizo (y Por Qué)

- [Tarea planeada que no se hizo] - [Razón: tiempo, prioridad, etc.]

## Qué Viene la Próxima Semana

- [Feature/fix planeado 1]
- [Feature/fix planeado 2]

## Issues/PRs Abiertos

- Issue #XX: [descripción breve]
- PR #XX: [descripción breve]

## Bugs Críticos

- [Si hay bugs críticos, listar aquí]

## Notas Personales

- [Cualquier nota personal, bloqueo, idea, etc.]
```

**Tareas:**

- Escribe weekly (aunque sea breve)
- Guarda en `docs/weekly/2026-WXX.md`

**Por qué importa:**

Los weeklies internos te ayudan a trackear progreso, identificar bloqueos, y mantener foco. Aunque sea opcional, es muy útil.

---

**Checklist Semanal Fijo (Semanas 5-12):**

- Lunes: Issues + PRs + Discord revisados
- Miércoles: Trabajo en VantaDB (features, fixes, docs)
- Viernes: Anuncio en Discord + Tweet
- Domingo (opcional): Weekly interno

---

### 📅 Hitos por Alcanzar (Semanas 5-12)

---

#### 🎯 Hito 1: v0.6.0 (Semana 6-8)

**Objetivo:**

Publicar v0.6.0 con 1-2 features pedidas por usuarios + fixes de bugs.

---

#### Tareas para v0.6.0

**1. Implementar Features Pedidas:**

- Revisa feedback de Semana 3 (`docs/feedback/semana-3.md`)
- Elige 1-2 features más pedidas:
    - Ej: Entity resolution manual, RRF tuning, más ejemplos
- Implementa esas features
- Escribe tests para las features
- Actualiza docs con las nuevas features

**2. Fixear Bugs Reportados:**

- Revisa issues abiertos en GitHub
- Fixea bugs críticos o comunes
- Actualiza CHANGELOG.md con los fixes

**3. Mejorar Docs:**

- Agrega ejemplos de las nuevas features
- Aclara secciones confusas (basado en feedback)
- Actualiza Quickstart si hay cambios

**4. Publicar v0.6.0:**

- Actualiza `version` en `pyproject.toml` y `package.json` a `0.6.0`
- Sube a PyPI:
    
    ```
    python -m build
    twine upload dist/*
    ```
    
- Publica en npm:
    
    ```
    npm run build
    npm publish
    ```
    
- Crea release en GitHub (`v0.6.0`):
    - Ve a `Releases` → `Draft a new release`
    - Tag: `v0.6.0`
    - Título: `v0.6.0 - [Nombre de la feature principal]`
    - Descripción: Lista de cambios (copy de CHANGELOG.md)
    - Publica

**5. Anunciar v0.6.0:**

- Publica en GitHub Discussions:

**Formato de Post:**

```
# VantaDB v0.6.0 - [Nombre de la Feature Principal]

¡Hola comunidad! 👋

VantaDB v0.6.0 está disponible.

## Qué hay de Nuevo

### Features

- ✅ [Feature 1] - [descripción breve]
- ✅ [Feature 2] - [descripción breve]

### Fixes

- 🐛 [Bug fix 1]
- 🐛 [Bug fix 2]

### Docs

- 📖 [Doc update 1]
- 📖 [Doc update 2]

## Upgrade

### Python

```bash
pip install --upgrade vantadb-py
```

### TypeScript

```bash
npm install vantadb@latest
```

## Qué Viene en v0.7.0

- 🔵 Entity resolution automática
- 🔵 Conflict detection
- 🔵 Skills extract

Ver Roadmap

---

**SyntropyOS** — Orden desde el caos.
```

- Publica en Discord (`#📢anuncios`)
- Tweet (si tienes Twitter)

---

**Checklist para v0.6.0:**

- 1-2 features implementadas (basadas en feedback)
- Bugs críticos fixeados
- Docs actualizadas
- Tests pasando
- Publicado en PyPI y npm
- Release en GitHub
- Anuncio en GitHub Discussions + Discord + Twitter

---

#### 🎯 Hito 2: Primeros Contribuidores Externos (Semana 8-10)

**Señales de que estás listo:**

- Alguien abre un PR sin que se lo pidas
- Alguien responde issues de otros en Discord
- Alguien escribe docs o tutorials no oficiales

---

#### Tareas para Contribuidores Externos

**1. Agradecer Públicamente:**

- En Discord, anuncia en `#📢anuncios`:

**Formato de Anuncio:**

```
# 🎉 Primeros Contribuidores Externos

¡Hola comunidad! 👋

Quiero agradecer a @user1 y @user2 por sus primeras contribuciones:

- @user1: PR #XX (fix en docs)
- @user2: Issue #XX (bug report detallado)

¡Son increíbles! Sin ustedes, VantaDB no sería lo mismo. 🙌

¿Quieres contribuir? Ver CONTRIBUTING.md

---

**SyntropyOS** — Orden desde el caos.
```

- En GitHub, comenta en el PR:

**Formato de Comentario:**

```
¡Muchas gracias por este PR! 🎉

Es increíble ver contribuciones externas. Mergeo esto con gusto.

¿Hay algo más en lo que te gustaría trabajar? Podemos discutir ideas en Discord o en issues.

¡Gracias de nuevo!
```

---

**2. Agregar Sección "Contributors" en README:**

- Actualiza README de `ness-e/Vantadb`:

```
## Contribuidores

¡Gracias a todos los que hacen VantaDB posible! 🎉

- @ness-e - Fundador
- @user1 - Docs
- @user2 - Bug reports

Ver lista completa
```

- Actualiza `CONTRIBUTORS.md`:

```
# Contribuidores de VantaDB

¡Gracias a todos los que hacen VantaDB posible! 🎉

## Core Team

- @ness-e - Fundador

## Contribuidores

- @user1 - Docs (PR #XX)
- @user2 - Bug reports (Issue #XX)

## Cómo Ser Listado Aquí

1. Contribuye con código, docs, tests, traducciones, o feedback valioso
2. Abre un PR o issue
3. Te agregaremos a esta lista

Ver CONTRIBUTING.md
```

---

**3. Canal `#contribuidores` en Discord (Opcional):**

- Crea canal `#🛠️contribuidores`:
    - Solo para contribuidores activos
    - Discusiones de código, PRs, arquitectura
    - Opcional, pero ayuda a retener contribuidores

---

**4. Roles de Mod en Discord (Opcional):**

- Si hay contribuidores muy activos, considera darles rol `@Mod`:
    - Pueden moderar canales
    - Responder issues en tu lugar (si estás ocupado)
    - Ayudar con onboarding de nuevos usuarios

---

**Checklist para Contribuidores Externos:**

- Agradecimiento público en Discord + GitHub
- Sección "Contributors" en README
- `CONTRIBUTORS.md` actualizado
- Canal `#contribuidores` (opcional)
- Roles de mod para contribuidores activos (opcional)

---

#### 🎯 Hito 3: v0.7.0 (Semana 10-12)

**Objetivo:**

Publicar v0.7.0 con governance básica (entity resolution, conflict detection si es posible).

---

#### Tareas para v0.7.0

**1. Implementar Governance Básica:**

> ⚠️ ESPECIFICACIÓN PROPUESTA — `mark_duplicate`, `detect_conflicts`, `extract_skills` y demás métodos de esta lista NO existen en v0.5.0 (verificado: 0 hits en bindings). Diseñar la firma contra el core antes de documentarla; no publicar ejemplos hasta que corran en entorno limpio.

- **Entity Resolution**:
    - API para marcar duplicados manualmente (si automática no es posible aún)
    - Ej: `db.mark_duplicate(namespace, key1, key2)`
    - Docs de cómo usar
- **Conflict Detection**:
    - API para detectar contradicciones (si es posible)
    - Ej: `db.detect_conflicts(namespace)` devuelve lista de conflictos
    - Docs de cómo usar
- **Skills Extract** (si es posible):
    - API para extraer memoria procedimental
    - Ej: `db.extract_skills(namespace)` devuelve lista de skills
    - Docs de cómo usar

**2. Docs Completas de Governance:**

- Agrega página `governance.md` en docs:

```
# Governance

VantaDB v0.7.0 introduce governance del ciclo de vida de la memoria.

## Entity Resolution

Entity resolution detecta duplicados (ej: "Acme Corp" = "Acme Corporation").

### Manual (v0.7.0)

```python
db.mark_duplicate(namespace="empresas", key1="acme-corp", key2="acme-corporation")
```

### Automática (v1.0.0)

En v1.0.0, entity resolution será automática.

## Conflict Detection

Conflict detection detecta contradicciones en la memoria.

```python
conflicts = db.detect_conflicts(namespace="prefs")
for conflict in conflicts:
    print(conflict)
```

## Skills Extract

Skills extract extrae memoria procedimental (cómo hacer cosas).

```python
skills = db.extract_skills(namespace="procedimientos")
for skill in skills:
    print(skill)
```

Ver API Reference
```

- Actualiza Quickstart con ejemplos de governance

---

**3. Tests de Integración para Governance:**

- Tests para entity resolution
- Tests para conflict detection
- Tests para skills extract
- Asegúrate de que CI pasa

---

**4. Publicar v0.7.0:**

- Actualiza `version` en `pyproject.toml` y `package.json` a `0.7.0`
- Sube a PyPI y npm
- Crea release en GitHub (`v0.7.0`)
- Anuncia en GitHub Discussions + Discord + Twitter

**Formato de Post:**

```
# VantaDB v0.7.0 - Governance Ready

¡Hola comunidad! 👋

VantaDB v0.7.0 está disponible. Esta release introduce **governance básica** del ciclo de vida de la memoria.

## Qué hay de Nuevo

### Features

- ✅ **Entity Resolution Manual** - Marca duplicados manualmente
- ✅ **Conflict Detection** - Detecta contradicciones en la memoria
- ✅ **Skills Extract** - Extrae memoria procedimental

### Mejoras

- ✅ Mejoras en búsqueda híbrida (RRF tuning)
- ✅ Mejoras en performance (20% más rápido en benchmarks)

### Docs

- 📖 Nueva página: Governance
- 📖 Ejemplos actualizados en Quickstart

## Upgrade

### Python

```bash
pip install --upgrade vantadb-py
```

### TypeScript

```bash
npm install vantadb@latest
```

## Qué Viene en v1.0.0

- 🔵 Entity resolution automática
- 🔵 Conflict resolution automática
- 🔵 Consolidación automática L2→L3
- 🔵 Auditoría automática de vigencia
- 🔵 Docs completas
- 🔵 Tests de integración
- 🔵 Benchmarks de performance

Ver Roadmap

---

**SyntropyOS** — Orden desde el caos.
```

---

**Checklist para v0.7.0:**

- Entity resolution manual implementada
- Conflict detection implementado (si es posible)
- Skills extract implementado (si es posible)
- Docs de governance completas
- Tests de integración pasando
- Publicado en PyPI y npm
- Release en GitHub
- Anuncio en GitHub Discussions + Discord + Twitter

---

**Checklist Fase 2 (Semanas 5-12):**

- Rutina semanal fija (Lunes, Miércoles, Viernes, Domingo)
- v0.6.0 publicado con features de usuarios
- Primeros contribuidores externos reconocidos
- v0.7.0 publicado con governance básica
- 50-100 usuarios activos
- Comunidad activa en Discord (XX mensajes/semana)

---

*(Continuaré con Fase 3 en el siguiente mensaje...)*

---

## Fase 3: Preparación para v1.0.0 (Semanas 13-20)

### Objetivo:

VantaDB **"completo" para producción**, listo para lanzamiento oficial.

---

### 📅 Tareas Críticas para v1.0.0

---

#### 1. **VantaDB - Features Completas**

**Objetivo:**

Todas las features de governance completas y funcionales.

---

#### 1.1. Entity Resolution Automática

> ⚠️ PROPUESTA v1.0.0 — `auto_resolve_entities` no existe hoy. Todo el código de esta subsección es especificación a diseñar, no API a documentar.

**Tareas:**

- **Implementar Entity Resolution Automática**:
    - Algoritmo para detectar duplicados (ej: similarity scoring con fuzzy matching)
    - API: `db.auto_resolve_entities(namespace)` devuelve lista de duplicados detectados
    - Opcional: Auto-merge de duplicados (con threshold configurable)
- **Tests para Entity Resolution**:
    - Tests unitarios para el algoritmo de detección
    - Tests de integración para la API
    - Benchmarks de performance (tiempo de ejecución, memory usage)
- **Docs para Entity Resolution**:
    - Página `entity-resolution.md` en docs:
        
        ```
        # Entity Resolution
        
        Entity resolution detecta duplicados automáticamente.
        
        ## Automática (v1.0.0)
        
        ```python
        duplicates = db.auto_resolve_entities(namespace="empresas")
        for dup in duplicates:
            print(f"{dup['key1']} = {dup['key2']} (similarity: {dup['similarity']})")
        ```
        
        ## Manual (v0.7.0)
        
        ```python
        db.mark_duplicate(namespace="empresas", key1="acme-corp", key2="acme-corporation")
        ```
        
        Ver API Reference
        ```
        
    - Ejemplos en Quickstart

---

#### 1.2. Conflict Detection y Resolución

> ⚠️ PROPUESTA v1.0.0 — `detect_conflicts`/`resolve_conflict` no existen hoy. Especificación a diseñar.

**Tareas:**

- **Implementar Conflict Detection**:
    - Algoritmo para detectar contradicciones (ej: payloads opuestos para misma key)
    - API: `db.detect_conflicts(namespace)` devuelve lista de conflictos
    - API: `db.resolve_conflict(namespace, conflict_id, resolution)` resuelve conflicto
- **Tests para Conflict Detection**:
    - Tests unitarios para el algoritmo
    - Tests de integración para las APIs
    - Benchmarks de performance
- **Docs para Conflict Detection**:
    - Página `conflict-detection.md` en docs:
        
        ```
        # Conflict Detection
        
        Conflict detection detecta contradicciones en la memoria.
        
        ## Detectar Conflictos
        
        ```python
        conflicts = db.detect_conflicts(namespace="prefs")
        for conflict in conflicts:
            print(f"Conflicto: {conflict['key1']} vs {conflict['key2']}")
        ```
        
        ## Resolver Conflictos
        
        ```python
        db.resolve_conflict(namespace="prefs", conflict_id="conf-123", resolution="keep_first")
        ```
        
        Ver API Reference
        ```
        
    - Ejemplos en Quickstart

---

#### 1.3. Skills Extract (Memoria Procedimental)

> ⚠️ PROPUESTA v1.0.0 — `extract_skills`/`use_skill` no existen hoy. Especificación a diseñar.

**Tareas:**

- **Implementar Skills Extract**:
    - Algoritmo para extraer memoria procedimental (ej: patrones de uso, secuencias)
    - API: `db.extract_skills(namespace)` devuelve lista de skills
    - Opcional: `db.use_skill(namespace, skill_id)` aplica un skill
- **Tests para Skills Extract**:
    - Tests unitarios para el algoritmo
    - Tests de integración para las APIs
    - Benchmarks de performance
- **Docs para Skills Extract**:
    - Página `skills-extract.md` en docs:
        
        ```
        # Skills Extract
        
        Skills extract extrae memoria procedimental (cómo hacer cosas).
        
        ## Extraer Skills
        
        ```python
        skills = db.extract_skills(namespace="procedimientos")
        for skill in skills:
            print(f"Skill: {skill['name']} - {skill['description']}")
        ```
        
        ## Usar Skills
        
        ```python
        db.use_skill(namespace="procedimientos", skill_id="skill-123")
        ```
        
        Ver API Reference
        ```
        
    - Ejemplos en Quickstart

---

#### 1.4. Consolidación Automática L2→L3

> ⚠️ PROPUESTA v1.0.0 — existe `consolidate_node` interno del storage, pero NO `db.consolidate(namespace)`. Diseñar la API antes de documentarla.

**Tareas:**

- **Implementar Consolidación Automática**:
    - Algoritmo para consolidar memoria L2 (corto plazo) a L3 (largo plazo)
    - API: `db.consolidate(namespace)` consolida manualmente
    - Opcional: Auto-consolidación periódica (background job)
- **Tests para Consolidación**:
    - Tests unitarios para el algoritmo
    - Tests de integración para las APIs
    - Benchmarks de performance
- **Docs para Consolidación**:
    - Página `consolidation.md` en docs:
        
        ```
        # Consolidación L2→L3
        
        Consolidación automática de memoria corto plazo (L2) a largo plazo (L3).
        
        ## Manual
        
        ```python
        db.consolidate(namespace="memoria")
        ```
        
        ## Automática (Background)
        
        En v1.0.0, consolidación ocurre automáticamente cada X horas.
        
        Ver API Reference
        ```
        
    - Ejemplos en Quickstart

---

#### 1.5. Auditoría Automática de Vigencia

> ⚠️ PROPUESTA v1.0.0 — `audit_vigency` no existe hoy. Especificación a diseñar.

**Tareas:**

- **Implementar Auditoría Automática**:
    - Algoritmo para auditar vigencia de registros (ej: TTL expirado, supersession)
    - API: `db.audit_vigency(namespace)` audita manualmente
    - Opcional: Auto-auditoría periódica (background job)
- **Tests para Auditoría**:
    - Tests unitarios para el algoritmo
    - Tests de integración para las APIs
    - Benchmarks de performance
- **Docs para Auditoría**:
    - Página `vigency-audit.md` en docs:
        
        ```
        # Auditoría de Vigencia
        
        Auditoría automática de vigencia de registros (TTL, supersession).
        
        ## Manual
        
        ```python
        db.audit_vigency(namespace="sesion")
        ```
        
        ## Automática (Background)
        
        En v1.0.0, auditoría ocurre automáticamente cada X horas.
        
        Ver API Reference
        ```
        
    - Ejemplos en Quickstart

---

#### 1.6. Benchmarks de Performance

**Tareas:**

- **Crear Script de Benchmarks**:
    - Script `benchmarks.py` que mide:
        - Tiempo de `put()` (100, 1000, 10000 registros)
        - Tiempo de `search_memory()` (100, 1000, 10000 registros)
        - Tiempo de `add_edge()`, `graph_bfs()`
        - Tiempo de entity resolution, conflict detection, skills extract
        - Memory usage en cada operación
- **Correr Benchmarks**:
    - Corre benchmarks en tu máquina
    - Guarda resultados en `docs/benchmarks.md`:

```
# Benchmarks

## Hardware

- CPU: [tu CPU]
- RAM: [tu RAM]
- OS: [tu OS]

## Resultados

### Put

| Registros | Tiempo (ms) |
|-----------|-------------|
| 100       | XX          |
| 1000      | XX          |
| 10000     | XX          |

### Search

| Registros | Tiempo (ms) |
|-----------|-------------|
| 100       | XX          |
| 1000      | XX          |
| 10000     | XX          |

### Entity Resolution

| Registros | Tiempo (ms) |
|-----------|-------------|
| 100       | XX          |
| 1000      | XX          |
| 10000     | XX          |

Ver script de benchmarks
```

- **Publicar Benchmarks**:
    - Agrega sección "Performance" en docs
    - Incluye tabla de benchmarks

---

**Checklist para Features Completas:**

- Entity resolution automática implementada
- Conflict detection y resolución implementados
- Skills extract implementado
- Consolidación automática L2→L3 implementada
- Auditoría automática de vigencia implementada
- Tests unitarios y de integración para todas las features
- Benchmarks de performance creados y publicados
- Docs completas para todas las features

---

#### 2. **Docs Completas**

**Objetivo:**

Docs que cualquier usuario nuevo pueda seguir sin ayuda.

---

#### 2.1. User Guide Completo

**Tareas:**

- **Crear `user-guide.md`**:

```
# User Guide

## Instalación

Ver Installation

## Quickstart

Ver Quickstart

## Conceptos Clave

### Namespaces

Los namespaces son como "carpetas" para organizar tu memoria.

```python
db.put(namespace="prefs", key="editor", payload="usa tabs")
db.put(namespace="sesion", key="token", payload="Bearer abc")
```

### TTL (Time To Live)

TTL es la duración de un registro. Después de ese tiempo, expira automáticamente.

```python
db.put(
    namespace="sesion",
    key="token",
    payload="Bearer abc",
    ttl_ms=90*24*3600*1000  # top-level, NO dentro de metadata
)
```

### Grafos

Los grafos conectan entidades por ID numérico (no por namespace+key).

```python
db.add_edge(1, 2, "decidio")
result = db.graph_bfs([1], 2)
```

### Entity Resolution — [PROPUESTA v0.7+, NO existe en v0.5.0]

Entity resolution detecta duplicados.

```python
# FUTURO (especificación propuesta, hoy TypeError: método inexistente)
# duplicates = db.auto_resolve_entities(namespace="empresas")
```

### Conflict Detection — [PROPUESTA v0.7+, NO existe en v0.5.0]

Conflict detection detecta contradicciones en la memoria.

```python
# FUTURO (especificación propuesta, hoy TypeError: método inexistente)
# conflicts = db.detect_conflicts(namespace="prefs")
```

## Casos de Uso

### Agente Personal

Ver caso de uso

### RAG para Docs

Ver caso de uso

### Grafos de Conocimiento

Ver caso de uso

## FAQ

Ver FAQ

## API Reference

Ver API Reference
```

- **Crear Casos de Uso**:
    - `use-cases/personal-agent.md`
    - `use-cases/rag.md`
    - `use-cases/knowledge-graphs.md`

**Formato de Caso de Uso:**

```
# Caso de Uso: Agente Personal

## Descripción

Un agente personal que recuerda tus preferencias, historial, y contexto.

## Setup

```python
import vantadb

db = vantadb.VantaDB("./my_brain")
```

## Ejemplos

### Guardar Preferencias

```python
db.put(namespace="prefs", key="editor", payload="usa tabs")
db.put(namespace="prefs", key="tema", payload="oscuro")
```

### Buscar Preferencias

```python
result = db.search_memory("prefs", [], text_query="formateo")
print(result)
```

### Guardar Historial

```python
db.put(
    namespace="historial",
    key="reunion-2026-09-08",
    payload="Reunión con equipo de producto. Decisiones: X, Y, Z."
)
```

### Buscar Historial

```python
result = db.search_memory("historial", [], text_query="reunión producto")
print(result)
```

## Entity Resolution

```python
duplicates = db.auto_resolve_entities(namespace="contactos")
for dup in duplicates:
    print(f"{dup['key1']} = {dup['key2']}")
```

## Tips

- Usa namespaces para organizar (prefs, historial, contactos, etc.)
- Usa TTL para datos temporales (sesiones, tokens)
- Usa grafos para conectar entidades (persona → decisión)

Ver otros casos de uso
```

---

#### 2.2. API Reference Completa

**Tareas:**

- **Actualizar `api-reference.md`**:

```
# API Reference

## VantaDB Class

### `constructor(path: string)`

Crea una nueva base de datos.

```python
db = vantadb.VantaDB("./my_brain")
```

### `put(namespace, key, payload, metadata=None)`

Guarda un registro.

**Parámetros:**

- `namespace` (str): Namespace del registro
- `key` (str): Key única del registro
- `payload` (str | dict): Payload del registro
- `metadata` (dict, opcional): Metadatos libres. NOTA: `ttl_ms` NO va dentro de `metadata` — es parámetro top-level de `put()`

**Ejemplo:**

```python
db.put(
    namespace="prefs",
    key="editor",
    payload="usa tabs",
    ttl_ms=90*24*3600*1000  # Opcional: TTL en ms, top-level
)
```

### `search_memory(namespace, query_vector, filters=None, text_query=None, top_k=10)`

Busca registros. `query_vector` requerido (`[]` = solo texto).

**Parámetros:**

- `namespace` (str): Namespace a buscar
- `query_vector` (list, requerido): Query vector; `[]` para solo BM25
- `text_query` (str, opcional): Query de texto
- `top_k` (int, opcional): Número de resultados (default: 10)

**Ejemplo:**

```python
result = db.search_memory(
    "prefs",
    [],
    text_query="formateo",
    top_k=5
)
```

### `auto_resolve_entities(namespace)` — [PROPUESTA, NO existe]

Detecta duplicados automáticamente.

**Parámetros:**

- `namespace` (str): Namespace a auditar

**Ejemplo (futuro):**

```python
# duplicates = db.auto_resolve_entities(namespace="empresas")
# for dup in duplicates:
#     print(f"{dup['key1']} = {dup['key2']} (similarity: {dup['similarity']})")
```

### `detect_conflicts(namespace)` — [PROPUESTA, NO existe]

Detecta contradicciones.

**Parámetros:**

- `namespace` (str): Namespace a auditar

**Ejemplo (futuro):**

```python
# conflicts = db.detect_conflicts(namespace="prefs")
# for conflict in conflicts:
#     print(f"Conflicto: {conflict['key1']} vs {conflict['key2']}")
```

### `extract_skills(namespace)` — [PROPUESTA, NO existe]

Extrae memoria procedimental.

**Parámetros:**

- `namespace` (str): Namespace a extraer

**Ejemplo (futuro):**

```python
# skills = db.extract_skills(namespace="procedimientos")
# for skill in skills:
#     print(f"Skill: {skill['name']} - {skill['description']}")
```

### `add_edge(source_id, target_id, label, weight=None, created_at_ms=None)`

Agrega una arista al grafo (IDs numéricos u128).

**Parámetros:**

- `source_id` (int): ID del nodo origen
- `target_id` (int): ID del nodo destino
- `label` (str): Label de la arista
- `weight` (float, opcional): Peso de la arista

**Ejemplo:**

```python
db.add_edge(1, 2, "decidio")
```

### `graph_bfs(roots, max_depth=999999, direction="Forward")`

BFS en el grafo desde IDs raíz.

**Parámetros:**

- `roots` (list[int]): IDs iniciales
- `max_depth` (int, opcional): Profundidad máxima
- `direction` (str, opcional): "Forward", "Reverse" o "Both"

**Ejemplo:**

```python
result = db.graph_bfs([1], 2)
```

Ver código fuente
```

---

#### 2.3. Tutorials Paso a Paso

**Tareas:**

- **Crear al menos 3 tutorials**:

**Tutorial 1: "Tu Primer Agente Personal"**

```
# Tutorial: Tu Primer Agente Personal

En este tutorial, crearás un agente personal que recuerda tus preferencias y historial.

## Paso 1: Instalación

```bash
pip install vantadb-py
```

## Paso 2: Crear Base de Datos

```python
import vantadb

db = vantadb.VantaDB("./my_brain")
```

## Paso 3: Guardar Preferencias

```python
db.put(namespace="prefs", key="editor", payload="usa tabs")
db.put(namespace="prefs", key="tema", payload="oscuro")
```

## Paso 4: Buscar Preferencias

```python
result = db.search_memory("prefs", [], text_query="formateo")
print(result)
```

## Paso 5: Guardar Historial

```python
db.put(
    namespace="historial",
    key="reunion-2026-09-08",
    payload="Reunión con equipo de producto. Decisiones: X, Y, Z."
)
```

## Paso 6: Buscar Historial

```python
result = db.search_memory("historial", [], text_query="reunión producto")
print(result)
```

## Siguiente Paso

- Ver más casos de uso
- Ver API Reference
```

**Tutorial 2: "RAG para Documentos"**

**Tutorial 3: "Grafos de Conocimiento"**

---

#### 2.4. FAQs Actualizadas

**Tareas:**

- **Actualizar `faq.md` con al menos 10 preguntas**:

```
# FAQ

## ¿Qué es VantaDB?

VantaDB es memoria persistente gobernada para agentes de IA. Es el holón de memoria de SyntropyOS.

## ¿Está listo para producción?

Sí. v1.0.0 es production-ready. Tiene entity resolution, conflict detection, skills extract, consolidación, y auditoría automáticas.

## ¿Cómo se compara con Qdrant/Chroma?

VantaDB es local-first (sin servidor), con gobernanza del ciclo de vida de la memoria. Qdrant/Chroma son vector stores pasivos.

## ¿Puedo usarlo comercialmente?

Sí. Licencia Apache 2.0 permite uso comercial.

## ¿Cómo contribuyo?

Buscamos ayuda con docs, tests, y feedback. Únete a Discord o abre un issue en GitHub.

## ¿Qué sigue?

Próximos holones: Iris (visión), Cardinal (orientación), Sage (aprendizaje).

Ver Roadmap

## ¿VantaDB funciona en Windows?

Sí, funciona en Windows, macOS, Linux. Python 3.11+ o Node 18+ requeridos.

## ¿Cómo uso TTL?

Agrega `ttl_ms=90*24*3600*1000` como parámetro top-level al `put()`. Ver Quickstart.

## ¿Entity resolution ya funciona?

Sí. v1.0.0 tiene entity resolution automática. Ver Entity Resolution.

## ¿Conflict detection ya funciona?

Sí. v1.0.0 tiene conflict detection y resolución. Ver Conflict Detection.
```

---

#### 2.5. Migration Guide (si hubo breaking changes desde v0.x)

**Tareas:**

- **Crear `migration-guide.md`** (si hubo breaking changes):

```
# Migration Guide

## v0.7.0 → v1.0.0

### Breaking Changes (solo documentar los reales; verificar en código antes de publicar)

- (Ningún breaking change confirmado a la fecha. No inventar cambios de firma: `search_memory` siempre requirió `namespace` primero y `ttl_ms` siempre fue top-level.)

### Cómo Migrar

> Nota de migración honesta (verificada en código): `search_memory` NO cambió de forma entre v0.5.0 y v1.0.0. La forma correcta en ambas es con `query_vector` requerido (`[]` = solo texto). No documentar un breaking change que nunca existió.

#### Antes = Después (v0.5.0 y v1.0.0, misma firma)

```python
db.search_memory("prefs", [], text_query="formateo")
```

#### Sin TTL (registro permanente)

```python
db.put(namespace="sesion", key="token", payload="Bearer abc")  # Sin TTL
```

#### Con TTL (parámetro top-level, NUNCA dentro de `metadata`)

```python
db.put(namespace="sesion", key="token", payload="Bearer abc", ttl_ms=90*24*3600*1000)
```

### Nuevas Features

- Entity resolution automática
- Conflict detection y resolución
- Skills extract
- Consolidación automática L2→L3
- Auditoría automática de vigencia

Ver Changelog
```

---

#### 2.6. Changelog Completo (v0.5.0 → v1.0.0)

**Tareas:**

- **Actualizar `CHANGELOG.md`**:

```
# Changelog

## v1.0.0 (2026-XX-XX)

### Features

- ✅ Entity resolution automática
- ✅ Conflict detection y resolución
- ✅ Skills extract (memoria procedimental)
- ✅ Consolidación automática L2→L3
- ✅ Auditoría automática de vigencia
- ✅ Benchmarks de performance

### Mejoras

- ✅ Mejoras en búsqueda híbrida (RRF tuning)
- ✅ Mejoras en performance (30% más rápido que v0.7.0)

### Docs

- 📖 User guide completo
- 📖 API reference completa
- 📖 3 tutorials paso a paso
- 📖 FAQs actualizadas (10+ preguntas)
- 📖 Migration guide (v0.7.0 → v1.0.0)

## v0.7.0 (2026-09-XX)

### Features

- ✅ Entity resolution manual
- ✅ Conflict detection
- ✅ Skills extract

### Mejoras

- ✅ Mejoras en búsqueda híbrida
- ✅ Mejoras en performance (20% más rápido que v0.6.0)

### Docs

- 📖 Página de governance
- 📖 Ejemplos actualizados en Quickstart

## v0.6.0 (2026-09-XX)

### Features

- ✅ Entity resolution manual (API básica)
- ✅ Mejoras en RRF tuning

### Fixes

- 🐛 [Bug fix 1]
- 🐛 [Bug fix 2]

### Docs

- 📖 [Doc update 1]
- 📖 [Doc update 2]

## v0.5.1 (2026-09-XX)

### Fixes

- 🐛 Fixeado bug #42 (crash en Windows)
- 🐛 Mejorado error messages en Python binding

### Docs

- 📖 Aclarado TTL en docs (unidades en ms)
- 📖 Agregado ejemplo de RAG en Quickstart

## v0.5.0 (2026-08-01)

### Features

- ✅ Persistencia durable (WAL + fsync + CRC32C)
- ✅ Búsqueda híbrida (BM25 + HNSW + RRF)
- ✅ Filtrado por metadatos y namespaces
- ✅ TTL y expiración automática
- ✅ Grafos de entidades (BFS, DFS, PageRank)
- ✅ Supersession (marcar registros como obsoletos)
- ✅ Bindings Python y TypeScript

Ver releases en GitHub
```

---

**Checklist para Docs Completas:**

- User guide completo
- API reference completa
- 3+ tutorials paso a paso
- FAQs actualizadas (10+ preguntas)
- Migration guide (si hubo breaking changes)
- Changelog completo (v0.5.0 → v1.0.0)
- Casos de uso (personal agent, RAG, knowledge graphs)
- Benchmarks de performance en docs

---

#### 3. **Casos de Uso en Producción**

**Objetivo:**

3-5 usuarios usando VantaDB en producción (no solo testing).

---

#### 3.1. Conseguir Usuarios en Producción

**Tareas:**

- **Identificar Usuarios Tempranos Activos**:
    - Revisa Discord: ¿quién ha estado activo?
    - Revisa GitHub: ¿quién ha abierto issues, PRs?
    - Identifica 5-10 usuarios que podrían usar VantaDB en producción
- **Contactar Usuarios**:

**Formato de Mensaje (Discord DM o Email):**

```
¡Hola [Nombre]! 👋

Vi que has estado usando VantaDB. ¡Gracias por ser parte de la comunidad!

Estoy preparando el lanzamiento de v1.0.0 (production-ready) y me gustaría saber:

¿Estarías dispuesto/a a usar VantaDB en producción (no solo testing)?

Si sí, me encantaría:
- Escuchar tu experiencia
- Ayudarte con cualquier problema
- Feature tu caso de uso en las docs (si quieres)

¡Avísame!

[Tu Nombre]
Fundador, SyntropyOS
```

- **Trackear Respuestas**:
    - Crea doc interno (`docs/production-users.md`):

```
# Usuarios en Producción

## Usuarios

| Usuario | Caso de Uso | Fecha | Feedback |
|---------|-------------|-------|----------|
| @user1  | Agente personal | 2026-XX-XX | "VantaDB funciona bien en producción. Uso entity resolution para contactos." |
| @user2  | RAG para docs | 2026-XX-XX | "Búsqueda híbrida es rápida. Conflict detection me ayudó a evitar contradicciones." |

## Testimonios

- @user1: "VantaDB es mi memoria personal. Lo uso todos los días."
- @user2: "VantaDB es el backend de mi sistema RAG. Funciona bien."

Ver casos de uso
```

---

#### 3.2. Pedir Testimonios/Casos de Éxito

**Tareas:**

- **Pedir Testimonios**:

**Formato de Mensaje:**

```
¡Hola [Nombre]! 👋

VantaDB v1.0.0 está por lanzarse y me encantaría feature tu caso de uso en las docs.

¿Podrías escribir 2-3 frases sobre:
- ¿Para qué usas VantaDB?
- ¿Qué te parece?
- ¿Lo recomendarías?

Ejemplo:

"Uso VantaDB para mi agente personal. Recomiendo VantaDB porque es rápido y fácil de usar."

¡Gracias!

[Tu Nombre]
```

- **Agregar Testimonios en Docs**:

```
## Testimonios

> "Uso VantaDB para mi agente personal. Recomiendo VantaDB porque es rápido y fácil de usar."
> — @user1

> "VantaDB es el backend de mi sistema RAG. La búsqueda híbrida es increíble."
> — @user2

Ver casos de uso
```

---

#### 3.3. Documentar Casos de Uso en Docs

**Tareas:**

- **Crear Página `use-cases.md`**:

```
# Casos de Uso

## Agente Personal

@user1 usa VantaDB para su agente personal.

> "Uso VantaDB para mi agente personal. Recomiendo VantaDB porque es rápido y fácil de usar."

Ver tutorial

## RAG para Documentos

@user2 usa VantaDB para RAG.

> "VantaDB es el backend de mi sistema RAG. La búsqueda híbrida es increíble."

Ver tutorial

## Grafos de Conocimiento

@user3 usa VantaDB para grafos de conocimiento.

> "VantaDB me permite conectar entidades y navegar por ellas. Los grafos son poderosos."

Ver tutorial

Ver todos los casos de uso
```

---

**Checklist para Casos de Uso en Producción:**

- 3-5 usuarios identificados y contactados
- 3-5 usuarios usando VantaDB en producción
- Testimonios pedidos y recibidos
- Casos de uso documentados en docs
- Página `use-cases.md` creada

---

#### 4. **Preparación para Lanzamiento Oficial**

**Objetivo:**

Todo listo para anunciar v1.0.0 en Hacker News, Reddit, Twitter, etc.

---

#### 4.1. Post para Hacker News

> [PAUSADO hasta gate Fase A en verde, decisión 2026-09-08]

**Tareas:**

- **Preparar Post**:

**Título:**

```
VantaDB v1.0.0 - Memoria gobernada para agentes de IA
```

**Contenido:**

```
¡Hola HN! 👋

Lanzo VantaDB v1.0.0: memoria persistente gobernada para agentes de IA.

**¿Qué es VantaDB?**

VantaDB es el holón de memoria de SyntropyOS. Provee:

- Persistencia durable (WAL + fsync)
- Búsqueda híbrida (BM25 + HNSW + RRF)
- Entity resolution automática
- Conflict detection y resolución
- Skills extract (memoria procedimental)
- Consolidación y auditoría automáticas

**¿Por qué importa?**

Los agentes de IA actuales son frágiles. No tienen memoria persistente, no pueden razonar sobre su propio estado, no pueden colaborar. VantaDB resuelve esto con gobernanza del ciclo de vida de la memoria.

**Links:**

- GitHub: https://github.com/ness-e/Vantadb
- Docs: https://vantadb.vercel.app
- Discord: https://discord.gg/g8nqB3NtXt

**Stack:**

- Python (Rust embebido con bindings)
- TypeScript (bindings)
- Local-first (sin servidor)

**Feedback bienvenido!** 🙌
```

- **Publicar en Hacker News**:
    - Ve a news.ycombinator.com
    - Click en "submit"
    - Título: `VantaDB v1.0.0 - Memoria gobernada para agentes de IA`
    - URL: `https://vantadb.vercel.app` o `https://github.com/ness-e/Vantadb`
    - Texto: Copia el contenido de arriba
    - Publica

---

#### 4.2. Posts para Reddit

> [PAUSADO hasta gate Fase A en verde, decisión 2026-09-08]

**Tareas:**

- **Preparar Posts para Subreddits**:

**Subreddits Sugeridos:**

- r/MachineLearning
- r/LocalLLaMA
- r/ArtificialIntelligence
- r/Python
- r/TypeScript
- r/OpenSource

**Formato de Post (r/MachineLearning):**

```
# VantaDB v1.0.0 - Memoria gobernada para agentes de IA

¡Hola r/MachineLearning! 👋

Lanzo VantaDB v1.0.0: memoria persistente gobernada para agentes de IA.

**¿Qué es VantaDB?**

VantaDB es el holón de memoria de SyntropyOS. Provee:

- Persistencia durable (WAL + fsync)
- Búsqueda híbrida (BM25 + HNSW + RRF)
- Entity resolution automática
- Conflict detection y resolución
- Skills extract (memoria procedimental)
- Consolidación y auditoría automáticas

**¿Por qué importa?**

Los agentes de IA actuales son frágiles. No tienen memoria persistente, no pueden razonar sobre su propio estado, no pueden colaborar. VantaDB resuelve esto con gobernanza del ciclo de vida de la memoria.

**Links:**

- GitHub: https://github.com/ness-e/Vantadb
- Docs: https://vantadb.vercel.app
- Discord: https://discord.gg/g8nqB3NtXt

**Stack:**

- Python (Rust embebido con bindings)
- TypeScript (bindings)
- Local-first (sin servidor)

**Feedback bienvenido!** 🙌
```

- **Publicar en Subreddits**:
    - r/MachineLearning
    - r/LocalLLaMA
    - r/ArtificialIntelligence
    - r/Python
    - r/TypeScript
    - r/OpenSource

**Nota:** Lee las reglas de cada subreddit antes de publicar. Algunos requieren flair, algunos no permiten self-promotion, etc.

---

#### 4.3. Thread para Twitter

> [PAUSADO hasta gate Fase A en verde, decisión 2026-09-08]

> Regla: publicar SOLO los tweets cuyas features existan en la release anunciada. Reescribir cualquier tweet que afirme capacidades no verificadas.

**Tareas:**

- **Preparar Thread de 10-15 Tweets**:

**Tweet 1:**

```
🧵 VantaDB v1.0.0 - Memoria gobernada para agentes de IA

Lanzo VantaDB, el holón de memoria de SyntropyOS.

¿Qué es? ¿Por qué importa? ¿Cómo usarlo?

Te lo cuento en este thread. 👇
```

**Tweet 2:**

```
2/ ¿Qué es VantaDB?

VantaDB es memoria persistente gobernada para agentes de IA.

Provee:
- Persistencia durable (WAL + fsync)
- Búsqueda híbrida (BM25 + HNSW + RRF)
- Entity resolution automática
- Conflict detection
- Skills extract
```

**Tweet 3:**

```
3/ ¿Por qué "gobernada"?

La memoria no es pasiva. Tiene:
- TTL (expiración automática)
- Entity resolution (detecta duplicados)
- Conflict detection (detecta contradicciones)
- Consolidación (L2→L3)
- Auditoría de vigencia

Es como un cerebro, no un disco duro.
```

**Tweet 4:**

```
4/ ¿Por qué importa?

Los agentes de IA actuales son frágiles:
- No tienen memoria persistente
- No pueden razonar sobre su estado
- No pueden colaborar

VantaDB resuelve esto con gobernanza del ciclo de vida.
```

**Tweet 5:**

```
5/ Ejemplo de uso:

import vantadb

db = vantadb.VantaDB("./my_brain")

db.put(namespace="prefs", key="editor", payload="usa tabs")

result = db.search_memory("prefs", [], text_query="formateo")
print(result)
```

**Tweet 6:**

```
6/ Búsqueda híbrida:

VantaDB combina:
- BM25 (texto)
- HNSW (vectores)
- RRF (fusión)

Resultado: búsqueda más precisa que solo vectores o solo texto.
```

**Tweet 7:**

```
7/ Entity resolution:

db.auto_resolve_entities(namespace="empresas")

Detecta duplicados como:
- "Acme Corp" = "Acme Corporation"
- "OpenAI" = "Open AI"

Automáticamente.
```

**Tweet 8:**

```
8/ Conflict detection:

db.detect_conflicts(namespace="prefs")

Detecta contradicciones:
- "editor: tabs" vs "editor: spaces"
- "tema: oscuro" vs "tema: claro"

Te avisa para que resuelvas.
```

**Tweet 9:**

```
9/ Grafos de entidades (IDs numéricos):

db.add_edge(1, 2, "decidio")

result = db.graph_bfs([1], 2)
```

**Tweet 10:**

```
10/ Local-first:

VantaDB es local-first (sin servidor).

Tus datos, tu infra.

Opcionalmente distribuido cuando es necesario.
```

**Tweet 11:**

```
11/ Stack:

- Python (Rust embebido con bindings)
- TypeScript (bindings)
- Apache 2.0 (open source)

GitHub: https://github.com/ness-e/Vantadb
Docs: https://vantadb.vercel.app
```

**Tweet 12:**

```
12/ SyntropyOS:

VantaDB es el holón de memoria de SyntropyOS.

Próximos holones:
- Iris (visión)
- Cardinal (orientación)
- Sage (aprendizaje)
- Execute (ejecución)
- Plan (planificación)
```

**Tweet 13:**

```
13/ Comunidad:

Únete a Discord:
https://discord.gg/g8nqB3NtXt

- Chat, soporte, anuncios
- primeros miembros (verificar conteo real al publicar)
- Contribuidores bienvenidos
```

**Tweet 14:**

```
14/ Feedback:

¿Qué te parece VantaDB?

¿Lo usarías para tu agente de IA?

¿Qué feature te gustaría ver?

¡Feedback bienvenido! 🙌
```

**Tweet 15:**

```
15/ Gracias:

Gracias por leer este thread.

Si te interesa:
- GitHub: https://github.com/ness-e/Vantadb
- Docs: https://vantadb.vercel.app
- Discord: https://discord.gg/g8nqB3NtXt

¡Espero tus comentarios!

#AI #OpenSource #MachineLearning
```

- **Publicar Thread**:
    - Publica Tweet 1
    - Responde a Tweet 1 con Tweet 2
    - Responde a Tweet 2 con Tweet 3
    - ... (hasta Tweet 15)

---

#### 4.4. Demo en Vivo (Video)

> [PAUSADO hasta gate Fase A en verde + VantaDB funcionando, decisión 2026-09-08]

**Tareas:**

- **Grabar Video de 10-15 min**:

**Guion de Video:**

```
# Guion de Video: VantaDB v1.0.0 Demo

## Intro (0:00 - 1:00)

- Hola, soy [Tu Nombre], fundador de SyntropyOS
- Hoy te muestro VantaDB v1.0.0
- Memoria gobernada para agentes de IA

## Instalación (1:00 - 3:00)

- pip install vantadb-py
- Verificar instalación

## Quickstart (3:00 - 6:00)

- Crear base de datos
- Put, search, TTL
- Ejemplo de agente personal

## Features Avanzadas (6:00 - 10:00)

- Entity resolution
- Conflict detection
- Skills extract
- Grafos

## Casos de Uso (10:00 - 12:00)

- Agente personal
- RAG para docs
- Grafos de conocimiento

## Cierre (12:00 - 13:00)

- GitHub: https://github.com/ness-e/Vantadb
- Docs: https://vantadb.vercel.app
- Discord: https://discord.gg/g8nqB3NtXt
- ¡Gracias!
```

- **Editar Video**:
    - Usa OBS Studio o similar para grabar
    - Usa DaVinci Resolve o similar para editar
    - Agrega subtítulos (opcional pero recomendado)
- **Subir a YouTube**:
    - Título: `VantaDB v1.0.0 - Memoria gobernada para agentes de IA (Demo)`
    - Descripción:
        
        ```
        VantaDB v1.0.0 es memoria persistente gobernada para agentes de IA.
        
        GitHub: https://github.com/ness-e/Vantadb
        Docs: https://vantadb.vercel.app
        Discord: https://discord.gg/g8nqB3NtXt
        
        #AI #OpenSource #MachineLearning
        ```
        
    - Tags: `AI`, `OpenSource`, `MachineLearning`, `Python`, `TypeScript`, `Database`, `Vector Search`

---

#### 4.5. Contactar Influencers/Técnicos Relevantes

> [PENDIENTE DE DECISIÓN — el usuario pidió explicación antes de decidir. Ver explicación en reporte 2026-09-08. No contactar grandes nombres sin estrategia.]

**Tareas:**

- **Identificar 5-10 Influencers/Técnicos**:
    - Gente que habla de IA, open source, bases de datos en Twitter, YouTube, blogs
    - Ej: @karpathy, @sama, @jasonwei, etc. (o gente más niche si no puedes con los grandes)
- **Contactar**:

**Formato de Email/DM:**

```
Asunto: VantaDB - Memoria gobernada para agentes de IA

Hola [Nombre],

Soy [Tu Nombre], fundador de SyntropyOS.

Construí VantaDB, memoria persistente gobernada para agentes de IA.

- Entity resolution automática
- Conflict detection
- Búsqueda híbrida (BM25 + HNSW + RRF)
- Local-first (sin servidor)

GitHub: https://github.com/ness-e/Vantadb
Docs: https://vantadb.vercel.app

¿Te gustaría verlo? Me encantaría tu feedback.

¡Gracias!

[Tu Nombre]
Fundador, SyntropyOS
```

- **Trackear Respuestas**:
    - Crea doc interno (`docs/influencer-outreach.md`):

```
# Influencer Outreach

## Contactados

| Nombre | Plataforma | Fecha | Respuesta |
|--------|------------|-------|-----------|
| @user1 | Twitter    | 2026-XX-XX | "Suena interesante, lo revisaré" |
| @user2 | YouTube    | 2026-XX-XX | No respondió aún |

## Respuestas Positivas

- @user1: "Suena interesante, lo revisaré"

## Próximos Pasos

- Seguir con @user2 en 1 semana
- Contactar a @user3, @user4
```

---

**Checklist para Lanzamiento Oficial:**

- Post para Hacker News preparado
- Posts para Reddit preparados (r/MachineLearning, r/LocalLLaMA, etc.)
- Thread para Twitter preparado (10-15 tweets)
- Demo en vivo (video) grabado y subido a YouTube
- 5-10 influencers/técnicos contactados
- Trackeo de respuestas en doc interno

---

## ✅ Checklist Final: ¿Estás Listo para Publicar?

### Antes de la Semana 1 (Lanzamiento Suave):

- VantaDB v0.5.0 funciona sin errores críticos
- README de `ness-e/Vantadb` actualizado con estado claro
- Docs básicas en `vantadb.vercel.app`
- `SyntropyOS/.github` con todos los docs canónicos
- Discord configurado con canales básicos
- Invite de Discord actualizado en todos los READMEs
- Paquetes PyPI y npm actualizados
- Tests mínimos corriendo en CI
- Issues templates en GitHub

### Antes de v1.0.0 (Lanzamiento Oficial):

- VantaDB tiene governance completa (entity resolution, conflict detection, etc.)
- Docs completas (user guide, API reference, tutorials, FAQs)
- 3-5 casos de uso en producción
- Benchmarks de performance
- Changelog completo (v0.5.0 → v1.0.0)
- Posts preparados para HN, Reddit, Twitter
- Demo en vivo (video) preparado
- 5-10 influencers contactados

---


### Métricas primarias (valor, no vanidad) — mandan sobre las de abajo

| Métrica | Objetivo inicial |
| --- | --- |
| Instalaciones limpias completadas | 5 |
| Usuarios que ejecutan el quickstart | 3 |
| Usuarios que vuelven a usarlo | 2 |
| Bugs críticos abiertos | 0 |
| Tiempo de respuesta | < 48 h |
| Feedback cualitativo | 5 conversaciones |
| Contribuciones externas | 1 pequeña |

## 📊 Métricas a Trackear

### Semanales:

- 📦 Downloads PyPI
- 📦 Downloads npm
- ⭐ GitHub Stars
- 💬 Discord Members
- 🐛 Issues abiertos/cerrados
- 🔄 PRs abiertos/mergeados

### Mensuales:

- 👥 Usuarios activos (Discord, GitHub)
- 📈 Crecimiento de comunidad
- 🎯 Hitos alcanzados (v0.6.0, v0.7.0, v1.0.0)

---

**¡Este es tu plan completo!** ¿Quieres que ajuste algo (más/menos detalle, otras tareas, otro timeline)?




---

# Apéndice Z — Tablas verificadas y agregados de la auditoría (2026-09-08)

> Todo lo de abajo es NUEVO respecto al plan original: sale de verificar cada afirmación contra código (`lib.rs`, `vantadb.ts`, `types.ts`, `wal.rs`, `gds.rs`), PyPI/npm, `git tag`, GitHub API y docs oficiales. Método: 3 subagentes en paralelo (API vs código, plataformas vs docs oficiales, estado de repos) + verificación directa del lead.

## Z.1 Tabla API real por binding (fuente: código, no memoria)

| Método | Python (`vantadb`) | TypeScript (`vantadb`) | Notas |
|---|---|---|---|
| Crear/abrir | `VantaDB("./my_brain")` | `VantaDB.create()` = solo memoria; persistente: `VantaDB.connect(path)` / `open(path)` | TS `create()` ignora `storage_path` |
| Guardar | `put(ns, key, payload, metadata=None, vector=None, ttl_ms=None)` | `put({namespace, key, payload, metadata?, vector?, ttl_ms?})` | `ttl_ms` TOP-LEVEL en ambos; en metadata queda inerte |
| Buscar | `search_memory(ns, query_vector, filters?, text_query?, top_k=10)` — vector requerido (`[]` = solo texto) | `search({namespace, query_vector, filters?, text_query?, top_k?})` — vector requerido | Texto-solo sin vector = `TypeError` / error TS |
| Grafo | `add_edge(source_id: u128, target_id, label)` · `graph_bfs(roots: list[int], …)` | `addEdge(source: number, target: number, label?)` · `graphBfs(roots: number[], …)` | IDs numéricos, sin namespace; NO existen formas objeto `{namespace, from_key…}` |
| Versión | `vantadb.__version__` | Sin API runtime (usar `package.json`; `capabilities()` no expone versión) | `VantaDB.version` NO existe |
| TTL purge | `purge_expired()` | `purgeExpired()` | ambos |
| Listar NS | `list_namespaces()` | `listNamespaces()` | ambos |
| PageRank | `graph_page_rank(roots, …)` Python-only | No expuesto en TS/wasm | Va por server (`POST /api/v2/graph/pagerank`) y MCP |
| Governance (`auto_resolve*`, `mark_duplicate`, `detect_*`, `extract_skills(ns)`, `consolidate(ns)`, `audit_vigency`, `resolve_*`, `use_skill`) | 0 hits en bindings | 0 hits | Solo internos con otra forma; todo lo que los use es [PROPUESTA] |

## Z.2 Qué se puede ejecutar desde la org vs qué requiere `ness-e/Vantadb` [BLOQUEADO ness-e]

| Ejecutable desde `SyntropyOS/.github` (ya hecho o hacible) | Requiere owner de `ness-e/Vantadb` (NO tocar sin orden) |
|---|---|
| Profile READMEs, MANIFESTO/CONTRIBUTING/ROADMAP/SECURITY/CoC, badges con formatos de §2.5, habilitar Discussions, `question.md` (bug/feature ya existen), topics/descripciones de repos org | README + quickstarts, `package.json` (`homepage` real: NO es `vantadb.dev` —muerto sin DNS— sino `vantadb.vercel.app`; agregar `engines`), `pyproject.toml`, CI, releases/tags, issues/PRs/discussions de ese repo |

## Z.3 Reglas de plataforma verificadas (fuentes oficiales)

- **HN Show HN**: producto usable sin signup, título `Show HN…`, autor presente, prohibido pedir votos (ban); cuentas nuevas visibles en verde + weighting anti-abuso → calentar cuenta antes. Fuentes: `news.ycombinator.com/showhn.html`, `/newsguidelines.html`, `/newsfaq.html`.
- **Reddit**: r/LocalLLaMA Regla 4 (10% autopromo como máximo + declarar afiliación, sin clickbait); r/MachineLearning ≈ cero promo directa; r/Python/TS/OpenSource/AI: leer cada sidebar (no verificadas, 403). No postear sin leer reglas + flair + karma.
- **PyPI**: `[project.urls]` libre (`Homepage/Documentation/Discord` válidos, PEP 753); sin classifier `License ::` (PEP 639). Fuente: `packaging.python.org`.
- **npm**: `homepage`/`engines` (advisory)/`prepublishOnly` vigentes; CI con `checkout@v5+`/`setup-node@v5+` (v4 obsoleto, latest v7). Publicación futura: trusted publishing OIDC o staged (tokens bypass-2FA pierden publish directo ~ene-2027). Fuentes: `docs.npmjs.com`, changelog `2026-07-08`.
- **GitHub**: Discussions se habilita por repo (Settings → Features) o a nivel org; pinnear soportado; `bug/enhancement/question` sí son labels por defecto. Fuente: `docs.github.com`.
- **Discord**: badge = `img.shields.io/discord/:serverId` + widget habilitado; invite permanente = expiración Never + usos ilimitados. Servidor real ya estructurado (ver §2.4 corregido).
- **Buymeacoffee `/ness`**: existe pero placeholder (0 supporters) — confirmar titularidad antes de publicarlo.

## Z.4 Estado real verificado (comandos → output, 2026-09-08)

- Tag `v0.5.0` = **2026-08-01** (no 09-01); único tag (no existen v0.5.1/v0.6.0/v0.7.0/v1.0.0); workspace `0.5.0`, python hereda.
- PyPI `vantadb-py` 0.5.0 (`>=3.11`, Apache-2.0, Homepage=github) · npm `vantadb` 0.5.0 (homepage `vantadb.dev` muerto, sin `engines`).
- `ness-e/Vantadb`: 28 issues (6 abiertos), 150 PRs (19 abiertos), 2 stars, 1 release v0.5.0 con changelog, 17 labels, templates bug/feature (.md+.yml) + docs (sin `question.md`), 24 workflows CI, `docs/user/tutorials/` (6) + `QUICKSTART.md` + `FAQ.md` (mayúsculas; no existe `api-reference.md`/`faq.md` minúscula), sin `requirements.txt` raíz ni `tests/python/`.
- Discord: 3 miembros (invite viejo `8rP8gxX5k` hoy NULL en API — muerto; canónico `g8nqB3NtXt`).
- `vantadb.vercel.app` vivo con 6 rutas 200; homepage real es marketing ("v0.1 · MVP"), no la estructura mínima del plan.

---

# Decisiones y pendientes SyntropyOS (trasladado de `docs/dev/Backlog.md` el 2026-09-09 — esta es la única fuente; nada de Syntropy vive en el backlog)

Contexto: org `SyntropyOS` configurada (admin `ness-e`, email/website/description/topics, 10 repos públicos). Profile con `README.md` EN + `README_ES.md`, tabla de 10 holones. `ness-e/Vantadb` NO migrado ni modificado (decisión explícita). Respuestas del usuario 2026-09-08/09: dominio → GitHub por ahora; migración → pospuesta; Discord → VantaDB Community tal cual; invite nuevo → otro día; archivo único; sin commit.

- `ORG-01` — Invite permanente nuevo de Discord + Server ID (widget). Invite actual `g8nqB3NtXt` (VantaDB Community, permanente); badge estático sin conteo. RE-PREGUNTAR: código del invite nuevo + Server ID → actualizar READMEs/MANIFESTO/CONTRIBUTING en 1 commit.
- `ORG-02` — Migración `ness-e/Vantadb` → `SyntropyOS/vantadb`. Pospuesta/sin decidir. Si migra: transferir repo, actualizar homepage PyPI/npm, enlaces `.github`, pins. Creados 9 repos `syntropy-{iris,reverb,cardinal,orchestra,sage,execute,plan,reason,meta}` (README ES/EN + LICENSE Apache 2.0). VantaDB NO incluido.
- `ORG-03` — `syntropyai.com` registrado pero EN VENTA (GoDaddy, verificado); `syntropy.com` = Syntropy Technologies LLC; `syntropyos.org` = portal contable ajeno. RE-PREGUNTAR: comprar o alternativo verificado.
- `ORG-04` — Homepage npm `vantadb.dev` MUERTO; sitio real `vantadb.vercel.app` (vivo, verificado). Requiere editar `ness-e` — PROHIBIDO sin orden. RE-PREGUNTAR: autorización o issue al owner.
- `ORG-05` — Discord API: bot válido en el guild; `/guilds/*` → 40333 y `/applications/*/commands` → 403. App ID + secret + public key VERIFICADOS (`verify_key` == key). Secreto solo en memoria, nunca guardado. MCP `@rayenking/discord-mcp` habilitado en `opencode.jsonc` y probado por stdio (handshake + `tools/list` + `list_guilds`/`list_channels` OK; escrituras bloqueadas igual). RE-PREGUNTAR: regenerar token o runbook manual.
- `ORG-06` — Servidor `SYNTROPY` vacío: borrar tras consolidación (cliente Discord, owner).
- `ORG-07` — Check visual `github.com/SyntropyOS` pendiente: header (badges derecha/idiomas izquierda), taglines centrados, tabla 10 holones, ES↔EN, links sin 404, 10 repos con descripciones/topics.
- `ORG-08` — Naming aprobado: scope `@syntropy-ai/*` (org npm creada, 0 paquetes); `@syntropy` descartado (ocupado ~2017). Nombres: Iris, Reverb, Cardinal, Orchestra, Sage, Execute, Plan, Reason, Meta (Comunicación descartada; Orchestra candidato a core). Flag: `Meta` vs Meta/Facebook. NO renombrar `vantadb`/`vantadb-py`.
- `ORG-09` — Primer publish `@syntropy-ai/*` solo con holón real (sin squatting). Depende de ORG-08 + ORG-10.
- `ORG-10` — Auth npm: 2FA `auth-and-writes` + trusted publishing OIDC (bypass-2FA pierde publish directo ~ene-2027). CLI local sin login.
- `ORG-11` — PyPI `syntropy-*` libres al 2026-09-08; asegurar solo finalistas al publicar.
- `ORG-12` — FUNDING + BuyMeACoffee PAUSADO hasta migrar; posts en discussions PAUSADOS hasta migración (+ habilitar Discussions); archivo ÚNICO. Re-preguntar al reactivar ORG-02.
- `ORG-13` — Launch externo/video pausados hasta gate Fase A; influencers pausado (explicación entregada); roles/bots después; fixes `ness-e` solo al migrar; orden de holones tras v1.0.0; plan sin commit. Re-preguntar: nicho, orden (tras v1.0.0), invite/ID.
- `ORG-14` — Dual license y donaciones: evaluado, NO ejecutar (2026-09-09). Apache 2.0 vigente; sin `LICENSE.COMMERCIAL` casera, sin cambios en `ness-e`, FUNDING pausado, sin enlaces de pago. Donación pura futura sin contraprestaciones; tiers solo sugeridos ($3-5/$10-15/$25-50/libre). Reapertura: tras migración o demanda real, nunca antes del gate. Regla: donación ≠ membresía ≠ soporte ≠ consultoría ≠ licencia comercial ≠ SaaS.

