use std::fmt::{self, Debug, Display, Formatter};
use std::hint::unreachable_unchecked;

use crate::NonZeroGpr;

/// Either zero or one of the general-purpose registers, `r1..=r31`.
///
/// Typeset in the PowerPC manual as (*r*X|0) for some letter X.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GprOrZero {
    Zero,
    Gpr(NonZeroGpr),
}

impl GprOrZero {
    pub fn new(x: u32) -> Option<GprOrZero> {
        match x {
            0 => Some(GprOrZero::Zero),
            x if x < 32 => Some(GprOrZero::Gpr(NonZeroGpr::new(x).unwrap())),
            _ => None,
        }
    }

    /// SAFETY: x must be in 0..32.
    pub unsafe fn new_unchecked(x: u32) -> GprOrZero {
        match x {
            0 => GprOrZero::Zero,
            x if x < 32 => GprOrZero::Gpr(NonZeroGpr::new(x).unwrap()),
            _ => unreachable_unchecked(),
        }
    }

    pub fn is_zero(self) -> bool {
        self == GprOrZero::Zero
    }

    pub fn is_gpr(self) -> bool {
        match self {
            GprOrZero::Gpr(_) => true,
            _ => false,
        }
    }

    pub fn try_unwrap_gpr(self) -> Option<NonZeroGpr> {
        match self {
            GprOrZero::Zero => None,
            GprOrZero::Gpr(gpr) => Some(gpr),
        }
    }

    pub fn unwrap_gpr(self) -> NonZeroGpr {
        match self {
            GprOrZero::Zero => panic!("GprOrZero::unwrap_gpr() called with Zero"),
            GprOrZero::Gpr(gpr) => gpr,
        }
    }

    pub fn as_u32(self) -> u32 {
        match self {
            GprOrZero::Zero => 0,
            GprOrZero::Gpr(gpr) => gpr.as_u32(),
        }
    }
}

impl Display for GprOrZero {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            GprOrZero::Zero => write!(f, "0"),
            GprOrZero::Gpr(x) => write!(f, "r{}", x.as_u32()),
        }
    }
}

impl Debug for GprOrZero {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        <GprOrZero as Display>::fmt(self, f)
    }
}

impl From<NonZeroGpr> for GprOrZero {
    fn from(gpr: NonZeroGpr) -> Self {
        GprOrZero::Gpr(gpr)
    }
}

#[cfg(test)]
mod gpr_or_zero_tests {
    use super::GprOrZero;

    #[test]
    fn new_in_range() {
        assert!(GprOrZero::new(0).unwrap().is_zero());
        assert_eq!(GprOrZero::new(5).unwrap().unwrap_gpr().as_u32(), 5);
        assert_eq!(GprOrZero::new(31).unwrap().unwrap_gpr().as_u32(), 31);
    }

    #[test]
    fn new_out_of_range() {
        assert!(GprOrZero::new(32).is_none());
        assert!(GprOrZero::new(std::u32::MAX).is_none());
    }

    #[test]
    fn display() {
        assert_eq!(format!("{}", GprOrZero::new(0).unwrap()), "0");
        assert_eq!(format!("{}", GprOrZero::new(5).unwrap()), "r5");
    }

    #[test]
    fn debug() {
        assert_eq!(format!("{:?}", GprOrZero::new(0).unwrap()), "0");
        assert_eq!(format!("{:?}", GprOrZero::new(5).unwrap()), "r5");
    }
}
