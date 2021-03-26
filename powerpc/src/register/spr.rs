use std::fmt::{self, Debug, Display, Formatter};

/// A special-purpose register.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Spr {
    IntegerException,
    Link,
    Count,
}

impl Spr {
    pub fn new(spr: u32) -> Option<Spr> {
        match spr {
            1 => Some(Spr::IntegerException),
            8 => Some(Spr::Link),
            9 => Some(Spr::Count),
            _ => None,
        }
    }
}

impl Display for Spr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Spr::IntegerException => write!(f, "xer"),
            Spr::Link => write!(f, "lr"),
            Spr::Count => write!(f, "ctr"),
        }
    }
}

impl Debug for Spr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        <Spr as Display>::fmt(self, f)
    }
}
