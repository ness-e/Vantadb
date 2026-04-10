# Fase 29: Especificación Técnica de NeuLISP (v0.4.0)

## 1. Gramática y Estructura S-Expression
El subsistema de ejecución funcional evoluciona a NeuLISP, integrando operadores probabílisticos y evaluación de certeza nativa. 
El evaluador devuelve un resultado dual: `(Value, TrustScore)`.

### Operador `~` (Similitud Vectorial)
El operador `~` en NeuLISP conecta directamente el parsing LISP con el índice HNSW mediante SIMD.

**Sintaxis:**
```lisp
(~ VECTOR_A VECTOR_B)
```
Ejemplo en el contexto de un trigger:
```lisp
(IF (> (~ ?input_vec ?stored_vec) 0.85) (ACCEPT) (REJECT))
```

### Opcodes Fundamentales de Otimización
El `VM` (`src/eval/vm.rs`) procesa internamente:
- `OP_VEC_SIM`: Ejecuta la invocación `wide::f32x8` para procesar distancias cosenoidales sobre capas 512D.
- `OP_TRUST_CHECK`: Empuja a la pila el puntaje de confianza del nodo en evaluación contextual.
