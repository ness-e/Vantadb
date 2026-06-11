# Future Modifications Roadmap - VantaDB v0.2.x+

**Date:** June 2026  
**Version:** 1.0  
**Status:** Proposed  
**Scope:** High-ROI data type enhancements and graph execution primitives

---

## 📋 Executive Summary

This document outlines the **recommended future modifications** to VantaDB that provide high strategic value while maintaining architectural coherence. Based on analysis of effort vs. strategic benefit, only **3 modifications** are recommended for implementation in v0.2.x.

**Key Principle:** VantaDB maintains its embedded-first, local-first positioning as "SQLite for AI Agents" by avoiding scope creep into multi-model databases, application-level patterns, or ML infrastructure.

**Recommended Modifications:**
1. **Native Date/Time Types** - Resolves temporal tracking needs without string hacks
2. **Flat Array/List Types** - Enables common metadata patterns without JSON complexity
3. **Basic DAG Execution Primitives** - Completes existing graph traversal for local orchestration

**Explicitly Out of Scope:** JSON nesting, geospatial types, large blobs, fine-tuning, and application-level patterns (GraphRAG, Agentic RAG, etc.) remain delegated to specialized systems.

---

## 🎯 Modification 1: Native Date/Time Types

**Priority:** 🟡 Medium-High  
**Effort Estimate:** 2-3 weeks  
**Strategic Value:** 🔴 High  
**Risk:** 🟢 Low  

### Motivation

**Current Problem:**
- AI agents require temporal tracking: conversation timestamps, event expiration, session windows
- Users currently implement temporal data as string hacks (`"2024-06-10T14:30:00Z"`) or Unix epoch integers
- Inconsistent formats across user bases cause interoperability issues
- No native time-based queries or indexing capabilities

**User Pain Points:**
```python
# Current hack - inconsistent across users
metadata = {"created_at": "2024-06-10T14:30:00Z"}  # String parsing required
metadata = {"expiry": 1718044200}  # Unix epoch, no timezone context
metadata = {"duration": 3600}  # Seconds, but no semantic meaning
```

### Benefits

**Technical Benefits:**
- Type-safe temporal operations (comparisons, arithmetic, ranges)
- Native temporal indexing for efficient time-range queries
- Deterministic serialization/deserialization (no string parsing ambiguity)
- Timezone-aware operations (UTC normalization)

**Strategic Benefits:**
- Removes common user complaint about temporal data handling
- Enables basic time-series patterns without full time-series database
- Aligns with enterprise expectations for data management systems
- Low technical debt investment for high user value

### Technical Implementation

#### Phase 1: Core Type Extensions (1 week)

**File:** `src/node.rs`

**Changes to FieldValue enum:**
```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FieldValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    
    // NEW: Temporal types
    DateTime(DateTime<Utc>),      // RFC 3339 timestamp with timezone
    Duration(Duration),           // Span of time for arithmetic
    Date(NaiveDate),              // Calendar date (YYYY-MM-DD)
    Time(NaiveTime),              // Clock time (HH:MM:SS)
}
```

**Dependencies to add:**
```toml
[dependencies]
chrono = { version = "0.4", features = ["serde"] }
```

**Tasks:**
1. [ ] Add chrono dependency to Cargo.toml
2. [ ] Extend FieldValue enum with 4 temporal variants
3. [ ] Implement `as_datetime()`, `as_duration()`, `as_date()`, `as_time()` helpers
4. [ ] Update Bincode serialization compatibility tests
5. [ ] Add temporal type validation in metadata insertion

#### Phase 2: Indexing Support (3-5 days)

**File:** `src/storage.rs`

**Changes:**
- Extend derived payload index encoding to support temporal types
- Implement temporal range query optimization in query planner
- Add B-tree ordering for chronological scans

**Index Key Encoding:**
```rust
fn encoded_temporal_value(value: &FieldValue) -> Vec<u8> {
    match value {
        FieldValue::DateTime(dt) => {
            let mut encoded = b"dt:".to_vec();
            encoded.extend_from_slice(&dt.timestamp().to_be_bytes());
            encoded
        }
        FieldValue::Duration(dur) => {
            let mut encoded = b"du:".to_vec();
            encoded.extend_from_slice(&dur.num_seconds().to_be_bytes());
            encoded
        }
        // ... other temporal types
        _ => unreachable!()
    }
}
```

