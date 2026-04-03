# Fase 28: Optimización de Inferencias (LISP VM & Bloom)

Para lograr el objetivo de sub-milisegundos en consultas híbridas complejas, ConnectomeDB evoluciona su motor de ejecución desde la interpretación directa hacia una arquitectura de Máquina Virtual ligera.

## 1. El Salto a Bytecode (LISP VM)
La interpretación recursiva del AST de las S-Expressions consume ciclos de CPU excesivos.
- **Implementación**:
    1. El motor LISP compila la expresión `.lisp` en una secuencia plana de **Opcodes** (ej. `OP_INSERT_NODE`, `OP_VALIDATE_AXIOM`).
    2. Una VM escrita en Rust seguro ejecuta este bytecode utilizando un dispatch basado en `match` optimizado.
- **Beneficio**: Velocidad $10\times$ superior y control total sobre el consumo de `Cognitive Fuel`.

## 2. Aceleración de Búsquedas (Bloom Co-location)
Integración de los **Filtros de Bloom** en el flujo del `Cortex`.
- **Pre-filtro de Existencia**: Antes de intentar cargar un `UnifiedNode` mediante su ID, el motor comprueba el Bloom Filter del Lóbulo correspondiente.
- **Impacto**: Elimina el 99% de las lecturas falsas en disco durante los escaneos de grafos con relaciones rotas o hacia el Shadow Kernel.

## 3. Optimización SIMD de Tensores (F32x8)
Refinamiento de la función `cosine_similarity` en `src/index.rs`:
- Utiliza la librería `wide` para procesar bloques de 8 floats en una sola instrucción de CPU.
- **Fallback Automático**: Si el hardware no soporta AVX2/AVX-512, el motor conmuta automáticamente a iteradores Rust estándar sin intervención del usuario.

## 4. Model Context Protocol (MCP) Integration
ConnectomeDB expone una interfaz estandarizada para agentes:
- **Endpoint `/mcp/context`**: Permite al agente volcar su contexto actual directamente al Lóbulo Primario.
- **Discovery**: Los agentes pueden "preguntar" al motor qué Axiomas de Hierro están activos para ajustar su generación de texto.
