use crate::{ConditionBehavior, CtrBehavior};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchInfo {
    pub condition: ConditionBehavior,
    pub ctr: CtrBehavior,
    pub link: bool,
    pub target: Option<u32>,
}

impl BranchInfo {
    pub fn is_conditional(&self) -> bool {
        (match self.condition {
            ConditionBehavior::BranchFalse(..) | ConditionBehavior::BranchTrue(..) => true,
            ConditionBehavior::BranchAlways => false,
        }) | match self.ctr {
            CtrBehavior::DecrementBranchNonzero | CtrBehavior::DecrementBranchZero => true,
            CtrBehavior::None => false,
        }
    }

    /// True if the branch is unconditional and not linked.
    pub fn diverges(&self) -> bool {
        !self.is_conditional() && !self.link
    }
}
