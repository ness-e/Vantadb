use crate::parser::lisp::LispExpr;
use crate::executor::{Executor, ExecutionResult};
use crate::error::{ConnectomeError, Result};
use std::collections::BTreeMap;
use crate::node::FieldValue;

const MAX_FUEL: u64 = 1000;

pub struct LispSandbox<'a> {
    executor: &'a Executor<'a>,
    fuel: u64,
}

impl<'a> LispSandbox<'a> {
    pub fn new(executor: &'a Executor<'a>) -> Self {
         Self { executor, fuel: MAX_FUEL }
    }

    pub async fn eval(&mut self, expr: std::borrow::Cow<'_, LispExpr>) -> Result<ExecutionResult> {
         if self.fuel == 0 { 
             return Err(ConnectomeError::Execution("Sandbox Abort: Out of Cognitive Fuel (DDO Protected)".to_string())); 
         }
         self.fuel -= 1;
                  
         match expr.as_ref() {
             LispExpr::List(list) => {
                 if list.is_empty() { return Err(ConnectomeError::Execution("Empty LISP statement".to_string())); }
                 
                 if let LispExpr::Atom(func) = &list[0] {
                     match func.to_uppercase().as_str() {
                         "INSERT" => self.eval_insert(&list[1..]).await,
                         "MATCH" => Err(ConnectomeError::Execution("MATCH LISP logic pending".to_string())),
                         _ => Err(ConnectomeError::Execution(format!("Unknown LISP logic intrinsic: {}", func)))
                     }
                 } else {
                     Err(ConnectomeError::Execution("Expected function atom at beginning of expression".to_string()))
                 }
             },
             _ => Err(ConnectomeError::Execution("Top level must be a LISP List".to_string()))
         }
    }

    // MVP: (INSERT :neuron {:label "IA" :trust 0.9})
    async fn eval_insert(&mut self, args: &[LispExpr]) -> Result<ExecutionResult> {
        if args.len() < 2 { return Err(ConnectomeError::Execution("INSERT requires target and payload".to_string())); }

        let target = if let LispExpr::Keyword(k) = &args[0] { k.as_str() } else { "neuron" };
        let mut fields = BTreeMap::new();
        let node_type = target.to_string();
        
        let node_id = rand::random::<u64>(); // Generación genérica

        if let LispExpr::Map(map) = &args[1] {
            for (key, val) in map {
                if let LispExpr::Keyword(k) = key {
                    match val {
                        LispExpr::StringLiteral(s) => { fields.insert(k.clone(), FieldValue::String(s.clone())); },
                        LispExpr::Number(n) => { fields.insert(k.clone(), FieldValue::Float(*n as f64)); },
                        _ => {} // Fallback for simple map parser
                    }
                }
            }
        }
        
        // Atar Metadata Homoiconica por v0.4.0 directiva "sys_rule: true"
        fields.insert("sys_rule".to_string(), FieldValue::Bool(true));

        // Las reglas LISP son nodos cognitivos activos (STNeuron) —
        // deben vivir en cortex_ram para ser accesibles sin hit de disco.
        let mut node = crate::node::UnifiedNode::new(node_id);
        node.neuron_type = crate::node::NeuronType::STNeuron;
        node.set_field("type", FieldValue::String(node_type.clone()));
        for (k, v) in &fields {
            node.set_field(k.as_str(), v.clone());
        }

        self.executor.storage.insert(&node)?;
        Ok(ExecutionResult::Write {
            affected_nodes: 1,
            message: format!("LISP Node {} inserted as STNeuron.", node_id),
            node_id: Some(node_id),
        })
    }
}
