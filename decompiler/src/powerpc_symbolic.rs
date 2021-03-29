use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::ops::Range;

use powerpc::{cr_constants::*, DecodedInstruction, Register};
use symbolic::{Expr, ExprRef, VariableKind};
use thiserror::Error;

pub type Context = symbolic::Context<Register>;

#[derive(Default)]
pub struct MachineState {
    ctx: Context,
    registers: HashMap<Register, ExprRef>,
    memory_blocks: HashMap<Register, MemoryBlock>,
}

impl MachineState {
    pub fn new() -> Self {
        Self::default()
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
                let ctx = &mut self.ctx;
                *self.registers.entry(register).or_insert_with(|| {
                    let variable = ctx.allocate_named_variable(register, None);
                    ctx.variable_expr(variable)
                })
            }
        }
    }

    pub fn define_memory_block<T: Into<Register>>(
        &mut self,
        register: T,
        memory_block: MemoryBlock,
    ) {
        let replaced = self.memory_blocks.insert(register.into(), memory_block);
        assert!(replaced.is_none());
    }

    pub fn read_memory_base_offset<T: Into<Register>>(
        &self,
        register: T,
        offset: i32,
    ) -> Result<Option<ExprRef>, MemoryError> {
        let register = register.into();
        match self.memory_blocks.get(&register) {
            Some(memory_block) => {
                let offset_from_start = offset.wrapping_sub(memory_block.extent.start) as u32;
                assert!(
                    offset_from_start & 3 == 0,
                    "unexpected unaligned memory read",
                );

                if memory_block.extent.contains(&offset) {
                    let index = (offset_from_start >> 2) as usize;
                    Ok(memory_block.words[index])
                } else {
                    Err(MemoryError::OutOfBounds {
                        base: register,
                        extent: memory_block.extent.clone(),
                        offset,
                    })
                }
            }
            None => Err(MemoryError::Unmapped {
                base: register,
                offset,
            }),
        }
    }

    pub fn write_memory_base_offset<T: Into<Register>>(
        &mut self,
        register: T,
        offset: i32,
        data: ExprRef,
    ) -> Result<(), MemoryError> {
        let register = register.into();
        match self.memory_blocks.get_mut(&register) {
            Some(memory_block) => {
                let offset_from_start = offset.wrapping_sub(memory_block.extent.start) as u32;
                assert!(
                    offset_from_start & 3 == 0,
                    "unexpected unaligned memory write",
                );

                if memory_block.extent.contains(&offset) {
                    let index = (offset_from_start >> 2) as usize;
                    memory_block.words[index] = Some(data);

                    println!(
                        "* wrote memory at {}({}): {}",
                        offset,
                        register,
                        self.ctx.display_expr(data),
                    );
                    print!("* memory block contents: [");
                    let mut first = true;
                    for word in &memory_block.words {
                        if first {
                            first = false;
                        } else {
                            print!(", ");
                        }
                        match word {
                            Some(expr) => print!("{}", self.ctx.display_expr(*expr)),
                            None => print!("_"),
                        }
                    }
                    println!("]");

                    Ok(())
                } else {
                    Err(MemoryError::OutOfBounds {
                        base: register,
                        extent: memory_block.extent.clone(),
                        offset,
                    })
                }
            }
            None => Err(MemoryError::Unmapped {
                base: register,
                offset,
            }),
        }
    }

    fn interpret_memory_access(&self, addr: ExprRef) -> Result<(Register, i32), MemoryError> {
        let too_complicated = || MemoryError::TooComplicated {
            addr: format!("{}", self.ctx.display_expr(addr)),
        };

        match self.ctx.get_expr(addr) {
            // A literal is interpreted as an offset from the zero register.
            Expr::Literal(literal) => Ok((Register::Zero, *literal as i32)),

            Expr::Variable(variable) => {
                match self.ctx.get_variable(*variable).identity.kind() {
                    // A named variable is interpreted as a zero-offset reference from the
                    // corresponding register.
                    VariableKind::Named(register) => Ok((*register, 0)),

                    // A temporary variable is not recursed into.
                    VariableKind::Temporary => Err(too_complicated()),
                }
            }

            Expr::Add(exprs) => {
                // There have to be precisely two expressions added together.
                if exprs.len() != 2 {
                    return Err(too_complicated());
                }

                // One of them has to be a named variable reference.
                let register = exprs
                    .iter()
                    .copied()
                    .filter_map(|expr| {
                        if let Expr::Variable(variable) = self.ctx.get_expr(expr) {
                            if let VariableKind::Named(register) =
                                self.ctx.get_variable(*variable).identity.kind()
                            {
                                return Some(*register);
                            }
                        }
                        None
                    })
                    .singleton()
                    .ok_or_else(too_complicated)?;

                // And one of them has to be a literal.
                let literal = exprs
                    .iter()
                    .copied()
                    .filter_map(|expr| {
                        if let Expr::Literal(literal) = self.ctx.get_expr(expr) {
                            return Some(*literal);
                        }
                        None
                    })
                    .singleton()
                    .ok_or_else(too_complicated)?;

                Ok((register, literal as i32))
            }

            // Anything else is deemed too complicated.
            _ => Err(too_complicated()),
        }
    }

    pub fn read_memory(&self, addr: ExprRef) -> Result<Option<ExprRef>, MemoryError> {
        let (register, offset) = self.interpret_memory_access(addr)?;
        self.read_memory_base_offset(register, offset)
    }

    pub fn write_memory(&mut self, write: &Write) -> Result<(), MemoryError> {
        assert_eq!(write.width, AccessWidth::Word);

        let (register, offset) = self.interpret_memory_access(write.addr)?;
        self.write_memory_base_offset(register, offset, write.data)
    }

    pub fn apply(&mut self, update: Update) {
        self.registers.extend(update.registers);
        for write in update.writes {
            match self.write_memory(&write) {
                Ok(()) => (),
                Err(e) => panic!("{}", e),
            }
        }
    }

    pub fn prepare_update(&mut self, instruction: &DecodedInstruction) -> Update {
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
            DecodedInstruction::B {
                link,
                absolute,
                target,
            } => {
                // TODO: I have no idea what branch instructions should do here.
                Update::default()
            }
            DecodedInstruction::Bc {
                condition,
                ctr,
                link,
                absolute,
                target,
            } => {
                // TODO: I have no idea what branch instructions should do here.
                Update::default()
            }
            DecodedInstruction::Bclr {
                condition,
                ctr,
                link,
            } => {
                // TODO: I have no idea what branch instructions should do here.
                Update::default()
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
            } => todo!(),
            DecodedInstruction::Crxor { dst, srcs } => todo!(),
            DecodedInstruction::Lbz { dst, offset, base } => todo!(),
            DecodedInstruction::Lha { dst, offset, base } => todo!(),
            DecodedInstruction::Lwz { dst, offset, base } => {
                let offset_expr = self.ctx.literal_expr(*offset as u32);
                let base_expr = self.get_register(*base);
                let addr_expr = self.ctx.add_expr(vec![offset_expr, base_expr]);
                let data_expr = self.read_memory(addr_expr).unwrap().unwrap();
                Update::one_register(*dst, data_expr)
            }
            DecodedInstruction::Mfspr { spr, dst } => {
                Update::one_register(*dst, self.get_register(*spr))
            }
            DecodedInstruction::Mtspr { spr, src } => todo!(),
            DecodedInstruction::Or { dst, srcs, record } => {
                let src0_expr = self.get_register(srcs[0]);
                let src1_expr = self.get_register(srcs[1]);
                let bit_or_expr = self.ctx.bit_or_expr(vec![src0_expr, src1_expr]);

                let mut update = Update::new();
                update.set_register(*dst, bit_or_expr);
                if *record {
                    let zero = self.ctx.literal_expr(0);
                    update.set_register(LT, self.ctx.less_signed_expr(bit_or_expr, zero));
                    update.set_register(GT, self.ctx.less_signed_expr(zero, bit_or_expr));
                    update.set_register(EQ, self.ctx.equal_expr(bit_or_expr, zero));
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

pub struct MemoryBlock {
    extent: Range<i32>,
    words: Vec<Option<ExprRef>>,
}

impl MemoryBlock {
    pub fn new(extent: Range<i32>) -> Self {
        assert_eq!(extent.start & 3, 0);
        assert_eq!(extent.end & 3, 0);
        let span = extent.end.wrapping_sub(extent.start) as u32;
        let len = (span / 4) as usize;
        let words = vec![None; len];

        Self { extent, words }
    }
}

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error(
        "memory access out of bounds: base = {base}, extent = 0x{start:08x}..0x{end:08x}, offset = {offset}",
        start = .extent.start,
        end = .extent.end,
    )]
    OutOfBounds {
        base: Register,
        extent: Range<i32>,
        offset: i32,
    },

    #[error("memory access relative to unmapped register: base = {base}, offset = {offset}")]
    Unmapped { base: Register, offset: i32 },

    #[error("memory access too complicated: {addr}")]
    TooComplicated { addr: String },
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

trait IteratorExt: Iterator {
    /// Consumes the iterator, returning an element if there is precisely one, or `None` otherwise.
    fn singleton(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let element = self.next()?;
        match self.next() {
            Some(_) => None,
            None => Some(element),
        }
    }
}

impl<T: Iterator> IteratorExt for T {}
