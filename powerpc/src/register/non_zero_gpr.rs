use std::fmt::{self, Debug, Display, Formatter};
use std::num::NonZeroU32;

use crate::Gpr;

/// One of the non-`r0` general-purpose registers, `r1..=r31`.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NonZeroGpr(NonZeroU32);

impl NonZeroGpr {
    pub fn new(x: u32) -> Option<NonZeroGpr> {
        if x > 0 && x < 32 {
            Some(NonZeroGpr(NonZeroU32::new(x).unwrap()))
        } else {
            None
        }
    }

    /// SAFETY: x must be in 1..32.
    pub unsafe fn new_unchecked(x: u32) -> NonZeroGpr {
        NonZeroGpr(NonZeroU32::new_unchecked(x))
    }

    pub fn as_gpr(self) -> Gpr {
        // SAFETY: The value is in 1..32, a subset of the required 0..32.
        unsafe { Gpr::new_unchecked(self.0.get()) }
    }

    pub fn as_non_zero_u32(self) -> NonZeroU32 {
        self.0
    }

    pub fn as_u32(self) -> u32 {
        self.0.get()
    }
}

impl Display for NonZeroGpr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "r{}", self.0)
    }
}

impl Debug for NonZeroGpr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        <NonZeroGpr as Display>::fmt(self, f)
    }
}
