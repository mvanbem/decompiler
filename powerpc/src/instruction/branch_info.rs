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

    // TODO: Reintroduce this as part of the decompiler!
    /*
    fn interpretation(&self) -> BranchInterpretation {
        if self.link {
            BranchInterpretation {
                trace_target: false,
                scan: BasicBlockScan::Continue,
            }
        } else {
            BranchInterpretation {
                trace_target: true,
                scan: BasicBlockScan::EndBasicBlock {
                    trace_next: self.is_conditional(),
                },
            }
        }
    }
    */
}
