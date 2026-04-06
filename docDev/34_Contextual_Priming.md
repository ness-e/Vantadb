# Fase 34: Contextual Priming (Caché Anticipatorio)

> **Estado:** 🔲 PENDIENTE  
> **Versión Objetivo:** v0.5.0  
> **Prerequisito:** Fase 33

---

## Concepto

Pre-cargar proactivamente nodos vecinos de alta probabilidad de consulta en `cortex_ram` antes de que sean explícitamente solicitados, emulando el "priming" neuronal del cerebro humano.

## Objetivo

Reducir la latencia de queries de grafo multi-hop, anticipando los nodos que el usuario probablemente necesitará basándose en patrones de acceso observados.

## Componentes Propuestos

### 1. Trigger de Priming (src/storage.rs)
- En `StorageEngine::get()`: si `node.hits > 20` → `tokio::spawn` pre-carga edges nivel 1 a `cortex_ram`.
- Límite: máx 50 nodos por operación de priming para evitar floods.

### 2. Configuración (Environment Variables)
```bash
CONNECTOME_PRIMING_ENABLED=true    # Activar/desactivar
CONNECTOME_PRIMING_THRESHOLD=20     # Hits mínimos para trigger
CONNECTOME_PRIMING_MAX_NODES=50     # Nodos máx por operación
```

### 3. Integración con HardwareProfile
- `SurvivalProfile`: Priming desactivado (preservar RAM).
- `PerformanceProfile`: Priming activo con límite de 50 nodos.
- `EnterpriseProfile`: Priming agresivo (nivel 2 de profundidad).

### 4. Métricas de Cache Hit
- Nuevo campo atómico: `priming_hits: AtomicU64` en `StorageEngine`.
- Exposible via `/health` endpoint.

## Archivos a Crear/Modificar
- `src/storage.rs` — lógica de priming
- `src/hardware/mod.rs` — configuración por perfil
- `src/server.rs` — métricas en /health
- `tests/contextual_priming.rs`

## Métricas de Aceptación
- [ ] Nodos frecuentes pre-cargan sus vecinos en RAM.
- [ ] Límite de 50 nodos por operación respetado.
- [ ] Desactivado automáticamente en Survival Mode.
- [ ] Test verde: `tests/contextual_priming.rs`.
