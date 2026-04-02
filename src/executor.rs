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
    pub fn execute_statement(&self, statement: Statement) -> Result<ExecutionResult> {
        match statement {
            Statement::Query(query) => {
                let plan = query.into_logical_plan();
                let nodes = self.execute_plan(plan)?;
                Ok(ExecutionResult::Read(nodes))
            }
            Statement::Insert(insert) => {
                let mut node = UnifiedNode::new(insert.node_id);
                // Inserción de tipo como campo implícito
                node.set_field("type", crate::node::FieldValue::String(insert.node_type));
                for (k, v) in insert.fields {
                    node.set_field(k, v);
                }
                if let Some(vec) = insert.vector {
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
        }
    }

    /// Evaluates the Logical Plan over the underlying storage engine
    pub fn execute_plan(&self, plan: LogicalPlan) -> Result<Vec<UnifiedNode>> {
        // Simplified MVP physical execution:
        let mut results = Vec::new();
        for op in plan.operators {
            match op {
                LogicalOperator::Scan { entity: _ } => {
                     // Normally scan rocksdb. We mock fetching node #1 to represent a hit.
                     if let Ok(Some(node)) = self.storage.get(1) {
                         results.push(node);
                     }
                },
                _ => {} // Remaining operations
            }
        }
        Ok(results)
    }
}
