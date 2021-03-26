use std::fmt::{self, Display, Formatter};

use crate::{ConditionBit, Crf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConditionBehavior {
    BranchFalse(ConditionBit),
    BranchTrue(ConditionBit),
    BranchAlways,
}

impl ConditionBehavior {
    pub fn crf(self) -> Option<Crf> {
        match self {
            ConditionBehavior::BranchFalse(c) => Some(c.crf()),
            ConditionBehavior::BranchTrue(c) => Some(c.crf()),
            ConditionBehavior::BranchAlways => None,
        }
    }
}

impl Display for ConditionBehavior {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ConditionBehavior::BranchFalse(c) => write!(f, "{}", c.bi().negative()),
            ConditionBehavior::BranchTrue(c) => write!(f, "{}", c.bi().positive()),
            ConditionBehavior::BranchAlways => Ok(()),
        }
    }
}
