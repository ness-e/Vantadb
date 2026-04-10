# Fase 36: Inmunología Lógica & Slashing Epistémico

## Objetivo
Implementar defensas de "Inmunología Lógica" en **ConnectomeDB** (Arquitectura NexusDB) para transformar el motor de una postura puramente reactiva a un sistema defensivo capaz de repeler proactivamente ataques de entropía (spam semántico) y gaslighting algorítmico, y penalizar definitivamente (Slashing) orígenes hostiles o alucinantes, protegiendo así el Axiom Set L1.

## Modificaciones Estructurales Principales

### 1. `OriginCollisionTracker` y Métrica de Fricción Axiomática
*   **Problema:** Anteriormente, el `DevilsAdvocate` era un objeto *stateless*. Un atacante malicioso (con el mismo `_owner_role`) podía inundar el sistema con vectores adyacentes a un axioma y eventualmente generar un sobrepaso en el `TrustScore`, forzando la degradación o alteración del conocimiento.
*   **Implementación:** Se introdujo `OriginCollisionTracker` en `src/governance/mod.rs` que rastrea colisiones origen a nivel semántico y computa la métrica `F_ax` utilizando una fórmula de fricción logarítmica:
    `F_ax = Σ [ log2(1 + c_i) × T_i ]`
    Donde `c_i` es el conteo de colisiones desde el origen `i`, y `T_i` es su confianza.
*   **Impacto:** Los ataques provenientes de unos pocos actores tienen su impacto "aplanado" logarítmicamente ("Barrera Hematoencefálica Semántica"). Romper un axioma consolidado (alta valencia) requiere ahora la concurrencia de una **amplia base de agentes diversos y confiables**.
*   **Integration:** El `StorageEngine` ahora cuenta con una instancia global de `DevilsAdvocate` inyectada para usar este tracker compartido a través de las operaciones del motor.

### 2. Slashing Epistémico en Mantenimiento Tímico (`SleepWorker`)
*   **Problema:** Agentes que repetidamente enviaban datos identificados como falsos debían ser penalizados de forma que perdieran completamente su privilegio en la red.
*   **Implementación:** Se amplió la fase REM en `src/governance/sleep_worker.rs`. Si se detecta un nodo etiquetado como "hallucination", se obtiene su `_owner_role`. A continuación, el tracker compartido en el `StorageEngine` lo invoca en `slash_origin(role)`, lo que fuerza el `TrustScore` interno a `0.0`.
*   **Impacto:** Los agentes detectados inyectando conocimiento anómalo son sometidos a una cuarentena instantánea, invalidando todas sus contribuciones pasadas en métricas de confianza y bloqueando futuros impactos.

### 3. Filtro L1: `ThalamicGate` (Bloqueo en Tiempo Constante)
*   **Problema:** Una vez slasheados, los agentes hostiles aún podían iniciar operaciones pesadas (verificación de vectores HNSW, parseo de LISP), consumiendo valiosos ciclos del CPU en escenarios de supervivencia (Survival Mode).
*   **Implementación:** Se expandió el filtro de Boom ("Bloom Filter array") en `src/governance/thalamic_gate.rs` para registrar `_owner_role` mediante hashing (`record_role_ban`, `is_role_banned`). Al ejecutarse el `slash_origin`, el ban se propaga al filtro de Bloom perma-bloqueando al actor directamente en la puerta.
*   **Impacto:** Deslizadores / Actores penalizados en capa 1 (L1) son rechazados instantáneamente (`return Err()`) en `Executor` (insertar/actualizar) **antes** de que comience a ejecutarse la lógica vector-semántica. Cero gasto de recursos (O(1)).

## Decisiones Críticas
1.  **Aislamiento de Riesgo en la Serialización:**
    A pesar de que requerimos identificar la procedencia en Rust, evitamos tocar la estructura raíz del parser HCS/LISP (`UnifiedNode`) ni crear nuevos campos estáticos en `node.rs`. Todo referenciamiento a la procedencia se realiza usando el hashmap `relational` con la clave pre-reservada `_owner_role`. Esta decisión elimina un riesgo inaceptable de migraciones de `rocksdb_backend` de cara a la v1.0.
2.  **Castigo Irreversible L1 vs L2 (Bloom Array):**
    No se incluyó un *Counting Bloom Filter*. Las "apelaciones" no son una prioridad estricta para la persistencia; una vez que un `_owner_role` es baneado por inyectar vectores de alucinación, dicho ban persiste en RAM de forma inexpugnable (al menos para el motor actual) en modo `Survival`.

## Pruebas De Integración (`tests/immunology.rs`)
Se integró satisfactoriamente una suite de la `immunology` con 5 pruebas que verifican matemáticamente toda la cadena algorítmica:
*   `test_single_origin_logarithmic_friction`: Prueba que el logaritmo limita a un atacante masivo solitario frente a un Axioma, bloqueando su ataque con éxito.
*   `test_diverse_origins_breach_axiom`: Garantiza que, si N fuentes distribuidas fiables difieren iterativamente, el axioma _sí puede_ fracturarse en Superposición (la resistencia no bloquea el progreso del modelo del mundo).
*   `test_slashing_bans_agent`: Verifica el enlace total desde el tracker al baneo definitivo por ThalamicGate al detectar alucinación REM.
*   `test_thalamic_role_ban`: Comprueba que el hashing de la string es consistente dentro de las máscaras de Bloom y que no se interceptan entre nodos arbitrarios.
*   `test_friction_formula_properties`: Audiciones exhaustivas a propiedades individuales y el cálculo EMA (Media Móvil Exponencial) de Trust Score originario. `cargo test --test immunology` ejecuta todas las anteriores sin errores ni bloqueos.

## Pasos Siguientes
Con Inmunología Lógica activa, el motor central es defensivamente estable para la ejecución remota y tolerante a spam (Byzantine-Resistant adaptado). Pasamos a revisar el `strategic_master_plan.md` (posible refactor de APIs remotos/RPC o validaciones End-to-End).
