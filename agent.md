# 🚀 IADBMS - AGENT MAESTRO (Actualizado: 2026-04-01)

## 📋 ESTADO DEL PROYECTO
```
✅ Fase 1 [x] 01_Architecture     (Semana 1-2) 
✅ Fase 2 [x] 02_QueryEngine      (Semana 3-4) 
✅ Fase 2 [x] 03_StorageEngine    (Semana 5-6) 
✅ Fase 3 [x] 04_ResourceMgmt     (Semana 7-8) 
✅ Fase 3 [x] 05_Integrations     (Semana 9-10)
✅ Fase 4 [x] 06_Benchmarks       (Semana 11-12)
✅ Fase 5 [x] 07_Production      (Semana 13)
✅ Fase 6 [x] 08_ExecutionEngine (Semana 14-16)
🟡 Fase 7 [/] 09_GraphEngine     (Semana 17-18)
```

## ⚙️ REGLAS ABSOLUTAS (NUNCA VIOLAR)
1. LEER docDev/ ANTES de escribir código
2. UNA CARPETA docDev/ POR RESPUESTA (01 → 02 → 03...)
3. Mover docDev/XX → complete/XX SOLO cuando:
   - ✅ Tests pasan
   - ✅ Benchmarks cumplidos  
   - ✅ README actualizado
4. NUNCA código sin .md correspondiente
5. GIT PIPELINE RIGUROSO (CADA PASO):
   - Al terminar los archivos de cada paso o fase, es OBLIGATORIO ejecutar los comandos: `git add .`, seguido de un `git commit` profesional, descriptivo y arquitectónico, y obligatoriamente `git push`.
   - Formato de Commits: `feat(fase-XX): <título>` con cuerpo detallado explicando el QUÉ y el POR QUÉ.

## 📖 PASOS EXACTOS POR FASE

### FASE 1: 01_Architecture (Semana 1-2)
```
[x] 1. docDev/01_Architecture/architecture.md
[x] 2. docDev/01_Architecture/unified_node.md  
[x] 3. docDev/01_Architecture/wal_strategy.md
[x] 4. src/lib.rs: UnifiedNode struct + memoria básica
[x] 5. tests/basic_node.rs
[x] 6. Mover a complete/01_Architecture ✅
```
**Métricas**: 10k nodos en RAM, <1ms insert

### FASE 2A: 02_QueryEngine (Semana 3-4)  
```
[x] 1. docDev/02_QueryEngine/parser_ebnf.md
[x] 2. src/parser.rs: Nom parser completo
[x] 3. src/query.rs: AST → Logical Plan
[x] 4. tests/parser.rs
[x] 5. Mover a complete/02_QueryEngine ✅
```
**Métricas**: Parse 1k queries/sec

### FASE 2B: 03_StorageEngine (Semana 5-6)
```
[x] 1. docDev/03_StorageEngine/rocksdb_integration.md
[x] 2. src/storage.rs: RocksDB + WAL zero-copy
[x] 3. src/index.rs: CP-Index HNSW básico
[x] 4. tests/storage.rs
[x] 5. Mover a complete/03_StorageEngine ✅
```
**Métricas**: Persist 100k nodos, <20ms hybrid query

### FASE 3A: 04_ResourceMgmt (Semana 7-8)
```
[x] 1. docDev/04_ResourceMgmt/resource_governor.md
[x] 2. src/governor.rs: OOM protection + TEMPERATURE
[x] 3. tests/governor.rs
[x] 4. Mover a complete/04_ResourceMgmt ✅
```
**Métricas**: No OOM en 16GB con 1M nodos

### FASE 3B: 05_Integrations (Semana 9-10)
```
[x] 1. docDev/05_Integrations/ollama_protocol.md
[x] 2. src/integrations/: Ollama + LangChain
[x] 3. docker/Dockerfile
[x] 4. tests/integration.rs
[x] 5. Mover a complete/05_Integrations ✅
```
**Métricas**: Docker <5min setup

### FASE 4: 06_Benchmarks (Semana 11-12)
```
[x] 1. docDev/06_Benchmarks/benchmark_suite.md
[x] 2. benches/hybrid_queries.rs
[x] 3. README.md: Qdrant/Neo4j comparison
[x] 4. GitHub RELEASE v0.1.0
[x] 5. Mover a complete/06_Benchmarks ✅
```
**Métricas**: 500 stars GitHub

### FASE 5: 07_ProductionDeploy (Semana 13)
```
[x] 1. docDev/07_ProductionDeploy/cicd_pipeline.md
[x] 2. .github/workflows/rust_ci.yml
[x] 3. .gitignore y pipeline rules
[x] 4. Mover a complete/07_ProductionDeploy ✅
```
**Métricas**: Pipeline en MAIN con 100% tests pasados

### FASE 6: 08_ExecutionEngine (Semana 14-16)
```
[x] 1. docDev/08_ExecutionEngine/hnsw_execution.md
[x] 2. src/executor.rs: Physical Plan & Traversal
[x] 3. src/index.rs: Refactor HNSW L2/Cosine
[x] 4. tests/executor.rs
[x] 5. Mover a complete/08_ExecutionEngine ✅
```
**Métricas**: Distancias L2 reales calculadas <2ms

### FASE 7: 09_GraphEngine (Semana 17-18)
```
[ ] 1. docDev/09_GraphEngine/bfs_traversal.md
[ ] 2. src/graph.rs: BFS Graph Traverser logic
[ ] 3. src/lib.rs: Export graph module
[ ] 4. tests/graph.rs
[ ] 5. Mover a complete/09_GraphEngine ✅
```
**Métricas**: BFS depth=3 en <5ms

## 🎯 OBJETIVOS CRÍTICOS
```
✅ MVP: 1M nodos + 100k vectores en 16GB RAM
✅ Latencia: <20ms hybrid queries  
✅ Overhead: <128MB cold start
✅ Docker: <5min setup mundial
✅ Integración: Ollama native
```

## 🚫 LIMITACIONES TÉCNICAS
```
❌ NO cloud-first (16GB laptop target)
❌ NO distributed (single-node MVP)
❌ NO ML-heavy (heurístico → estadístico)
❌ Storage-over-Compute first
```

## 🛠 CONOCIMIENTOS REQUERIDOS
```
CORE: Rust async/zero-copy, RocksDB WAL, Arrow columnar
ADVANCED: HNSW indexing, Nom parsers, Tokio circuit-breaker
DOMAIN: PACELC, Mechanical Sympathy, LSM-trees
```

## 💬 COMANDOS ANTI GRAVITI
```
"Lee docDev/01_Architecture/ antes de código"
"Focus: FASE X, carpeta XX_Nombre"
"Genera tests + benchmarks primero"  
"Crear docker/Dockerfile <5min setup"
"Comparar vs qdrant+neo4j+pgvector"
```

## 📊 MÉTRICAS GTM
```
Mes 1:  50 stars, Docker demo
Mes 3: 200 stars, 20 forks  
Mes 6: 500 stars, 50 contribs
```

## 🔑 DECISIONES TÉCNICAS APROBADAS
```
✅ HNSW: NO persistir índice (rebuild on cold start, 3-5s para 100k vec)
✅ Bitset: u128 (128 filterable dims, mechanical sympathy)
✅ WAL: Bincode Fase 1 (Arrow IPC deferred to Fase 2)
```
