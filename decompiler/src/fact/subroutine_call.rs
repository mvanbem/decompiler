use std::any::Any;

use crate::fact::Fact;

#[derive(Default, Debug)]
pub struct SubroutineCallFact {
    target: u32,
}

impl SubroutineCallFact {
    pub fn new(target: u32) -> Self {
        Self { target }
    }
}

impl Fact for SubroutineCallFact {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
