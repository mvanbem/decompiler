use std::any::Any;
use std::fmt::{self, Display, Formatter};

use crate::fact::Fact;

/// This address is called as a subroutine. It might be a good candidate for a C function.
#[derive(Default, Debug)]
pub struct SubroutineFact;

impl Fact for SubroutineFact {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_display(&self) -> Option<&dyn Display> {
        Some(self)
    }
}

impl Display for SubroutineFact {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "#[subroutine]")
    }
}
