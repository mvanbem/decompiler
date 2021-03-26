use std::fmt::{self, Display, Formatter};

use crate::Condition;

/// Formatting wrapper for `Condition`.
#[derive(Clone, Copy, Debug)]
pub struct NegativeCondition(pub Condition);

impl Display for NegativeCondition {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.0 {
                Condition::Less => "nl",
                Condition::Greater => "ng",
                Condition::Equal => "ne",
                Condition::SummaryOverflow => "ns",
            }
        )
    }
}
