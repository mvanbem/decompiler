use std::fmt::{self, Debug, Display, Formatter};

use crate::{Context, VariableKind, VariableName, VariableRef};

pub struct DisplayVariable<'ctx, N: VariableName> {
    pub(crate) ctx: &'ctx Context<N>,
    pub(crate) index: VariableRef,
}

impl<'ctx, N: VariableName> Debug for DisplayVariable<'ctx, N> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl<'ctx, N: VariableName> Display for DisplayVariable<'ctx, N> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let variable = &self.ctx.get_variable(self.index).identity;
        match &variable.kind {
            VariableKind::Named(name) => write!(f, "%{}.{}", name, variable.seq),
            VariableKind::Temporary => write!(f, "%t.{}", variable.seq),
        }
    }
}
