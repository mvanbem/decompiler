use std::fmt::{self, Display, Formatter};
use std::hash::Hash;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Variable<N: VariableName> {
    pub(crate) kind: VariableKind<N>,
    pub(crate) seq: usize,
}

impl<N: VariableName> Variable<N> {
    pub fn kind(&self) -> &VariableKind<N> {
        &self.kind
    }

    pub fn seq(&self) -> usize {
        self.seq
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum VariableKind<N: VariableName> {
    Named(N),
    Temporary,
}

pub trait VariableName: Clone + Display + Eq + Hash + PartialEq {}

impl<T> VariableName for T where T: Clone + Display + Eq + Hash + PartialEq {}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum NoNamedVariables {}

impl Display for NoNamedVariables {
    fn fmt(&self, _: &mut Formatter) -> fmt::Result {
        // This type is uninhabited.
        unreachable!()
    }
}
