use std::fmt::{self, Display, Formatter};

/// Branch instruction behavior for the count register.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CtrBehavior {
    DecrementBranchNonzero,
    DecrementBranchZero,
    None,
}

impl Display for CtrBehavior {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            CtrBehavior::DecrementBranchNonzero => write!(f, "dnz"),
            CtrBehavior::DecrementBranchZero => write!(f, "dz"),
            CtrBehavior::None => Ok(()),
        }
    }
}
