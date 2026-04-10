# Parser EBNF — Query Language Specification
> **Status**: 🟡 In Progress — FASE 2A (Semana 3-4)

## EBNF Grammar

```ebnf
query ::= "FROM" entity_id ("SIGUE" range edge_label)? target_alias
          ("WHERE" condition_list)?
          ("FETCH" fields)?
          ("RANK BY" order_clause)?
          ("WITH" temperature_clause)? ;

range ::= number ".." number ;
condition_list ::= condition ("AND" condition)* ;
condition ::= rel_pred | vec_sim | "(" condition ")" ;
rel_pred  ::= field op value ;
vec_sim   ::= field "~" string "," "min" "=" number ;
temperature_clause ::= "TEMPERATURE" number ;
```

## TODO
- [ ] Full EBNF with all operators
- [ ] Nom parser skeleton
- [ ] Error recovery strategy
- [ ] Syntax highlighting spec