**Tasks:**
1. [ ] Implement temporal index key encoding
2. [ ] Add temporal range query predicates to query planner
3. [ ] Optimize chronological scan operations in storage layer
4. [ ] Add temporal index rebuilding support

#### Phase 3: SDK Surface (2-3 days)

**File:** `src/sdk.rs`

**Changes to VantaValue enum:**
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VantaValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    
    // NEW: Temporal types (ISO 8601 string representation for JSON compatibility)
    DateTime(String),  // ISO 8601: "2024-06-10T14:30:00Z"
    Duration(String),  // ISO 8601 duration: "PT1H30M"
    Date(String),      // ISO 8601 date: "2024-06-10"
    Time(String),      // ISO 8601 time: "14:30:00"
}
```

**Python Binding Updates:**
```python
# vantadb-python/src/lib.rs
datetime.datetime -> VantaValue::DateTime
datetime.timedelta -> VantaValue::Duration
datetime.date -> VantaValue::Date
datetime.time -> VantaValue::Time
```

**Tasks:**
1. [ ] Extend VantaValue enum with temporal variants
2. [ ] Implement Python datetime → VantaValue conversion
3. [ ] Add temporal type constructors to Python SDK
4. [ ] Update CLI to support temporal input formats
5. [ ] Add temporal query examples to documentation

#### Phase 4: Testing & Validation (2-3 days)

**File:** `tests/temporal_types.rs` (new)

**Test Coverage:**
- Temporal type serialization/deserialization roundtrip
- Temporal index creation and querying
- Time range query correctness (edge cases: timezone transitions, leap seconds)
- Schema evolution compatibility (adding temporal types to existing databases)
- Performance benchmarks (temporal vs. string-based indexing)

**Tasks:**
1. [ ] Create comprehensive temporal type test suite
2. [ ] Add temporal certification corpus (similar to SIFT-1M for vectors)
3. [ ] Validate backward compatibility with existing databases
4. [ ] Performance benchmark: temporal index vs. string hack
5. [ ] Document temporal query patterns in user guide

### Acceptance Criteria

- [ ] All temporal types serialize correctly via Bincode without breaking existing data
- [ ] Temporal range queries execute in <5ms on 100K record dataset
- [ ] Python SDK accepts native datetime objects without conversion
- [ ] CLI supports ISO 8601 input formats for all temporal types
- [ ] Backward compatibility: existing databases without temporal types continue to work
- [ ] Documentation includes temporal query patterns and best practices
- [ ] Test suite achieves 100% coverage of temporal code paths

### Migration Strategy

**No data migration required** - temporal types are additive to existing schema:
- Existing databases continue to use string/int temporal hacks
- New records can use native temporal types
- Gradual migration path: users convert records incrementally
- No breaking changes to existing APIs

---

## 🎯 Modification 2: Flat Array/List Types

**Priority:** 🟡 Medium  
**Effort Estimate:** 1-2 weeks  
**Strategic Value:** 🟠 Medium  
**Risk:** 🟢 Low  

### Motivation

**Current Problem:**
- Common metadata patterns require multiple values: tags, categories, related IDs, enum-like sets
- Users currently work around with string concatenation hacks (`"ai,database,rust"`)
- No native containment queries (e.g., "find records where tags contain 'ai'")
- No array index support for efficient multi-value filtering

**User Pain Points:**
```python
# Current hack - no validation, no efficient querying
metadata = {
    "tags": "ai,database,rust",  # String parsing required
    "categories": "note|memo|log",  # Inconsistent delimiters
    "related_ids": "123,456,789"  # No type safety
}
```

**Desired Pattern:**
```python
# Desired - type-safe, queryable arrays
metadata = {
    "tags": ["ai", "database", "rust"],
    "categories": ["note", "memo"],
    "related_ids": [123, 456, 789]
}

