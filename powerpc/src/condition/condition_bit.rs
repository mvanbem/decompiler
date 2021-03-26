use std::fmt::{self, Debug, Display, Formatter};

use crate::{Condition, Crf};

/// A bit in the condition register, `0..=31`.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct ConditionBit(u32);

impl ConditionBit {
    pub fn new(x: u32) -> Option<ConditionBit> {
        if x < 32 {
            Some(ConditionBit(x))
        } else {
            None
        }
    }

    /// SAFETY: x must be in 0..32.
    pub unsafe fn new_unchecked(x: u32) -> ConditionBit {
        ConditionBit(x)
    }

    pub fn crf(self) -> Crf {
        // SAFETY: self.0 is in 0..32, so shifting it right two places yields a value in 0..8.
        unsafe { Crf::new_unchecked(self.0 >> 2) }
    }

    pub fn bi(self) -> Condition {
        // SAFETY: Masking with 3 ensures the result is in 0..4.
        unsafe { Condition::new_unchecked(self.0 & 3) }
    }
}

impl Debug for ConditionBit {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "ConditionBit({} = {}", self.0, self)
    }
}

impl Display for ConditionBit {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let crf = self.crf();
        if crf.get() > 0 {
            write!(f, "{}*4+", crf)?;
        }
        write!(f, "{}", self.bi())
    }
}
