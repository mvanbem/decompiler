use std::any::Any;
use std::collections::BTreeSet;
use std::fmt::{self, Display, Formatter};

use crate::fact::{DefaultFact, Fact};

/// This address is the target of one or more branch instructions.
///
/// Note that the instruction immediately following a branch instruction is considered a branch
/// target.
#[derive(Default, Debug)]
pub struct BranchTargetFact {
    sources: BTreeSet<u32>,
}

impl BranchTargetFact {
    pub fn record_source(&mut self, source: u32) {
        self.sources.insert(source);
    }
}

impl Fact for BranchTargetFact {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_display(&self) -> Option<&dyn Display> {
        Some(self)
    }
}

impl DefaultFact for BranchTargetFact {
    fn default() -> Box<Self> {
        Box::new(Default::default())
    }
}

impl Display for BranchTargetFact {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "#[branch_target(sources = [")?;
        let mut first = true;
        for source in self.sources.iter().copied() {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            write!(f, "0x{:08x}", source)?;
        }
        write!(f, "])]")
    }
}
