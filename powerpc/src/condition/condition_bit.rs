use std::fmt::{self, Debug, Display, Formatter};

use crate::{Condition, Crf};

/// A bit in the condition register, `0..=31`.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConditionBit(u32);

impl ConditionBit {
    pub fn new(x: u32) -> Option<Self> {
        if x < 32 {
            Some(Self(x))
        } else {
            None
        }
    }

    /// SAFETY: x must be in 0..32.
    pub const unsafe fn new_unchecked(x: u32) -> Self {
        Self(x)
    }

    pub fn from_crf_and_condition(crf: Crf, condition: Condition) -> Self {
        // SAFETY: crf is in 0..8 and condition is in 0..4, so 4 * crf + condition is in 0..32.
        unsafe { Self::new_unchecked(4 * crf.get() + condition.as_u32()) }
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
