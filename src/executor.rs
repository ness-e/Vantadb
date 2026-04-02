use crate::error::{Result, IadbmsError};
use crate::query::{LogicalPlan, LogicalOperator, Statement};
use crate::node::{UnifiedNode, VectorData};
use crate::storage::StorageEngine;

pub enum ExecutionResult {
    Read(Vec<UnifiedNode>),
    Write { affected_nodes: usize, message: String },
}

pub struct Executor<'a> {
    storage: &'a StorageEngine,
}

impl<'a> Executor<'a> {
    pub fn new(storage: &'a StorageEngine) -> Self {
        Self { storage }
    }

    /// Ejecuta el Statement completo, distinguiendo entre Query de lectura y DML de escritura
    pub async fn execute_statement(&self, statement: Statement) -> Result<ExecutionResult> {
        match statement {
            Statement::Query(query) => {
                let plan = query.into_logical_plan();
                let nodes = self.execute_plan(plan).await?;
                Ok(ExecutionResult::Read(nodes))
            }
            Statement::Insert(insert) => {
                let mut node = UnifiedNode::new(insert.node_id);
                node.set_field("type", crate::node::FieldValue::String(insert.node_type));
                
                // Copy all provided fields
                for (k, v) in insert.fields.clone() {
                    node.set_field(&k, v);
                }
                
                // Auto-Embedding Logic: If VECTOR is not provided in IQL, but "texto" field exists!
                if insert.vector.is_none() {
                    if let Some(crate::node::FieldValue::String(text)) = insert.fields.get("texto") {
                        let llm = crate::llm::LlmClient::new();
                        // Request vectors to local Ollama inference bridge
                        if let Ok(vec) = llm.generate_embedding(text).await {
                            node.vector = VectorData::F32(vec);
                            node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                        }
                    }
                } else if let Some(vec) = insert.vector {
                    node.vector = VectorData::F32(vec);
                    node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                }

                self.storage.insert(&node)?;
                Ok(ExecutionResult::Write { affected_nodes: 1, message: format!("Node {} inserted.", insert.node_id) })
            }
            Statement::Update(update) => {
                let mut node = match self.storage.get(update.node_id)? {
                    Some(n) => n,
                    None => return Err(IadbmsError::Execution(format!("Node {} not found for update", update.node_id))),
                };
                for (k, v) in update.fields {
                    node.set_field(k, v);
                }
                if let Some(vec) = update.vector {
                    node.vector = VectorData::F32(vec);
                    node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                }
                self.storage.insert(&node)?;
                Ok(ExecutionResult::Write { affected_nodes: 1, message: format!("Node {} updated.", update.node_id) })
            }
            Statement::Delete(delete) => {
                self.storage.delete(delete.node_id)?;
                Ok(ExecutionResult::Write { affected_nodes: 1, message: format!("Node {} deleted.", delete.node_id) })
            }
            Statement::Relate(relate) => {
                let mut node = match self.storage.get(relate.source_id)? {
                    Some(n) => n,
                    None => return Err(IadbmsError::Execution(format!("Source Node {} not found for relation", relate.source_id))),
                };
                if let Some(w) = relate.weight {
                    node.add_weighted_edge(relate.target_id, relate.label, w);
                } else {
                    node.add_edge(relate.target_id, relate.label);
                }
                self.storage.insert(&node)?;
                Ok(ExecutionResult::Write { affected_nodes: 1, message: format!("Edge related from {} to {}.", relate.source_id, relate.target_id) })
            }
            Statement::InsertMessage(msg) => {
                // Syntactic Sugar for Chat Threads: Creates a node and relates it.
                // Normally we'd use a UUID generator, but for MVP we use a timestamp-based ID or random
                let msg_id = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros() as u64;
                let mut node = UnifiedNode::new(msg_id);
                node.set_field("type", crate::node::FieldValue::String("Message".to_string()));
                node.set_field("role", crate::node::FieldValue::String(msg.msg_role.clone()));
                node.set_field("content", crate::node::FieldValue::String(msg.content.clone()));
                
                // Embed directly via LLM since it's a message
                let llm = crate::llm::LlmClient::new();
                if let Ok(vec) = llm.generate_embedding(&msg.content).await {
                    node.vector = VectorData::F32(vec);
                    node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                }

                // Node is saved
                self.storage.insert(&node)?;

                // Now create relationship: MESSAGE -> belongs_to -> THREAD
                node.add_edge(msg.thread_id, "belongs_to_thread".to_string());
                self.storage.insert(&node)?;

                Ok(ExecutionResult::Write { affected_nodes: 2, message: format!("Message {} inserted and linked to Thread {}.", msg_id, msg.thread_id) })
            }
        }
    }

    /// Evaluates the Logical Plan over the underlying storage engine
    pub async fn execute_plan(&self, plan: LogicalPlan) -> Result<Vec<UnifiedNode>> {
        let mut results = Vec::new();
        let mut target_nodes = Vec::new();

        // Pass 1: Resolver Escaneo Vectorial Dinámico (Si hubiere Condition::VectorSim)
        let mut searched_hnsw = false;

        for op in &plan.operators {
            if let LogicalOperator::VectorSearch { field: _, query_vec, min_score: _ } = op {
                let llm = crate::llm::LlmClient::new();
                
                // Real Inference: Translate NLP into Embedded Vectors
                if let Ok(vector) = llm.generate_embedding(query_vec).await {
                    let index = self.storage.hnsw.read().unwrap();
                    let neighbors = index.search_nearest(&vector, 0, 5); // MVP: top_k = 5
                    
                    for (id, _sim) in neighbors {
                        target_nodes.push(id);
                    }
                    searched_hnsw = true;
                }
            }
        }

        if !searched_hnsw {
            // Fallback mock if it's not a vector query (linear scan mock)
            target_nodes.push(1); 
        }

        // Pass 2: Materializar los nodos devueltos por el índice y filtrar RBAC
        for id in target_nodes {
            if let Ok(Some(node)) = self.storage.get(id) {
                // Agented RBAC (Role-Based Access Control) Graph pruning
                if let Some(required_role) = &plan.enforce_role {
                    let mut role_match = false;
                    if let Some(crate::node::FieldValue::String(node_role)) = node.fields.get("_owner_role") {
                        if node_role == required_role {
                            role_match = true;
                        }
                    }
                    if !role_match && required_role != "admin" {
                        continue; // Prune branch (Sub-graph isolation enforced)
                    }
                }
                
                results.push(node);
            }
        }

        Ok(results)
    }
}