# Efficient containment query
results = db.search_memory(
    "agent/main",
    metadata_filter={"tags": "ai"},  # Find records where tags contains "ai"
    top_k=10
)
```

### Benefits

**Technical Benefits:**
- Type-safe multi-value metadata without string parsing hacks
- Native containment queries for array fields
- Array indexing for efficient multi-value filtering
- Validation of array element types (all elements must be same scalar type)

**Strategic Benefits:**
- Resolves top user request for multi-value metadata
- Enables common patterns (tags, categories, related entities)
- Low implementation complexity for high user value
- No architectural disruption (flat arrays, not nested JSON)

### Technical Implementation

#### Phase 1: Core Type Extensions (3-4 days)

**File:** `src/node.rs`

**Changes to FieldValue enum:**
```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FieldValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    
    // NEW: Flat array types (homogeneous scalar arrays)
    StringArray(Vec<String>),
    IntArray(Vec<i64>),
    FloatArray(Vec<f64>),
    BoolArray(Vec<bool>),
}
```

**Design Decision:** Only flat arrays of homogeneous scalar types. No nested arrays, no mixed-type arrays.

**Validation Logic:**
```rust
impl FieldValue {
    pub fn validate_array_homogeneity(&self) -> Result<(), VantaError> {
        match self {
            FieldValue::StringArray(_) | FieldValue::IntArray(_) | 
            FieldValue::FloatArray(_) | FieldValue::BoolArray(_) => Ok(()),
            _ => Err(VantaError::InvalidType)
        }
    }
}
```

**Tasks:**
1. [ ] Extend FieldValue enum with 4 array variants
2. [ ] Implement array validation logic (homogeneous elements)
3. [ ] Add array accessor helpers (`as_string_array()`, etc.)
4. [ ] Update Bincode serialization compatibility tests
5. [ ] Add array size limits (max 1000 elements per array to prevent abuse)

#### Phase 2: Indexing and Containment Queries (4-5 days)

**File:** `src/storage.rs`

**Index Key Encoding for Containment:**
```rust
// For array field ["ai", "database", "rust"], create 3 index entries:
// metadata:tags:ai -> record_key
// metadata:tags:database -> record_key
// metadata:tags:rust -> record_key

fn array_containment_index_entries(
    namespace: &str, 
    field: &str, 
    array: &FieldValue
) -> Vec<(Vec<u8>, String)> {
    match array {
        FieldValue::StringArray(arr) => {
            arr.iter().map(|elem| {
                let index_key = format!("{}:{}:{}", namespace, field, elem);
                (index_key.into_bytes(), elem.clone())
            }).collect()
        }
        // ... similar for other array types
        _ => vec![]
    }
}
```

**Query Planner Updates:**
- Detect containment predicates: `field = "value"` where field is array type
- Execute index scan on containment index instead of full scan
- Support multiple containment predicates with AND/OR logic

**Tasks:**
1. [ ] Implement array containment index generation
2. [ ] Add containment query detection in query planner
3. [ ] Optimize multi-containment query execution
4. [ ] Add array index rebuilding support
5. [ ] Implement array size validation during insertion

#### Phase 3: SDK Surface (2-3 days)

**File:** `src/sdk.rs`

**Changes to VantaValue enum:**
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VantaValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    
    // NEW: Array types
    StringArray(Vec<String>),
    IntArray(Vec<i64>),
    FloatArray(Vec<f64>),
    BoolArray(Vec<bool>),
}
```

**Python Binding Updates:**
```python
# Automatic conversion from Python lists to VantaValue arrays
metadata = {
    "tags": ["ai", "database", "rust"],  # -> VantaValue::StringArray
    "scores": [0.9, 0.8, 0.7],         # -> VantaValue::FloatArray
}
```

**CLI Support:**
```bash
# Array input via JSON-like syntax
vanta-cli put \
  --db ./data \
  --namespace agent/main \
  --key mem-1 \
  --metadata '{"tags": ["ai", "database"]}'
```

**Tasks:**
1. [ ] Extend VantaValue enum with array variants
2. [ ] Implement Python list → VantaValue array conversion
3. [ ] Add CLI JSON parsing for array input
4. [ ] Update documentation with array query examples
5. [ ] Add array type validation error messages

#### Phase 4: Testing & Validation (2-3 days)

**File:** `tests/array_types.rs` (new)

