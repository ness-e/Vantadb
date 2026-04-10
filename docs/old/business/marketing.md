# ConnectomeDB — Marketing & Visibility Strategy

> **Referencia cruzada:** Los KPIs crudos de benchmarks provienen de
> `monetizacion_estrategia.md` §2. Aquí se expanden con formato de campaña.

---

## 1. Estadísticas MÍNIMAS para Homepage (Benchmarks Públicos)

### Hero Numbers (Above the fold)
```
┌─────────────────────────────────────────────────────┐
│  ⚡ 4ms   Hybrid Vector+Graph query (100k vectors)  │
│  📦 15MB  Cold start footprint (vs 2GB Neo4j)       │
│  🔗 1     Single binary. No JVM. No Python.         │
│  🧠 0ms   Auto-embedding overhead (Rust-native)     │
└─────────────────────────────────────────────────────┘
```

### Comparativa Detallada (para sección "Benchmarks")

| Operación | ConnectomeDB | Qdrant | Neo4j | pgvector |
|---|---|---|---|---|
| Vector search (100k, 384d) | **3.8ms** | 5.2ms | N/A | 12ms |
| Graph BFS depth=3 | **1.2ms** | N/A | 4.5ms | N/A |
| Hybrid (vector+graph+filter) | **8ms** | ∞* | ∞* | ∞* |
| Cold start memory | **15MB** | 180MB | 2.1GB | 400MB |
| Insert 10k nodes | **45ms** | 120ms | 800ms | 200ms |
| Docker ready | **<2min** | 3min | 8min | 5min |

> *∞ = Requiere múltiples servicios orquestados (no comparable en una sola pasada)*

### Fuente de datos:
- Ejecutar `cargo bench` contra `benches/hybrid_queries.rs`
- Documentar en `business/benchmarks_public.md` con metodología reproducible
- Publicar como GitHub Actions artifact para transparencia

---

## 2. Nombre del Proyecto

### Análisis del nombre actual "ConnectomeDB"
```
PROs:
  ✅ Descriptivo técnicamente (IA + DBMS)
  ✅ Fácil de buscar (no colisiona con otros proyectos)
  ✅ Suena "enterprise" y serio

CONs:
  ❌ No es memorable
  ❌ Parece sigla de gobierno o norma ISO
  ❌ No evoca velocidad, IA, o innovación
  ❌ Difícil de pronunciar en inglés
```

### Alternativas Propuestas (en orden de recomendación):

| # | Nombre | Tagline | Dominio Check | Sentimiento |
|---|---|---|---|---|
| 1 | **NexusDB** | "Where vectors meet graphs" | nexusdb.dev | Fusión, conexión, hub |
| 2 | **VortexDB** | "The unified AI database" | vortexdb.io | Velocidad, convergencia |
| 3 | **SynapseDB** | "Neural-native database engine" | synapsedb.dev | IA, cerebro, sinapsis |
| 4 | **OmniStore** | "One store to rule them all" | omnistore.dev | Universal, todo-en-uno |
| 5 | **ConnectomeDB** (mantener) | "Rust-native multimodel AI DB" | connectomedb.dev | Technical authority |

### Recomendación:
**Mantener "ConnectomeDB" para código/repositorio** pero adoptar un nombre comercial/marketing como **NexusDB** o **VortexDB** para landing page y comunicación pública. El patrón es común:
- "crates.io" → Rust package registry (nombre técnico diferente al marketing)
- "Turso" → LibSQL fork (marca ≠ proyecto técnico)

---

## 3. Logo

### Directrices de diseño:
```
Estilo:     Geométrico minimalista, líneas limpias
Paleta:     Naranja Rust (#CE422B) + Azul Oscuro (#0D1117) + Blanco
Forma:      Hexágono (estabilidad) con 3 nodos internos conectados (tri-modelo)
Tipografía: Inter Bold o JetBrains Mono (developer-friendly)
Variantes:  Logo completo, Icono solo, Monocromo, Favicon 16px
```

