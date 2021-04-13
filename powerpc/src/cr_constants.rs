use crate::Crf;
use crate::{Condition, ConditionBit};

pub const CR0: Crf = unsafe { Crf::new_unchecked(0) };
pub const CR1: Crf = unsafe { Crf::new_unchecked(1) };
pub const CR2: Crf = unsafe { Crf::new_unchecked(2) };
pub const CR3: Crf = unsafe { Crf::new_unchecked(3) };
pub const CR4: Crf = unsafe { Crf::new_unchecked(4) };
pub const CR5: Crf = unsafe { Crf::new_unchecked(5) };
pub const CR6: Crf = unsafe { Crf::new_unchecked(6) };
pub const CR7: Crf = unsafe { Crf::new_unchecked(7) };

pub const LT: Condition = Condition::Less;
pub const GT: Condition = Condition::Greater;
pub const EQ: Condition = Condition::Equal;
pub const SO: Condition = Condition::SummaryOverflow;

pub const CR0LT: ConditionBit = unsafe { ConditionBit::new_unchecked(0) };
pub const CR0GT: ConditionBit = unsafe { ConditionBit::new_unchecked(1) };
pub const CR0EQ: ConditionBit = unsafe { ConditionBit::new_unchecked(2) };
pub const CR0SO: ConditionBit = unsafe { ConditionBit::new_unchecked(3) };