**Test Coverage:**
- Array type serialization/deserialization roundtrip
- Array index creation and containment queries
- Array homogeneity validation (mixed-type arrays rejected)
- Array size limit enforcement
- Schema evolution compatibility (adding arrays to existing databases)
- Performance benchmarks (array index vs. string hack)

**Tasks:**
1. [ ] Create comprehensive array type test suite
2. [ ] Add array query certification corpus
3. [ ] Validate backward compatibility with existing databases
4. [ ] Performance benchmark: array index vs. comma-separated string
5. [ ] Document array query patterns in user guide

### Acceptance Criteria

- [ ] All array types serialize correctly via Bincode without breaking existing data
- [ ] Containment queries execute in <5ms on 100K record dataset
- [ ] Python SDK accepts native Python lists without conversion
- [ ] CLI supports JSON array input formats
- [ ] Array homogeneity validation rejects mixed-type arrays
- [ ] Array size limits enforced (max 1000 elements)
- [ ] Backward compatibility: existing databases without array types continue to work
- [ ] Documentation includes array query patterns and best practices
- [ ] Test suite achieves 100% coverage of array code paths

### Migration Strategy

**No data migration required** - array types are additive to existing schema:
- Existing databases continue to use string hacks for multi-value data
- New records can use native array types
- Gradual migration path: users convert records incrementally
- No breaking changes to existing APIs

---

## 🎯 Modification 3: Basic DAG Execution Primitives

**Priority:** 🟡 Medium  
**Effort Estimate:** 3-4 weeks  
**Strategic Value:** 🟠 Medium  
**Risk:** 🟡 Medium  

### Motivation

**Current State:**
- VantaDB already implements BFS graph traversal in `src/graph.rs` <ref_file file="c:/Users/Eros/VantaDB Proyect/VantaDB/src/graph.rs" />
- Users can store directed edges with weights
- Missing: cycle detection, topological sorting, DAG validation

**User Use Cases:**
- Local agent pipeline orchestration (task dependencies)
- Build system dependency tracking
- Workflow execution ordering
- Resource dependency graphs

**Current Limitation:**
```python
# Current: Can traverse, but cannot validate DAG structure or get execution order
edges = [
    Edge(target=2, label="depends_on"),  # Node 1 -> Node 2
    Edge(target=3, label="depends_on"),  # Node 1 -> Node 3
    Edge(target=4, label="depends_on"),  # Node 2 -> Node 4
]

# Can BFS traverse, but:
# - No cycle detection (could create 1 -> 2 -> 3 -> 1)
# - No topological sort (execution order)
# - No DAG validation
```

### Benefits

**Technical Benefits:**
- Completes existing graph traversal implementation (80% already done)
- Enables local pipeline orchestration without external DAG systems
- Deterministic execution order for dependent tasks
- Cycle detection prevents infinite loops in agent workflows

**Strategic Benefits:**
- Low incremental effort (building on existing BFS implementation)
- Enables local agent workflows without external orchestrators (Airflow, Temporal)
- Maintains embedded-first positioning (no external services required)
- Competitive advantage over pure vector databases (no graph capabilities)

### Technical Implementation

#### Phase 1: Cycle Detection (5-7 days)

**File:** `src/graph.rs`

**Algorithm:** DFS-based cycle detection with coloring
```rust
impl<'a> GraphTraverser<'a> {
    /// Detect cycles in the graph starting from given roots
    /// Returns true if a cycle exists
    pub fn detect_cycles(&self, roots: &[u64]) -> Result<bool, VantaError> {
        enum VisitState { Unvisited, Visiting, Visited }
        let mut state = HashMap::new();
        let mut has_cycle = false;
        
        for &root in roots {
            if self.dfs_cycle_detect(root, &mut state, &mut has_cycle)? {
                return Ok(true);
            }
        }
        Ok(has_cycle)
    }
    
    fn dfs_cycle_detect(
        &self, 
        node_id: u64, 
        state: &mut HashMap<u64, VisitState>, 
        has_cycle: &mut bool
    ) -> Result<bool, VantaError> {
        use VisitState::*;
        
        match state.get(&node_id) {
            Some(Visiting) => return Ok(true),  // Back edge = cycle
            Some(Visited) => return Ok(false),
            Some(Unvisited) => (),
            None => state.insert(node_id, Unvisited),
        }
        
        state.insert(node_id, Visiting);
        
        if let Ok(Some(node)) = self.storage.get(node_id) {
            for edge in &node.edges {
                if self.dfs_cycle_detect(edge.target, state, has_cycle)? {
                    return Ok(true);
                }
            }
        }
        
        state.insert(node_id, Visited);
        Ok(false)
    }
}
```

