use std::collections::BTreeSet;

use crate::fact::{DefaultFact, Fact};

#[derive(Default)]
pub struct BasicBlockEndFact {
    successors: BTreeSet<u32>,
}

impl BasicBlockEndFact {
    pub fn record_successor(&mut self, successor: u32) {
        self.successors.insert(successor);
    }

    pub fn successors(&self) -> impl Iterator<Item = u32> + '_ {
        self.successors.iter().copied()
    }
}

impl Fact for BasicBlockEndFact {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl DefaultFact for BasicBlockEndFact {
    fn default() -> Box<Self> {
        Box::new(Default::default())
    }
}
