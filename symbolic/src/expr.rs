use crate::{ExprRef, VariableRef};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum Expr {
    Literal(u32),
    Variable(VariableRef),
    Add(Vec<ExprRef>),
    Mul(Vec<ExprRef>),
    BitOr(Vec<ExprRef>),
    BitAnd(Vec<ExprRef>),
    Not(ExprRef),
    Equal(ExprRef, ExprRef),
    LessSigned(ExprRef, ExprRef),
}
