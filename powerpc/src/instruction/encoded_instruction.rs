use thiserror::Error;

use crate::{Bo, ConditionBit, Crf, DecodedInstruction, Gpr, GprOrZero, Spr};

#[derive(Clone, Copy, Debug)]
pub struct EncodedInstruction(pub u32);

impl EncodedInstruction {
    // Follows PowerPC manual convention:
    // - MSB is bit 0
    // - LSB is bit 31
    // - Bitfields are specified as inclusive ranges
    fn bits(self, from: u32, to: u32) -> u32 {
        if !(from < 32 && to < 32 && from <= to) {
            panic!("bad indices: bits(_, {}, {})", from, to);
        }
        (self.0 >> (31 - to)) & ((1 << (to - from + 1)) - 1)
    }

    // Getters for instruction bitfields. Only a few will apply to any
    // given instruction.

    fn opcode(self) -> u32 {
        self.bits(0, 5)
    }

    fn extended_opcode(self) -> u32 {
        self.bits(21, 30)
    }

    fn gpr_a(self) -> Gpr {
        Gpr::new(self.bits(11, 15)).unwrap()
    }

    fn gpr_a_or_zero(self) -> GprOrZero {
        GprOrZero::new(self.bits(11, 15)).unwrap()
    }

    fn bi(self) -> ConditionBit {
        ConditionBit::new(self.bits(11, 15)).unwrap()
    }

    fn gpr_b(self) -> Gpr {
        Gpr::new(self.bits(16, 20)).unwrap()
    }

    fn gpr_b_or_zero(self) -> GprOrZero {
        GprOrZero::new(self.bits(16, 20)).unwrap()
    }

    fn gpr_c(self) -> Gpr {
        Gpr::new(self.bits(6, 10)).unwrap()
    }

    fn gpr_c_or_zero(self) -> GprOrZero {
        GprOrZero::new(self.bits(6, 10)).unwrap()
    }

    fn crf_d(self) -> Crf {
        Crf::new(self.bits(6, 8)).unwrap()
    }

    fn bo(self) -> Bo {
        Bo::new(self.bits(6, 10))
    }

    fn try_spr(self) -> Option<Spr> {
        Spr::new((self.bits(16, 20) << 5) | self.bits(11, 15))
    }

    fn unsigned_immediate(self) -> u16 {
        self.bits(16, 31) as u16
    }

    fn signed_immediate(self) -> i16 {
        self.bits(16, 31) as i16
    }

    fn small_branch_offset(self) -> i32 {
        let mut tmp = self.0 & 0x0000fffc;
        // Sign extend.
        if tmp & 0x00008000 == 0x00008000 {
            tmp |= 0xffff0000;
        }
        tmp as i32
    }

    fn large_branch_offset(self) -> i32 {
        let mut tmp = self.0 & 0x03fffffc;
        // Sign extend.
        if tmp & 0x02000000 == 0x02000000 {
            tmp |= 0xfc000000;
        }
        tmp as i32
    }

    fn update_condition_register(self) -> bool {
        self.bits(31, 31) == 1
    }

    fn absolute_address(self) -> bool {
        self.bits(30, 30) == 1
    }

    fn link(self) -> bool {
        self.bits(31, 31) == 1
    }