### 3 Conceptos a generar en `business/logo_concepts/`:
1. **Concept A — "Trinity Node":** Tres nodos (Vector/Grafo/Relacional) conectados dentro de un hexágono con gradiente naranja→azul
2. **Concept B — "Neural Mesh":** Red neuronal estilizada formando las letras "IA" con partículas vectoriales
3. **Concept C — "Rust Crab + DB":** Ferris el cangrejo sosteniendo un cilindro de base de datos con nodos de grafo orbitando

---

## 4. Social Launch Strategy

### Plataformas de prioridad:

| Plataforma | Audiencia | Contenido | Timing |
|---|---|---|---|
| **HackerNews** | Ingenieros senior, CTOs | "Show HN: ConnectomeDB — 3-in-1 AI DB in Rust (Vector+Graph+SQL)" | Launch Day (martes 10am EST) |
| **Reddit /r/rust** | Comunidad Rust | Technical deep-dive: "How we built HNSW from scratch in 118 lines" | Día +1 |
| **Reddit /r/MachineLearning** | ML engineers | "Native RAG without Python: auto-embedding in a Rust database" | Día +2 |
| **Twitter/X** | Dev influencers | Thread: "We replaced 3 databases with 1 Rust binary" 🧵 | Launch Day |
| **LinkedIn** | Enterprise decision makers | Article: "Why your AI stack needs a unified database" | Día +3 |
| **Dev.to / Hashnode** | Early-career devs | Tutorial: "Build a RAG agent with ConnectomeDB + Ollama in 5 min" | Semana 2 |
| **YouTube** | Broad dev audience | 3-min demo video | Launch Day |
| **Discord** | Community building | Servidor propio ConnectomeDB | Pre-launch |

### HackerNews Launch Playbook:
```
TÍTULO: "Show HN: ConnectomeDB – Rust database that unifies vectors, graphs, and SQL for local AI"

REGLAS:
1. Postear MARTES o MIÉRCOLES a las 10am EST (peak HN traffic)
2. NO pedir upvotes (violación de reglas HN)
3. Top comment debe ser del autor explicando el "why"
4. Responder CADA comentario en las primeras 4 horas
5. Tener README impecable con GIF demo ANTES de postear
6. Docker one-liner listo: docker run connectomedb/connectomedb
```

---

## 5. Demo Video (30 segundos)

### Storyboard:
```
[0-5s]   Logo animado + "ConnectomeDB: One database for AI"
[5-10s]  Terminal: docker run connectomedb → server starts in 1.2s
[10-15s] CLI: INSERT NODE con datos + auto-embedding happening
[15-20s] CLI: Hybrid query (vector + graph traversal) → result in 4ms
[20-25s] Browser: Ollama chat usando contexto de ConnectomeDB como RAG
[25-30s] Benchmarks table overlay + "Star us on GitHub" + URL
```

### Herramienta de grabación:
- **asciinema** para terminal recordings
- **OBS** para compositing final
- **Canva** para overlays y transiciones

---

## 6. GitHub README Restructuración

### Orden óptimo de secciones:
```markdown
1. # Hero: Nombre + One-liner + Hero image/GIF
2. ## ⚡ 30-Second Demo (GIF de CLI en acción)
3. ## 📊 Benchmarks (tabla ConnectomeDB vs competencia)
4. ## 🤔 Why ConnectomeDB? (3 bullet points con iconos)
5. ## 🚀 Quick Start (docker run + 3 comandos)
6. ## 💡 IQL Examples (5 queries progresivas)
7. ## 🏗️ Architecture (diagrama simplificado)
8. ## 📦 Installation (cargo, docker, python pip)
9. ## 🤖 AI Integration (Ollama + LangChain)
10. ## 📖 Documentation (link a docs site)
11. ## 🗺️ Roadmap (link a GitHub Projects)
12. ## 💬 Community (Discord + Contributing)
13. ## ⚖️ License (Apache 2.0)
```

### Regla de oro:
> Un dev que llega al README debe poder copiar un comando y tener ConnectomeDB
> corriendo en menos de 60 segundos. Si tarda más, el README falló.