**Tasks:**
1. [ ] Implement DFS-based cycle detection algorithm
2. [ ] Add cycle detection unit tests (acyclic vs. cyclic graphs)
3. [ ] Add cycle detection to public GraphTraverser API
4. [ ] Document cycle detection algorithm in architecture docs
5. [ ] Add cycle detection performance benchmarks

#### Phase 2: Topological Sort (5-7 days)

**File:** `src/graph.rs`

**Algorithm:** Kahn's algorithm for topological sorting
```rust
impl<'a> GraphTraverser<'a> {
    /// Returns nodes in topological order (if DAG)
    /// Returns error if graph contains cycles
    pub fn topological_sort(&self, roots: &[u64]) -> Result<Vec<u64>, VantaError> {
        // 1. Build adjacency list and in-degree count
        let mut adj = HashMap::new();
        let mut in_degree = HashMap::new();
        
        // Build graph from stored edges
        for &root in roots {
            self.build_graph_structure(root, &mut adj, &mut in_degree)?;
        }
        
        // 2. Kahn's algorithm
        let mut queue = VecDeque::new();
        let mut result = Vec::new();
        
        // Start with nodes of in-degree 0
        for (&node, °ree) in &in_degree {
            if degree == 0 {
                queue.push_back(node);
            }
        }
        
        while let Some(node) = queue.pop_front() {
            result.push(node);
            
            // Reduce in-degree of neighbors
            if let Some(neighbors) = adj.get(&node) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(*neighbor);
                        }
                    }
                }
            }
        }
        
        // If not all nodes processed, cycle exists
        if result.len() != adj.len() {
            return Err(VantaError::CycleDetected);
        }
        
        Ok(result)
    }
}
```

**Tasks:**
1. [ ] Implement Kahn's algorithm for topological sorting
2. [ ] Add topological sort unit tests (various DAG structures)
3. [ ] Validate topological sort produces valid execution order
4. [ ] Add topological sort to public GraphTraverser API
5. [ ] Document topological sort algorithm in architecture docs

#### Phase 3: DAG Validation API (2-3 days)

**File:** `src/graph.rs`

**Combined API for DAG validation and execution order:**
```rust
impl<'a> GraphTraverser<'a> {
    /// Validates that the subgraph is a DAG and returns execution order
    pub fn validate_dag_and_get_execution_order(
        &self, 
        roots: &[u64]
    ) -> Result<DagExecutionPlan, VantaError> {
        // 1. Check for cycles
        if self.detect_cycles(roots)? {
            return Err(VantaError::CycleDetected);
        }
        
        // 2. Compute topological order
        let execution_order = self.topological_sort(roots)?;
        
        // 3. Build execution plan with levels (parallelizable groups)
        let levels = self.compute_execution_levels(&execution_order)?;
        
        Ok(DagExecutionPlan {
            is_dag: true,
            execution_order,
            levels,
            total_nodes: execution_order.len(),
        })
    }
}

pub struct DagExecutionPlan {
    pub is_dag: bool,
    pub execution_order: Vec<u64>,
    pub levels: Vec<Vec<u64>>,  // Nodes that can execute in parallel
    pub total_nodes: usize,
}
```

**Tasks:**
1. [ ] Implement combined DAG validation API
2. [ ] Add execution level computation (parallelizable groups)
3. [ ] Add DAG validation unit tests
4. [ ] Add execution plan serialization for SDK boundary
5. [ ] Document DAG validation use cases

#### Phase 4: SDK Surface and Integration (3-4 days)

**File:** `src/sdk.rs`

**New SDK Types:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VantaDagValidationResult {
    pub is_dag: bool,
    pub execution_order: Vec<u64>,
    pub parallel_levels: Vec<Vec<u64>>,
    pub cycle_nodes: Option<Vec<u64>>,  // If not DAG, where is the cycle?
}

