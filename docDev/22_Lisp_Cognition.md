# Fase 22: Cognitive IQL & S-Expressions (Homoiconicidad)

## Meta
Dotar a ConnectomeDB de una capa teórica extremadamente avanzada donde el código es igual a los datos. El motor será capaz de razonar funcionalmente, almacenando nodos que no solo representan "hechos pasivos", sino "reglas de negocio dinámicas" (S-Expressions).

## Mecanismo (The `nom` Crate & LISP Dialect)
Un parser secundario vivirá en `src/parser/lisp.rs`.
Si la entrada de IQL detecta que se inicia un macro estructurado mediante `(...)`, el Executor redirige el payload al parseador de S-Expressions.
- Interoperabilidad Datalog-LISP: Funciones de inferencia `(MATCH ?x) -> (WHERE ?y)`.
- **Homoiconicidad Transaccional**: El usuario inserta `(INSERT :neuron {:label "Rule"})` conteniendo S-Expressions.
El "Abogado del Diablo" tratará estas reglas de forma especial.
