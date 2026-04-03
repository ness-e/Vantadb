# 🚀 ConnectomeDB - AGENT MAESTRO (Actualizado: 2026-04-01)

## 📋 ESTADO DEL PROYECTO

*(Fases V0.1.0 y V0.2.0 archivadas. El proyecto inicia nueva etapa arquitectónica V0.3.0 de Gobernanza y Cognición).*

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

### FASE 16: 18_CognitiveArchitecture
- [x] Implementar Trait `CognitiveUnit` y enum `NeuronType` en `src/node.rs`.
- [x] Inyectar campos: `hits`, `last_accessed` y `trust_score` (v0.5 por defecto).
- [x] Crear flags de `PINNED` y logica de metadatos expandibles.
- [x] **Estrategia:** Lazy Write-Back en memoria protegida sin romper bincode para evitar write amplification.

### FASE 17: 19_ShadowKernel & Trust Governance
- [x] Configurar Column Families en `StorageEngine` ('default', 'shadow_kernel', 'tombstones').
- [x] Crear el submódulo `governance.rs` con `AuditableTombstone` y `original_hash`.
- [x] Implementar Borrado Atómico (WriteBatch): Clonar a shadow_kernel, crear lápida y borrar de default en `.delete()`.

### FASE 18: 20_SecurityAxioms
- [x] Implementar reglas `Iron Axioms` para consistencia del DAG en `src/engine.rs`.
- [x] Configurar RocksDB Checkpointing en `src/storage.rs` (Life Insurance).

### Hoja de Ruta de Inteligencia (V0.4.0+)
**FASE 19: Cognitive Sovereignty**
- [x] Implementar modo 'Abogado del Diablo' para detección de sesgos y contradicciones lógicas.
- [x] Algoritmo de resolución automática de conflictos basado en Trust Scores de fuentes.

**FASE 20: Mantenimiento Circadiano (Auditoría de Sueño)**
- [ ] **Fase REM:** Escaneo de IDs en RAM vs Point-Lookup en RocksDB (Reconstrucción probabilística de Bloom Filters si el desvío es > 0.01%).
- [ ] **Fase de Podado (Pruning):** Desalojo masivo de `trust_score` crítico o `hits` nulos a `shadow_kernel`, validando que cada lápida esté íntegra sin interrumpir el I/O del usuario.

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
❌ Preparar para Sharding Semántico en v0.4.0+ (Mantiene MV local, pero abre puerta enterprise)
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
## RECORDATORIOS
```
No ejecutar corgo build ni cargo test ya que en github action se ejecuta automaticamente
```