impl VantaEmbedded {
    pub fn validate_dag(
        &self, 
        namespace: &str, 
        root_keys: &[String]
    ) -> Result<VantaDagValidationResult, VantaError> {
        // Convert keys to node IDs, run DAG validation
        // Return structured result
    }
}
```

**Python Binding:**
```python
class VantaDB:
    def validate_dag(self, namespace: str, root_keys: List[str]) -> Dict:
        """
        Validate that the subgraph rooted at given keys is a DAG.
        Returns execution order if valid, cycle information if invalid.
        """
        result = self._rust_validate_dag(namespace, root_keys)
        return {
            "is_dag": result.is_dag,
            "execution_order": result.execution_order,
            "parallel_levels": result.parallel_levels,
            "cycle_nodes": result.cycle_nodes
        }
```

**CLI Support:**
```bash
# Validate DAG structure
vanta-cli validate-dag \
  --db ./data \
  --namespace pipelines/build \
  --roots task-1,task-2

# Output execution order
# DAG: VALID
# Execution Order: [task-1, task-2, task-3, task-4]
# Parallel Levels:
#   Level 0: [task-1, task-2]
#   Level 1: [task-3]
#   Level 2: [task-4]
```

**Tasks:**
1. [ ] Add VantaDagValidationResult to SDK types
2. [ ] Implement validate_dag() method in VantaEmbedded
3. [ ] Add Python binding for DAG validation
4. [ ] Add CLI validate-dag command
5. [ ] Update documentation with DAG validation examples
6. [ ] Add case study: local agent pipeline orchestration

#### Phase 5: Testing & Validation (2-3 days)

**File:** `tests/dag_validation.rs` (new)

**Test Coverage:**
- Cycle detection on various graph structures (acyclic, single cycle, multiple cycles)
- Topological sort correctness (validates edge dependencies)
- DAG validation with complex dependency graphs
- Execution level computation (parallelizable groups)
- Performance benchmarks (large DAGs: 10K+ nodes)

**Test Corpora:**
```rust
// Acyclic DAG (should pass)
const DAG_VALID: &[Edge] = &[
    Edge { target: 2, label: "depends_on" },
    Edge { target: 3, label: "depends_on" },
    Edge { target: 4, label: "depends_on" },
];

