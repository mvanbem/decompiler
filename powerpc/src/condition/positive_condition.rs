use std::fmt::{self, Display, Formatter};

use crate::Condition;

/// Formatting wrapper for `Condition`.
#[derive(Clone, Copy, Debug)]
pub struct PositiveCondition(pub Condition);

impl Display for PositiveCondition {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.0 {
                Condition::Less => "lt",
                Condition::Greater => "gt",
                Condition::Equal => "eq",
                Condition::SummaryOverflow => "so",
            }
        )
    }
}
