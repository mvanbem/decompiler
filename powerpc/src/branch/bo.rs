use std::hint::unreachable_unchecked;

use crate::{ConditionBehavior, ConditionBit, CtrBehavior};

/// The 'bo' instruction field as defined in the PowerPC manual.
pub struct Bo(u32);

impl Bo {
    pub fn new(x: u32) -> Bo {
        Bo(x)
    }

    pub fn ctr(self) -> CtrBehavior {
        match self.0 & 0x06 {
            0x00 => CtrBehavior::DecrementBranchNonzero,
            0x02 => CtrBehavior::DecrementBranchZero,
            0x04 | 0x06 => CtrBehavior::None,
            // SAFETY: All bit patterns are covered.
            _ => unsafe { unreachable_unchecked() },
        }
    }

    pub fn modify_condition(self, condition: ConditionBit) -> ConditionBehavior {
        match self.0 & 0x18 {
            0x00 => ConditionBehavior::BranchFalse(condition),
            0x08 => ConditionBehavior::BranchTrue(condition),
            0x10 | 0x18 => ConditionBehavior::BranchAlways,
            // SAFETY: All bit patterns are covered.
            _ => unsafe { unreachable_unchecked() },
        }
    }
}