// Cyclic graph (should fail)
const DAG_CYCLIC: &[Edge] = &[
    Edge { target: 2, label: "depends_on" },
    Edge { target: 3, label: "depends_on" },
    Edge { target: 1, label: "depends_on" },  // Back edge creates cycle
];
```

**Tasks:**
1. [ ] Create comprehensive DAG validation test suite
2. [ ] Add DAG certification corpora (valid/invalid DAG structures)
3. [ ] Performance benchmark: cycle detection on 10K node graphs
4. [ ] Performance benchmark: topological sort on complex DAGs
5. [ ] Document DAG validation performance characteristics

### Acceptance Criteria

- [ ] Cycle detection correctly identifies cyclic vs. acyclic graphs
- [ ] Topological sort produces valid execution order (all dependencies satisfied)
- [ ] DAG validation completes in <100ms on 10K node DAGs
- [ ] Python SDK exposes DAG validation API
- [ ] CLI includes validate-dag command with clear output
- [ ] Execution level computation correctly identifies parallelizable groups
- [ ] Documentation includes DAG validation patterns and use cases
- [ ] Test suite achieves 100% coverage of DAG validation code paths
- [ ] Backward compatibility: existing graph operations continue to work

### Migration Strategy

**No breaking changes** - DAG validation is additive functionality:
- Existing graph operations (BFS traversal) continue to work unchanged
- DAG validation is opt-in (new API method, not automatic)
- No schema changes required
- No data migration required

---

## 🚫 Explicitly Out of Scope

The following features are **explicitly excluded** from future modifications:

### Application-Level Patterns (Never Implement)
- **GraphRAG, Agentic RAG, RAPTOR, CAG, KAG** - These are application patterns, not database features. Delegate to LangGraph, LlamaIndex, AutoGen.
- **Agent Orchestration** - VantaDB provides persistence, not orchestration logic.
- **Semantic Routing** - Application-level concern, not storage engine responsibility.

### Multi-Model Scope Creep (Never Implement)
- **JSON Complex Nesting** - Use MongoDB for nested documents. VantaDB maintains flat schema.
- **Geospatial Types** - Use PostGIS for spatial data. Outside AI agent memory scope.
- **Large Binary Blobs** - Use object storage (S3, local files) for large binaries. Breaks embedded model.
- **Time Series Database** - Use TimescaleDB for complex time series. VantaDB provides basic temporal types only.

### ML Infrastructure (Never Implement)
- **Fine-Tuning / Learning** - Contradicts deterministic, reproducible design. Use ML frameworks.
- **Adaptive Ranking** - Explicitly excluded in architecture docs <ref_file file="c:/Users/Eros/VantaDB Proyect/VantaDB/docs/architecture/ARCHITECTURE.md" line="68" />.
- **Graph Embeddings / Node2Vec** - Application-level ML, not storage engine feature.

### Streaming/Reactive (Never Implement)
- **Real-time Streaming** - VantaDB is synchronous, batch-oriented. Use Kafka/Pulsar for streaming.
- **Change Data Capture** - Not in scope for embedded local database.
- **Reactive Queries** - Use materialized views or specialized systems.

### Rationale
VantaDB maintains its **embedded-first, local-first** positioning by avoiding scope creep. The recommended 3 modifications provide high value while respecting architectural boundaries. All other requested features are better served by specialized systems or application-level frameworks.

---

## 📅 Implementation Timeline

### v0.2.x Development Block (8-10 weeks total)

**Week 1-3:** Modification 1 - Native Date/Time Types
- Week 1: Phase 1 (Core Type Extensions)
- Week 2: Phase 2 (Indexing Support)
- Week 3: Phase 3-4 (SDK Surface + Testing)

**Week 4-5:** Modification 2 - Flat Array/List Types
- Week 4: Phase 1-2 (Core Types + Indexing)
- Week 5: Phase 3-4 (SDK Surface + Testing)

**Week 6-9:** Modification 3 - Basic DAG Execution Primitives
- Week 6-7: Phase 1-2 (Cycle Detection + Topological Sort)
- Week 8: Phase 3-4 (DAG Validation API + SDK Integration)
- Week 9: Phase 5 (Testing + Validation)

**Week 10:** Integration and Documentation
- Cross-feature integration testing
- Documentation updates
- Release preparation

### Milestones

**M1 (Week 3):** Date/Time types production-ready
**M2 (Week 5):** Array types production-ready
**M3 (Week 9):** DAG validation production-ready
**M4 (Week 10):** v0.2.0 release candidate

---

## 🎯 Success Metrics

### Technical Metrics
- **Test Coverage:** ≥95% for all new code paths
- **Performance:** No regression in existing benchmarks (<5% degradation acceptable)
- **Backward Compatibility:** 100% compatibility with existing v0.1.x databases
- **Memory Footprint:** <10% increase in RSS for identical workloads

### User Metrics
- **Adoption:** ≥30% of active users adopt new features within 3 months
- **Satisfaction:** Positive feedback on temporal and array capabilities
- **Support Tickets:** No increase in support ticket volume related to new features
- **Documentation:** Clear user guides with examples for each new feature

### Strategic Metrics
- **Positioning:** Maintains "SQLite for AI Agents" positioning (no scope creep)
- **Competitive Differentiation:** Clear advantage over pure vector databases
- **Developer Experience:** Improved DX for common patterns (temporal, multi-value metadata)

---

## 🔗 Related Documentation

- **Architecture:** `docs/architecture/ARCHITECTURE.md` - Core design principles
- **Current Surface:** `docs/operations/EXPERIMENTAL_FEATURES.md` - Product boundary
- **Graph Implementation:** `src/graph.rs` - Existing BFS traversal
- **Type System:** `src/node.rs` - Current FieldValue enum
- **SDK Boundary:** `src/sdk.rs` - Public API surface

---

## 📝 Change History

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| 1.0 | 2026-06-10 | Initial proposal with 3 recommended modifications | Devin AI |

---

**Approval Required:** This document represents a proposal for v0.2.x development. Implementation should proceed only after:
1. Technical review by core maintainers
2. Alignment with VantaDB strategic positioning
3. Resource availability confirmation
4. User demand validation (optional but recommended)