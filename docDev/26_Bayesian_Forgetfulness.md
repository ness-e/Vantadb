# Fase 26: Olvido Bayesiano y Compresión Cognitiva

El crecimiento infinito de datos es insostenible en hardware local. ConnectomeDB resuelve esto mediante el **Mantenimiento Circadiano**, transformando el borrado de datos en un proceso de destilación de conocimiento.

## 1. La Poda de Entropía (Entropy Pruning)
Inspirado en el decaimiento de las conexiones sinápticas.
- **Mecanismo**: El `SleepWorker` recorre el Cortex RAM y el Lóbulo Primario durante periodos de baja actividad.
- **Acción**: Por cada ciclo, el valor `hits` (frecuencia de acceso) de las neuronas que no han sido consultadas recientemente se divide: `hits = hits * 0.5`.
- **Efecto**: Los datos "irrelevantes" pierden energía gradualmente hasta alcanzar un umbral crítico de evacuación.

## 2. Compresión Cognitiva (Neural Summarization)
En lugar de simplemente borrar, ConnectomeDB intenta "entender" qué se está perdiendo.
- **Flujo de Consolidación**:
    1. Si un grupo de neuronas relacionadas (ej. 10 mensajes de un mismo chat) cae por debajo del umbral de `hits`.
    2. El motor invoca al LLM local (Ollama) con una instrucción interna de "Summarize Context".
    3. Se genera una nueva **Neurona de Resumen** que captura la esencia semántica del grupo.
    4. Las neuronas originales se mueven al Lóbulo de la Sombra (`shadow_kernel`) como lápidas.
    5. La Neurona de Resumen permanece en el Lóbulo Histórico (`deep_memory`).

---

## 3. Estados de Salud Neuronal

| Estado | Hits / Trust | Acción del SleepWorker |
|:---:|:---:|:---|
| **Lúcido** | Alto | Mantener en Lóbulo Primario (RAM/Hot). |
| **Dudoso** | Medio/Bajo | Migrar de STN a LTN (Disco). |
| **Onírico** | Muy Bajo | Candidato a Compresión Cognitiva. |
| **Difunto** | < 0.1 | Mover al Shadow Archive (Tombstone). |

---

## Axioma de Inmortalidad (Axiom Lock)
Si un nodo posee el flag `PINNED`, el `SleepWorker` tiene prohibido reducir sus `hits` o mutar su ubicación. Este mecanismo se reserva para "Verdades Fundamentales" definidas por el desarrollador o axiomas del sistema.
