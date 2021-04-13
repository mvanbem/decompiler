use std::collections::{BTreeSet, HashMap};

use powerpc::Register;
use symbolic::ExprRef;

use crate::fact::basic_block_end::BasicBlockEndFact;
use crate::fact::branch_target::BranchTargetFact;
use crate::fact::Fact;
use crate::fact_database::FactDatabase;
use crate::powerpc_symbolic::Write;

#[derive(Debug)]
pub struct BasicBlockFact {
    end_addr: u32,
    predecessors: Vec<u32>,
    successors: Vec<u32>,
    writes: Vec<Write>,
    registers_leaving: HashMap<Register, ExprRef>,
}

impl BasicBlockFact {
    pub fn end_addr(&self) -> u32 {
        self.end_addr
    }

    pub fn predecessors(&self) -> &[u32] {
        &self.predecessors
    }

    pub fn successors(&self) -> &[u32] {
        &self.successors
    }

    pub fn writes(&self) -> &[Write] {
        &self.writes
    }

    pub fn record_write(&mut self, write: Write) {
        self.writes.push(write);
    }

    // pub fn registers_leaving(&self) -> &HashMap<Register, ExprRef> {
    //     &self.registers_leaving
    // }

    // pub fn record_register_leaving(&mut self, register: Register, expr: ExprRef) {
    //     assert!(self.registers_leaving.insert(register, expr).is_none());
    // }
}

impl Fact for BasicBlockFact {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
pub struct BasicBlockFactBuilder {
    end_addr: u32,
    predecessors: BTreeSet<u32>,
    successors: Vec<u32>,
}

impl BasicBlockFactBuilder {
    /// Scans a basic block, relying on previously inserted [`BasicBlockEndFact`]s and
    /// [`BranchTargetFact`]s.
    pub fn new(db: &FactDatabase, mut next_addr: u32) -> Self {
        let mut successors = BTreeSet::new();
        loop {
            let addr = next_addr;
            next_addr += 4;

            // End the basic block where a relevant branch was noted.
            if let Some(end_fact) = db.get_fact::<BasicBlockEndFact>(addr) {
                // Ensure the target basic blocks are known and record the outgoing edges.
                successors.extend(end_fact.successors());
                break;
            }

            // End the basic block if the next instruction is a branch target, and thus the start of
            // its own basic block.
            if db.get_fact::<BranchTargetFact>(next_addr).is_some() {
                successors.insert(next_addr);
                break;
            }
        }

        Self {
            end_addr: next_addr,
            predecessors: BTreeSet::new(), // To be filled in by the caller.
            successors: successors.into_iter().collect(),
        }
    }

    pub fn insert_predecessor(&mut self, predecessor: u32) {
        self.predecessors.insert(predecessor);
    }

    pub fn successors(&self) -> &[u32] {
        &self.successors
    }

    pub fn build(self) -> BasicBlockFact {
        BasicBlockFact {
            end_addr: self.end_addr,
            predecessors: self.predecessors.into_iter().collect(),
            successors: self.successors,
            writes: Vec::new(),
            registers_leaving: HashMap::new(),
        }
    }
}
