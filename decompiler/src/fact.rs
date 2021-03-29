use std::any::Any;
use std::fmt::Display;

pub mod basic_block;
pub mod basic_block_end;
pub mod branch_target;
pub mod parse_error;
pub mod subroutine;
pub mod subroutine_call;

pub trait Fact: Any + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn as_display(&self) -> Option<&dyn Display> {
        None
    }
}

pub trait DefaultFact: Fact {
    fn default() -> Box<Self>
    where
        Self: Sized;
}

#[deprecated]
pub trait ConstructibleFact: Fact {
    type Args;

    fn new(args: Self::Args) -> Box<Self>
    where
        Self: Sized;
}
