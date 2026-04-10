# Fase 30: Protocolo de Rehidratación de Memoria (v0.4.0)

## 1. Problema de Amnesia Inducida
Cuando el Olvido Bayesiano poda excesivamente el Cortex, los datos bajan al `Shadow Archive` perdiendo el `TrustScore` activo. Si un agente o usuario hace una consulta sobre una Neurona de Resumen cuyo `TrustScore` histórico es bajo, se requiere rehidratar el contexto original.

## 2. Mecanismo de Rehidratación (Transparencia Selectiva)
La arquitectura equilibra Certidumbre Cognitiva vs Determinismo (P99). En lugar de rehidratar silenciosamente introduciendo latencia I/O sorpresiva, el motor emite una alerta temprana.

### Paso A: Detección de Trustscore Insuficiente y Notificación
Si un agente invoca una consulta: `FROM Historical FETCH Summary WHERE id=123`.
El `Executor` nota que `TrustScore < 0.4`.
- **Comportamiento No-Bloqueante:** El Motor NO bloquea el thread. Inmediatamente retorna un estado `ExecutionStatus::StaleContext(summary_id)` (o flag MCP `rehydration_available: true`).
- Esto avisa al Agente externo que "hay más información profunda", dándole la soberanía de decidir si invocar cirugía cognitiva.

### Paso B: Solicitud de Arqueología (`rehydrate`)
Si el agente decide recuperar los recuerdos, invoca asíncronamente `rehydrate(summary_id)`:
- Escaneo Zero-Copy: Utiliza `DB::get_pinned()` en RocksDB (CF `shadow_kernel`) buscando Nodos con la relación `belonged_to` ligada a la resumen.
- Los nodos descubiertos se copian hacia RAM, marcados con una bandera especial `NodeFlags::REHYDRATED` para trazar su *provenance* arqueológica.
- **Sincronización HNSW:** Se inyectan y sincronizan inmediatamente en el `CPIndex` vectorial en memoria para ser perceptibles en las futuras búsquedas topológicas de sub-grafo.

### Paso C: Barredora de Limpieza Circadiana
Tras el evento de rehidratación y lectura, los nodos efímeros `REHYDRATED` quedan en `cortex_ram`. El `SleepWorker` es el encargado de expurgar/liberar estos nodos periódicamente si ya se satisfizo su propósito de síntesis asíncrona (cuando el orquestador aplica una mutación reparatoria sobre el `trust_score` original).
