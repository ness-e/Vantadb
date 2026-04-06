use crate::node::{UnifiedNode, VectorRepresentations};

#[derive(Debug, Clone)]
pub enum Opcode {
    OpPushFloat(f32),
    OpPushVector(VectorRepresentations),
    OpTrustCheck,
    OpVecSim,
    OpRehydrate,
}

pub struct NeuLispVM {
    float_stack: Vec<f32>,
    vec_stack: Vec<VectorRepresentations>,
    pub needs_rehydration: bool,
}

impl NeuLispVM {
    pub fn new() -> Self {
        Self {
            float_stack: Vec::new(),
            vec_stack: Vec::new(),
            needs_rehydration: false,
        }
    }

    /// Executa el array de bytecode (Opcodes) retornando (Valor, TrustScore)
    pub fn execute(&mut self, program: &[Opcode], current_context: &UnifiedNode) -> Result<(f32, f32), String> {
        // En v0.4.0, cada ejecución NeuLISP evalúa un TrustScore inherente base general
        let mut op_trust = current_context.trust_score;

        for op in program {
            match op {
                Opcode::OpPushFloat(f) => {
                    self.float_stack.push(*f);
                }
                Opcode::OpPushVector(v) => {
                    self.vec_stack.push(v.clone());
                }
                Opcode::OpTrustCheck => {
                    // Empuja a la pila de floats el trust score de contexto
                    self.float_stack.push(current_context.trust_score);
                }
                Opcode::OpVecSim => {
                    let v2 = self.vec_stack.pop().ok_or("Stack underflow: OP_VEC_SIM")?;
                    let v1 = self.vec_stack.pop().ok_or("Stack underflow: OP_VEC_SIM")?;
                    
                    if let Some(sim) = v1.cosine_similarity(&v2) {
                        self.float_stack.push(sim);
                    } else {
                        // Penalizar trust si no hay similitud cálculable
                        op_trust *= 0.8;
                        self.float_stack.push(0.0);
                    }
                }
                Opcode::OpRehydrate => {
                    self.needs_rehydration = true;
                    // Retorna temporalmente NaN float o similar para la pila (o simplemente ignora)
                    self.float_stack.push(0.0);
                }
            }
        }
        
        let result_val = self.float_stack.pop().unwrap_or(0.0);
        Ok((result_val, op_trust))
    }
}
