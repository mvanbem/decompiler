use std::fmt::{self, Display, Formatter};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use crate::{Context, ExprRef};

/// A variable that is either a number or some other type of variable.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NumberedVariable<N> {
    Numbered(usize),
    Named(N),
}

impl<N> Display for NumberedVariable<N>
where
    N: Display,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            NumberedVariable::Numbered(number) => write!(f, "v_{}", number),
            NumberedVariable::Named(name) => write!(f, "{}", name),
        }
    }
}

/// A wrapper around [`Context`] that provides
/// [`next_numbered_variable_expr`](Self::next_numbered_variable_expr).
#[derive(Default)]
pub struct NumberedContext<N> {
    ctx: Context<NumberedVariable<N>>,
    next_variable_number: usize,
}

impl<N: Clone + Eq + Hash> NumberedContext<N> {
    pub fn new() -> Self {
        Self {
            ctx: Context::new(),
            next_variable_number: 0,
        }
    }

    pub fn variable_expr(&mut self, name: N) -> ExprRef {
        self.ctx.variable_expr(NumberedVariable::Named(name))
    }

    pub fn next_numbered_variable_expr(&mut self) -> ExprRef {
        let number = self.next_variable_number;
        self.next_variable_number += 1;
        self.ctx.variable_expr(NumberedVariable::Numbered(number))
    }
}

impl<N> Deref for NumberedContext<N> {
    type Target = Context<NumberedVariable<N>>;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl<N> DerefMut for NumberedContext<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}
