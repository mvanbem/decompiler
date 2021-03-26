use std::fmt::{self, Debug, Display, Formatter};

// One of the eight condition register fields, `crf0..=crf7`.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Crf(u32);

impl Crf {
    pub fn new(x: u32) -> Option<Crf> {
        if x < 8 {
            Some(Crf(x))
        } else {
            None
        }
    }

    /// SAFETY: x must be in 0..8.
    pub unsafe fn new_unchecked(x: u32) -> Crf {
        Crf(x)
    }

    pub fn get(self) -> u32 {
        self.0
    }

    pub fn nonzero(self) -> Option<Crf> {
        if self.0 > 0 {
            Some(self)
        } else {
            None
        }
    }
}

impl Display for Crf {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "cr{}", self.0)
    }
}

impl Debug for Crf {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        <Crf as Display>::fmt(self, f)
    }
}
