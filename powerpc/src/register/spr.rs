use std::fmt::{self, Debug, Display, Formatter};

/// A special-purpose register.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Spr {
    IntegerException,
    Link,
    Count,
    Gqr1,
    Gqr2,
    Gqr3,
    Gqr4,
    Gqr5,
    Gqr6,
    Gqr7,
}

impl Spr {
    pub fn new(spr: u32) -> Option<Spr> {
        match spr {
            0b00000_00001 => Some(Spr::IntegerException),
            0b00000_01000 => Some(Spr::Link),
            0b00000_01001 => Some(Spr::Count),
            0b11100_10001 => Some(Spr::Gqr1),
            0b11100_10010 => Some(Spr::Gqr2),
            0b11100_10011 => Some(Spr::Gqr3),
            0b11100_10100 => Some(Spr::Gqr4),
            0b11100_10101 => Some(Spr::Gqr5),
            0b11100_10110 => Some(Spr::Gqr6),
            0b11100_10111 => Some(Spr::Gqr7),
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
            Spr::Gqr1 => write!(f, "gqr1"),
            Spr::Gqr2 => write!(f, "gqr2"),
            Spr::Gqr3 => write!(f, "gqr3"),
            Spr::Gqr4 => write!(f, "gqr4"),
            Spr::Gqr5 => write!(f, "gqr5"),
            Spr::Gqr6 => write!(f, "gqr6"),
            Spr::Gqr7 => write!(f, "gqr7"),
        }
    }
}

impl Debug for Spr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        <Spr as Display>::fmt(self, f)
    }
}
