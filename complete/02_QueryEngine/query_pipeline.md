# Query Pipeline Architecture
> **Status**: 🟡 In Progress — FASE 2A

## 1. Pipeline Stages

```
User Input (String)
       │
       ▼
1. Parser (Nom) ──▶ Extacts EBNF tokens, validates syntax.
       │            Returns: `Query` (AST)
       ▼
2. Logical Planner ──▶ Maps AST into relational/graph/vector operators.
       │               Resolves aliases, expands inferential rules.
       │               Returns: `LogicalPlan`
       ▼
3. CBO / Optimizer ──▶ Reorders LogicalPlan nodes based on heuristics/SCE.
       │               Embeds bitset filters into Physical Nodes.
       ▼
4. Executor ──▶ Consumes PhysicalPlan, interfaces with `InMemoryEngine`.
                Returns: `QueryResult`
```

## 2. Logical Operators
- `Scan(entity)`
- `Traverse(min_depth, max_depth, label)`
- `VectorSearch(query_vec, min_score)`
- `Filter(field, op, val)`
- `Sort(field, direction)`
- `Project(fields)`

## 3. Implicit Relational Inference (IRI)
Beyond standard explicit operations, the pipeline handles inference:
If the user queries `NAVIGATE u -> p WHERE semántico`, the Logical Planner translates this into a hybrid `Traverse` + `VectorSearch` subplan with implicit join correlations.
