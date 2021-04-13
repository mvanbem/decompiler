use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

use powerpc::{cr_constants::*, Spr};
use powerpc::{gpr_constants::*, Gpr};
use powerpc::{ConditionBit, DecodedInstruction, Register};
use symbolic::ExprRef;

pub type Context = symbolic::NumberedContext<Variable>;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Variable {
    /// An unknown value. Referencing a garbage value means a program is violating the C ABI.
    Garbage,

    /// The initial value of a register on entering a basic block.
    RegisterEntering {
        basic_block_addr: u32,
        register: Register,
    },

    /// The final value of a register on leaving a basic block.
    RegisterLeaving {
        basic_block_addr: u32,
        register: Register,
    },

    /// The return value word (`r3`) as specified by the C ABI. If the called function does not
    /// return a value in this word, this is a synonym for `Garbage`.
    Return { call_addr: u32 },
    // TODO: There should also be an extended return word in r4 for any functions that need a second
    // word for the return value and aren't passed as a pointer.
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Variable::Garbage => write!(f, "garbage"),
            Variable::RegisterEntering {
                basic_block_addr,
                register,
            } => write!(f, "entering_0x{:08x}_{}", basic_block_addr, register),
            Variable::RegisterLeaving {
                basic_block_addr,
                register,
            } => write!(f, "leaving_0x{:08x}_{}", basic_block_addr, register),
            Variable::Return { call_addr } => {
                write!(f, "return_0x{:08x}", call_addr)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MemoryBlockRef(usize);

pub struct MachineState<'ctx> {
    ctx: &'ctx mut Context,
    basic_block: u32,

    registers: HashMap<Register, ExprRef>,
    // memory_blocks: Vec<HashMap<i32, ExprRef>>,
    // memory_pointers: HashMap<ExprRef, MemoryBlockRef>,
}

impl<'ctx> MachineState<'ctx> {
    pub fn new(ctx: &'ctx mut Context, basic_block: u32) -> Self {
        Self {
            ctx,
            basic_block,

            registers: HashMap::new(),
            // memory_blocks: Vec::new(),
            // memory_pointers: HashMap::new(),
        }
    }

    pub fn ctx(&self) -> &Context {
        &self.ctx
    }

    pub fn ctx_mut(&mut self) -> &mut Context {
        &mut self.ctx
    }

    pub fn get_register<T: Into<Register>>(&mut self, register: T) -> ExprRef {
        match register.into() {
            Register::Zero => self.ctx.literal_expr(0),
            register => {
                let current_basic_block = &self.basic_block;
                let ctx = &mut self.ctx;
                *self.registers.entry(register).or_insert_with(|| {
                    ctx.variable_expr(Variable::RegisterEntering {
                        basic_block_addr: *current_basic_block,
                        register,
                    })
                })
            }
        }
    }

    pub fn iter_registers(&self) -> impl Iterator<Item = (Register, ExprRef)> + '_ {
        self.registers
            .iter()
            .map(|(register, expr)| (*register, *expr))
    }

    // pub fn allocate_memory_block(&mut self) -> MemoryBlockRef {
    //     let index = self.memory_blocks.len();
    //     self.memory_blocks.push(HashMap::new());
    //     MemoryBlockRef(index)
    // }

    // pub fn set_memory_pointer(&mut self, base: ExprRef, memory_block: MemoryBlockRef) {
    //     assert!(self.ctx.is_variable(base));

    //     let replaced = self.memory_pointers.insert(base, memory_block);
    //     assert!(replaced.is_none());
    // }

    // fn get_memory_block(&mut self, base: ExprRef) -> MemoryBlockRef {
    //     if self.memory_pointers.contains_key(&base) {
    //         self.memory_pointers.get(&base).copied().unwrap()
    //     } else {
    //         let memory_block = self.allocate_memory_block();
    //         self.set_memory_pointer(base, memory_block);
    //         memory_block
    //     }
    // }

    // pub fn read_memory_base_offset(&mut self, base: ExprRef, offset: i32) -> ExprRef {
    //     assert!(self.ctx.is_variable(base));
    //     assert!(offset & 3 == 0, "unexpected unaligned memory read");

    //     let memory_block = self.get_memory_block(base);
    //     if self.memory_blocks[memory_block.0].contains_key(&offset) {
    //         self.memory_blocks[memory_block.0]
    //             .get(&offset)
    //             .copied()
    //             .unwrap()
    //     } else {
    //         let variable = self.ctx.next_numbered_variable_expr();
    //         self.memory_blocks[memory_block.0].insert(offset, variable);

    //         let offset_expr = self.ctx.literal_expr(offset as u32);
    //         let addr_expr = self.ctx.add_expr(vec![base, offset_expr]);
    //         let data_expr = self.ctx.read_expr(addr_expr);
    //         self.ctx.assign_variable(variable, data_expr);

    //         variable
    //     }
    // }

    // pub fn write_memory_base_offset(&mut self, base: ExprRef, offset: i32, data: ExprRef) {
    //     assert!(self.ctx.is_variable(base));
    //     assert!(offset & 3 == 0, "unexpected unaligned memory read");

    //     let memory_block = self.get_memory_block(base);
    //     self.memory_blocks[memory_block.0].insert(offset, data);
    //     let memory_block = &self.memory_blocks[memory_block.0];

    //     // Do some debug printing for now.
    //     println!(
    //         "* wrote memory at {}({}): {}",
    //         offset,
    //         self.ctx.display_expr(base),
    //         self.ctx.display_expr(data),
    //     );
    //     print!("* memory block contents: [");
    //     let addrs = {
    //         let mut addrs: Vec<i32> = memory_block.keys().copied().collect();
    //         addrs.sort_unstable();
    //         addrs
    //     };
    //     let mut first = true;
    //     for addr in addrs {
    //         if first {
    //             first = false;
    //         } else {
    //             print!(", ");
    //         }
    //         print!(
    //             "{}0x{:x}: {}",
    //             if addr < 0 { "-" } else { "" },
    //             addr.checked_abs().unwrap(),
    //             self.ctx.display_expr(*memory_block.get(&addr).unwrap()),
    //         );
    //     }
    //     println!("]");
    //     // End debug printing.
    // }

    // fn interpret_memory_access(&mut self, addr: ExprRef) -> Result<(ExprRef, i32), MemoryError> {
    //     let too_complicated = || MemoryError::TooComplicated {
    //         addr: format!("{}", self.ctx.display_expr(addr)),
    //     };

    //     match self.ctx.get_expr(addr) {
    //         // A literal is interpreted as an offset from a special absolute base.
    //         Expr::Literal(literal) => {
    //             let literal = *literal as i32;
    //             Ok((self.ctx.variable_expr(Variable::Absolute), literal))
    //         }

    //         // A variable is interpreted as a zero-offset reference from its own base.
    //         Expr::Variable(variable) => Ok((addr, 0)),

    //         Expr::Add(exprs) => {
    //             // There have to be precisely two expressions added together.
    //             if exprs.len() != 2 {
    //                 return Err(too_complicated());
    //             }

    //             // One of them has to be a variable.
    //             let register = exprs
    //                 .iter()
    //                 .copied()
    //                 .filter(|expr| self.ctx.is_variable(*expr))
    //                 .singleton()
    //                 .ok_or_else(too_complicated)?;

    //             // And one of them has to be a literal.
    //             let literal = exprs
    //                 .iter()
    //                 .copied()
    //                 .filter_map(|expr| {
    //                     if let Expr::Literal(literal) = self.ctx.get_expr(expr) {
    //                         return Some(*literal);
    //                     }
    //                     None
    //                 })
    //                 .singleton()
    //                 .ok_or_else(too_complicated)?;

    //             Ok((register, literal as i32))
    //         }

    //         // Anything else is deemed too complicated.
    //         _ => Err(too_complicated()),
    //     }
    // }

    // pub fn read_memory(&mut self, addr: ExprRef) -> Result<ExprRef, MemoryError> {
    //     let (base, offset) = self.interpret_memory_access(addr)?;
    //     Ok(self.read_memory_base_offset(base, offset))
    // }

    // pub fn write_memory(&mut self, write: &Write) -> Result<(), MemoryError> {
    //     assert_eq!(write.width, AccessWidth::Word);

    //     let (base, offset) = self.interpret_memory_access(write.addr)?;
    //     Ok(self.write_memory_base_offset(base, offset, write.data))
    // }

    pub fn apply(&mut self, update: Update) {
        self.registers.extend(update.registers);
        // for write in update.writes {
        //     match self.write_memory(&write) {
        //         Ok(()) => (),
        //         Err(e) => panic!("{}", e),
        //     }
        // }
    }

    pub fn prepare_update(&mut self, cia: u32, instruction: &DecodedInstruction) -> Update {
        match instruction {
            DecodedInstruction::Addi {
                dst,
                src,
                immediate,
            } => {
                let src_expr = self.get_register(*src);
                let immediate_expr = self.ctx.literal_expr(*immediate as u32);
                let add_expr = self.ctx.add_expr(vec![src_expr, immediate_expr]);
                Update::one_register(*dst, add_expr)
            }
            DecodedInstruction::Addis {
                dst,
                src,
                immediate,
            } => todo!(),
            DecodedInstruction::Addze {
                dst,
                src,
                overflow_enable,
                record,
            } => todo!(),
            DecodedInstruction::B { .. }
            | DecodedInstruction::Bc { .. }
            | DecodedInstruction::Bclr { .. } => {
                let branch_info = instruction.branch_info().unwrap();
                let mut update = Update::new();
                if branch_info.link {
                    // Function call. Clear all volatile registers.
                    let garbage = self.ctx.variable_expr(Variable::Garbage);
                    update.set_register(R0, garbage);
                    for gpr in 5..=12 {
                        update.set_register(Gpr::new(gpr).unwrap(), garbage);
                    }
                    for spr in [Spr::Link, Spr::Count, Spr::IntegerException]
                        .iter()
                        .copied()
                    {
                        update.set_register(spr, garbage);
                    }
                    for bit in (0..8).chain(20..32) {
                        update.set_register(ConditionBit::new(bit).unwrap(), garbage);
                    }
                    let return_var = self.ctx.variable_expr(Variable::Return { call_addr: cia });
                    update.set_register(R3, return_var);
                } else {
                    // Ignore local branches for now. These end basic blocks and should emit block
                    // linkage conditions.
                }
                update
            }
            DecodedInstruction::Cmpi {
                crf,
                src,
                immediate,
            } => todo!(),
            DecodedInstruction::Cmpl { crf, srcs } => todo!(),
            DecodedInstruction::Cmpli {
                crf,
                src,
                immediate,
            } => {
                let lhs_expr = self.get_register(*src);
                let rhs_expr = self.ctx.literal_expr(*immediate as u32);
                let lt_expr = self.ctx.less_unsigned_expr(lhs_expr, rhs_expr);
                let gt_expr = self.ctx.less_unsigned_expr(rhs_expr, lhs_expr);
                let eq_expr = self.ctx.equal_expr(lhs_expr, rhs_expr);
                let mut update = Update::new();
                update.set_register(ConditionBit::from_crf_and_condition(*crf, LT), lt_expr);
                update.set_register(ConditionBit::from_crf_and_condition(*crf, GT), gt_expr);
                update.set_register(ConditionBit::from_crf_and_condition(*crf, EQ), eq_expr);
                update
            }
            DecodedInstruction::Crxor { dst, srcs } => todo!(),
            DecodedInstruction::Lbz { dst, offset, base } => todo!(),
            DecodedInstruction::Lha { dst, offset, base } => todo!(),
            DecodedInstruction::Lwz { dst, offset, base } => {
                let offset_expr = self.ctx.literal_expr(*offset as u32);
                let base_expr = self.get_register(*base);
                let addr_expr = self.ctx.add_expr(vec![offset_expr, base_expr]);
                // TODO: Attach some kind of sequencing information for keeping sensitive memory
                // operations ordered.
                let read_variable = self.ctx.next_numbered_variable_expr();
                let data_expr = self.ctx.read_expr(addr_expr);
                self.ctx.assign_variable(read_variable, data_expr);
                Update::one_register(*dst, read_variable)
            }
            DecodedInstruction::Mfspr { spr, dst } => {
                Update::one_register(*dst, self.get_register(*spr))
            }
            DecodedInstruction::Mtspr { spr, src } => {
                Update::one_register(*spr, self.get_register(*src))
            }
            DecodedInstruction::Or { dst, srcs, record } => {
                let src0_expr = self.get_register(srcs[0]);
                let src1_expr = self.get_register(srcs[1]);
                let bit_or_expr = self.ctx.bit_or_expr(vec![src0_expr, src1_expr]);

                let mut update = Update::new();
                update.set_register(*dst, bit_or_expr);
                if *record {
                    let zero = self.ctx.literal_expr(0);
                    update.set_register(CR0LT, self.ctx.less_signed_expr(bit_or_expr, zero));
                    update.set_register(CR0GT, self.ctx.less_signed_expr(zero, bit_or_expr));
                    update.set_register(CR0EQ, self.ctx.equal_expr(bit_or_expr, zero));
                    // TODO: Support the SO bit.
                }
                update
            }
            DecodedInstruction::Rlwinm {
                dst,
                src,
                shift,
                mask_begin,
                mask_end,
                record,
            } => todo!(),
            DecodedInstruction::Srawi {
                dst,
                src,
                shift,
                record,
            } => todo!(),
            DecodedInstruction::Stmw { src, offset, base } => todo!(),
            DecodedInstruction::Stw { src, offset, base } => {
                let offset_expr = self.ctx.literal_expr(*offset as u32);
                let base_expr = self.get_register(*base);
                let addr_expr = self.ctx.add_expr(vec![offset_expr, base_expr]);
                let data_expr = self.get_register(*src);
                Update::one_write(AccessWidth::Word, addr_expr, data_expr)
            }
            DecodedInstruction::Stwu { src, offset, base } => {
                let offset_expr = self.ctx.literal_expr(*offset as u32);
                let base_expr = self.get_register(*base);
                let addr_expr = self.ctx.add_expr(vec![offset_expr, base_expr]);
                let data_expr = self.get_register(*src);

                let mut update = Update::new();
                update.set_register(*base, addr_expr);
                update.add_write(AccessWidth::Word, addr_expr, data_expr);
                update
            }
        }
    }
}

#[derive(Default)]
pub struct Update {
    pub registers: HashMap<Register, ExprRef>,
    pub writes: Vec<Write>,
}

impl Update {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_register<T: Into<Register>>(&mut self, register: T, data: ExprRef) {
        self.registers.insert(register.into(), data);
    }

    pub fn add_write(&mut self, width: AccessWidth, addr: ExprRef, data: ExprRef) {
        self.writes.push(Write { width, addr, data });
    }

    pub fn one_register<T: Into<Register>>(register: T, data: ExprRef) -> Self {
        Self {
            registers: [(register.into(), data)].iter().copied().collect(),
            writes: Vec::new(),
        }
    }

    pub fn one_write(width: AccessWidth, addr: ExprRef, data: ExprRef) -> Self {
        Self {
            registers: HashMap::new(),
            writes: vec![Write { width, addr, data }],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Write {
    pub width: AccessWidth,
    pub addr: ExprRef,
    pub data: ExprRef,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AccessWidth {
    Byte,
    Halfword,
    Word,
}

impl Display for AccessWidth {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            AccessWidth::Byte => write!(f, "b"),
            AccessWidth::Halfword => write!(f, "h"),
            AccessWidth::Word => write!(f, "w"),
        }
    }
}
