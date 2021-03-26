use std::fmt::{self, Debug, Display, Formatter};

use crate::{Crf, Gpr, GprOrZero, Spr};

pub mod crf;
pub mod gpr;
pub mod gpr_or_zero;
pub mod non_zero_gpr;
pub mod spr;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Register {
    Zero,
    GeneralPurpose(Gpr),
    SpecialPurpose(Spr),
    Condition(Crf),
}

impl From<Gpr> for Register {
    fn from(gpr: Gpr) -> Register {
        Register::GeneralPurpose(gpr)
    }
}

impl From<GprOrZero> for Register {
    fn from(gprz: GprOrZero) -> Register {
        match gprz.try_unwrap_gpr() {
            Some(gpr) => Register::GeneralPurpose(gpr.as_gpr()),
            None => Register::Zero,
        }
    }
}

impl From<Spr> for Register {
    fn from(spr: Spr) -> Register {
        Register::SpecialPurpose(spr)
    }
}

impl From<Crf> for Register {
    fn from(crf: Crf) -> Register {
        Register::Condition(crf)
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Register::Zero => write!(f, "0"),
            Register::GeneralPurpose(gpr) => write!(f, "{}", gpr),
            Register::SpecialPurpose(spr) => write!(f, "{}", spr),
            Register::Condition(crf) => write!(f, "{}", crf),
        }
    }
}

impl Debug for Register {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        <Register as Display>::fmt(self, f)
    }
}
