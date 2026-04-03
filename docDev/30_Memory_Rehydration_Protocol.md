# Fase 30: Protocolo de Rehidratación de Memoria (v0.4.0)

## 1. Problema de Amnesia Inducida
Cuando el Olvido Bayesiano poda excesivamente el Cortex, los datos bajan al `Shadow Archive` perdiendo el `TrustScore` activo. Si un agente o usuario hace una consulta sobre una Neurona de Resumen cuyo `TrustScore` histórico es bajo, se requiere rehidratar el contexto original.

## 2. Mecanismo de Rehidratación
La recuperación desde el Shadow Kernel hacia el Lóbulo Primario requiere cirugía cognitiva:

### Paso A: Detección de Trustscore Insuficiente
Si un agente invoca consulta: `FROM Historical FETCH Summary WHERE id=123`.
El motor nota que `TrustScore < 0.4`.

### Paso B: Solicitud de Arqueología (`OP_REHYDRATE`)
Se desencadena un volcado inverso de RocksDB:
- Busca todos los Nodos con relaciones `belonged_to` hacia la Neurona de Resumen en la Column Family de `Shadow`.
- Restaura temporalmente los atributos, pero bajo un estatus de inmutabilidad efímera (Solo Lectura) dentro de `cortex_ram`.

### Paso C: Síntesis de Corrección
El LLM procesa los datos rehidratados para formar un nuevo Consenso. Si se aprueba por el usuario, el sumario incrementa su `TrustScore` permanentemente.
