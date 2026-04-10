# Graph Engine: Breadth-First Traversals
> **Status**: 🟡 In Progress — FASE 7

## 1. Graph Semantics
ConnectomeDB allows evaluating topological paths efficiently. Traversals typically evaluate connections expressed in the query AST:
`SIGUE (Usuario)-[CONOCE_A]->(Usuario) HASTA DEPTH 3`

## 2. In-Memory BFS Strategy
Given our `16GB RAM` constraint, recursive Depth-First-Search (DFS) can trigger stack overflows over highly connected dense graph shards.
Instead, we implement an Iterative Breadth-First-Search (BFS).

## 3. Storage Layer Interaction
Nodes store out-bound explicit connections inside `UnifiedNode::graph_edges`.
When the query execution pipeline reaches a `LogicalOperator::Traverse`, it calls the `GraphTraverser`, feeding it the root nodes obtained from the initial `Scan` or `VectorSearch`.
