use std::fmt::{self, Display, Formatter};
use std::hint::unreachable_unchecked;

use crate::{NegativeCondition, PositiveCondition};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Condition {
    Less,
    Greater,
    Equal,
    SummaryOverflow,
}

impl Condition {
    pub fn new(x: u32) -> Option<Condition> {
        if x < 4 {
            // SAFETY: x is in 0..4, proven by the test above.
            Some(unsafe { Condition::new_unchecked(x) })
        } else {
            None
        }
    }

    /// SAFETY: x must be in 0..4.
    pub unsafe fn new_unchecked(x: u32) -> Condition {
        match x {
            0 => Condition::Less,
            1 => Condition::Greater,
            2 => Condition::Equal,
            3 => Condition::SummaryOverflow,
            _ => unreachable_unchecked(),
        }
    }

    pub fn as_u32(self) -> u32 {
        match self {
            Condition::Less => 0,
            Condition::Greater => 1,
            Condition::Equal => 2,
            Condition::SummaryOverflow => 3,
        }
    }

    pub fn positive(self) -> PositiveCondition {
        PositiveCondition(self)
    }

    pub fn negative(self) -> NegativeCondition {
        NegativeCondition(self)
    }
}

impl Display for Condition {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Condition::Less => write!(f, "lt"),
            Condition::Greater => write!(f, "gt"),
            Condition::Equal => write!(f, "eq"),
            Condition::SummaryOverflow => write!(f, "so"),
        }
    }
}