    pub fn parse(self, address: u32) -> Result<DecodedInstruction, ParseError> {
        match self.opcode() {
            10 => {
                // Check reserved bit and width flag, which must be clear.
                if self.bits(9, 10) == 0 {
                    Ok(DecodedInstruction::Cmpli {
                        crf: self.crf_d(),
                        src: self.gpr_a(),
                        immediate: self.unsigned_immediate(),
                    })
                } else {
                    Err(ParseError::IllegalEncoding)
                }
            }
            14 => Ok(DecodedInstruction::Addi {
                dst: self.gpr_c(),
                src: self.gpr_a_or_zero(),
                immediate: self.signed_immediate(),
            }),
            16 => Ok(DecodedInstruction::Bc {
                condition: self.bo().modify_condition(self.bi()),
                ctr: self.bo().ctr(),
                link: self.link(),
                absolute: self.absolute_address(),
                target: if self.absolute_address() { 0 } else { address }
                    .wrapping_add(self.small_branch_offset() as u32),
            }),
            18 => Ok(DecodedInstruction::B {
                link: self.link(),
                absolute: self.absolute_address(),
                target: if self.absolute_address() { 0 } else { address }
                    .wrapping_add(self.large_branch_offset() as u32),
            }),
            opcode @ 19 => match self.extended_opcode() {
                16 => {
                    if self.bits(16, 20) == 0 {
                        Ok(DecodedInstruction::Bclr {
                            condition: self.bo().modify_condition(self.bi()),
                            ctr: self.bo().ctr(),
                            link: self.link(),
                        })
                    } else {
                        Err(ParseError::IllegalEncoding)
                    }
                }
                extended_opcode => Err(ParseError::UnimplementedExtendedOpcode {
                    opcode,
                    extended_opcode,
                }),
            },
            opcode @ 31 => match self.extended_opcode() {
                339 => match (self.try_spr(), self.bits(31, 31)) {
                    (Some(spr), 0) => Ok(DecodedInstruction::Mfspr {
                        spr,
                        dst: self.gpr_c(),
                    }),
                    _ => Err(ParseError::IllegalEncoding),
                },
                444 => Ok(DecodedInstruction::Or {
                    dst: self.gpr_a(),
                    srcs: [self.gpr_c(), self.gpr_b()],
                    record: self.update_condition_register(),
                }),
                467 => match (self.try_spr(), self.bits(31, 31)) {
                    (Some(spr), 0) => Ok(DecodedInstruction::Mtspr {
                        spr,
                        src: self.gpr_c(),
                    }),
                    _ => Err(ParseError::IllegalEncoding),
                },
                extended_opcode => Err(ParseError::UnimplementedExtendedOpcode {
                    opcode,
                    extended_opcode,
                }),
            },
            32 => Ok(DecodedInstruction::Lwz {
                dst: self.gpr_c(),
                offset: self.signed_immediate(),
                base: self.gpr_a_or_zero(),
            }),
            36 => Ok(DecodedInstruction::Stw {
                src: self.gpr_c(),
                offset: self.signed_immediate(),
                base: self.gpr_a_or_zero(),
            }),
            37 => {
                if let Some(base) = self.gpr_a_or_zero().try_unwrap_gpr() {
                    Ok(DecodedInstruction::Stwu {
                        src: self.gpr_c(),
                        offset: self.signed_immediate(),
                        base,
                    })
                } else {
                    Err(ParseError::IllegalEncoding)
                }
            }
            opcode => Err(ParseError::UnimplementedOpcode(opcode)),
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unimplemented opcode: {0}")]
    UnimplementedOpcode(u32),

    #[error(
        "unimplemented extended opcode: opcode = {opcode}, extended_opcode = {extended_opcode}"
    )]
    UnimplementedExtendedOpcode { opcode: u32, extended_opcode: u32 },

    #[error("illegal encoding")]
    IllegalEncoding,
}

#[cfg(test)]
mod tests {
    use super::EncodedInstruction;

    #[test]
    fn bits_basic() {
        // |xxx-----------------------------|
        assert_eq!(EncodedInstruction(0x00000000).bits(0, 2), 0x0);
        assert_eq!(EncodedInstruction(0x20000000).bits(0, 2), 0x1);
        assert_eq!(EncodedInstruction(0x40000000).bits(0, 2), 0x2);
        assert_eq!(EncodedInstruction(0x80000000).bits(0, 2), 0x4);
        assert_eq!(EncodedInstruction(0xe0000000).bits(0, 2), 0x7);

        // |---------------------------xxxxx|
        assert_eq!(EncodedInstruction(0xaa55aa40).bits(27, 31), 0x00);
        assert_eq!(EncodedInstruction(0xaa55aa41).bits(27, 31), 0x01);
        assert_eq!(EncodedInstruction(0xaa55aa42).bits(27, 31), 0x02);
        assert_eq!(EncodedInstruction(0xaa55aa44).bits(27, 31), 0x04);
        assert_eq!(EncodedInstruction(0xaa55aa48).bits(27, 31), 0x08);
        assert_eq!(EncodedInstruction(0xaa55aa50).bits(27, 31), 0x10);
        assert_eq!(EncodedInstruction(0xaa55aa5f).bits(27, 31), 0x1f);
    }
}
