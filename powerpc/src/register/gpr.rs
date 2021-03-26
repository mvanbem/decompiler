use std::fmt::{self, Debug, Display, Formatter};

/// One of the general-purpose registers, `r0..=r31`.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Gpr(u32);

impl Gpr {
    pub fn new(x: u32) -> Option<Gpr> {
        if x < 32 {
            Some(Gpr(x))
        } else {
            None
        }
    }

    /// SAFETY: x must be in 0..32.
    pub unsafe fn new_unchecked(x: u32) -> Gpr {
        Gpr(x)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl Display for Gpr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "r{}", self.0)
    }
}

impl Debug for Gpr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        <Gpr as Display>::fmt(self, f)
    }
}

#[cfg(test)]
mod gpr_tests {
    use super::Gpr;

    #[test]
    fn new_in_range() {
        assert_eq!(Gpr::new(0).unwrap().as_u32(), 0);
        assert_eq!(Gpr::new(5).unwrap().as_u32(), 5);
        assert_eq!(Gpr::new(31).unwrap().as_u32(), 31);
    }

    #[test]
    fn new_out_of_range() {
        assert!(Gpr::new(32).is_none());
        assert!(Gpr::new(std::u32::MAX).is_none());
    }

    #[test]
    fn display() {
        assert_eq!(format!("{}", Gpr::new(0).unwrap()), "r0");
        assert_eq!(format!("{}", Gpr::new(5).unwrap()), "r5");
    }

    #[test]
    fn debug() {
        assert_eq!(format!("{:?}", Gpr::new(0).unwrap()), "r0");
        assert_eq!(format!("{:?}", Gpr::new(5).unwrap()), "r5");
    }
}
