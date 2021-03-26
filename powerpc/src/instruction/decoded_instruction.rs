use std::fmt::{self, Display, Formatter};

use crate::instruction::format_small_i16::FormatSmallI16;
use crate::instruction::format_small_u16::FormatSmallU16;
use crate::{BranchInfo, ConditionBehavior, Crf, CtrBehavior, Gpr, GprOrZero, NonZeroGpr, Spr};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodedInstruction {
    Addi {
        dst: Gpr,
        src: GprOrZero,
        immediate: i16,
    },
    B {
        link: bool,
        absolute: bool,
        target: u32,
    },
    Bc {
        condition: ConditionBehavior,
        ctr: CtrBehavior,
        link: bool,
        absolute: bool,
        target: u32,
    },
    Bclr {
        condition: ConditionBehavior,
        ctr: CtrBehavior,
        link: bool,
    },
    Cmpli {
        crf: Crf,
        src: Gpr,
        immediate: u16,
    },
    Lwz {
        dst: Gpr,
        offset: i16,
        base: GprOrZero,
    },
    Mfspr {
        spr: Spr,
        dst: Gpr,
    },
    Mtspr {
        spr: Spr,
        src: Gpr,
    },
    Or {
        dst: Gpr,
        srcs: [Gpr; 2],
        record: bool,
    },
    Stw {
        src: Gpr,
        offset: i16,
        base: GprOrZero,
    },
    Stwu {
        src: Gpr,
        offset: i16,
        base: NonZeroGpr,
    },
    UnimplementedOpcode {
        opcode: u32,
    },
    UnimplementedExtendedOpcode {
        opcode: u32,
        extended_opcode: u32,
    },
}

impl DecodedInstruction {
    pub fn is_valid(self) -> bool {
        match self {
            DecodedInstruction::UnimplementedOpcode { .. }
            | DecodedInstruction::UnimplementedExtendedOpcode { .. } => false,
            _ => true,
        }
    }

    pub fn branch_info(self) -> Option<BranchInfo> {
        match self {
            DecodedInstruction::B { link, target, .. } => Some(BranchInfo {
                condition: ConditionBehavior::BranchAlways,
                ctr: CtrBehavior::None,
                link,
                target: Some(target),
            }),
            DecodedInstruction::Bc {
                condition,
                ctr,
                link,
                target,
                ..
            } => Some(BranchInfo {
                condition,
                ctr,
                link,
                target: Some(target),
            }),
            DecodedInstruction::Bclr {
                condition,
                ctr,
                link,
            } => Some(BranchInfo {
                condition,
                ctr,
                link,
                target: None,
            }),
            _ => None,
        }
    }
}

impl Display for DecodedInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            DecodedInstruction::Addi {
                dst,
                src,
                immediate,
            } => {
                if src.is_gpr() {
                    write!(f, "addi {}, {}, {}", dst, src, FormatSmallI16(immediate))
                } else {
                    write!(f, "li {}, {}", dst, FormatSmallI16(immediate))
                }
            }
            DecodedInstruction::B {
                link,
                absolute,
                target,
            } => write!(
                f,
                "b{}{} 0x{:08x}",
                if link { "l" } else { "" },
                if absolute { "a" } else { "" },
                target
            ),
            DecodedInstruction::Bc {
                condition,
                ctr,
                link,
                absolute,
                target,
            } => {
                write!(
                    f,
                    "b{}{}{}{} ",
                    ctr,
                    condition,
                    if link { "l" } else { "" },
                    if absolute { "a" } else { "" },
                )?;
                if let Some(cr) = condition.crf().and_then(|crf| crf.nonzero()) {
                    write!(f, "{}, ", cr)?;
                }
                write!(f, "0x{:08x}", target)
            }
            DecodedInstruction::Bclr {
                condition,
                ctr,
                link,
            } => {
                write!(f, "b{}{}lr{}", ctr, condition, if link { "l" } else { "" })?;
                if let Some(cr) = condition.crf().and_then(|crf| crf.nonzero()) {
                    write!(f, " {}", cr)?;
                }
                Ok(())
            }
            DecodedInstruction::Cmpli {
                crf,
                src,
                immediate,
            } => {
                write!(f, "cmplwi ")?;
                if crf.get() > 0 {
                    write!(f, "{}, ", crf)?;
                }
                write!(f, "{}, {}", src, FormatSmallU16(immediate))
            }
            DecodedInstruction::Lwz { dst, offset, base } => {
                write!(f, "lwz {}, ", dst)?;
                if offset != 0 {
                    write!(f, "{}", FormatSmallI16(offset))?;
                }
                write!(f, "({})", base)
            }
            DecodedInstruction::Mfspr { spr, dst } => write!(f, "mf{} {}", spr, dst),
            DecodedInstruction::Mtspr { spr, src } => write!(f, "mt{} {}", spr, src),
            DecodedInstruction::Or { dst, srcs, record } => {
                if srcs[0] != srcs[1] {
                    write!(
                        f,
                        "or{} {}, {}, {}",
                        if record { "." } else { "" },
                        dst,
                        srcs[0],
                        srcs[1]
                    )
                } else {
                    write!(
                        f,
                        "mr{} {}, {}",
                        if record { "." } else { "" },
                        dst,
                        srcs[0]
                    )
                }
            }
            DecodedInstruction::Stw { src, offset, base } => {
                write!(f, "stw {}, ", src)?;
                if offset != 0 {
                    write!(f, "{}", FormatSmallI16(offset))?;
                }
                write!(f, "({})", base)
            }
            DecodedInstruction::Stwu { src, offset, base } => {
                write!(f, "stwu {}, ", src)?;
                if offset != 0 {
                    write!(f, "{}", FormatSmallI16(offset))?;
                }
                write!(f, "({})", base)
            }
            _ => write!(f, "unimplemented({:?})", self),
        }
    }
}
