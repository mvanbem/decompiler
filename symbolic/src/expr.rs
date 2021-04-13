use crate::ExprRef;

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum Expr<V> {
    Literal(u32),
    Variable(V),
    Read(ExprRef),
    Phi(Vec<ExprRef>), // TODO: always a variable?
    Add(Vec<ExprRef>),
    Mul(Vec<ExprRef>),
    BitOr(Vec<ExprRef>),
    BitAnd(Vec<ExprRef>),
    Not(ExprRef),
    Equal(ExprRef, ExprRef),
    LessSigned(ExprRef, ExprRef),
    LessUnsigned(ExprRef, ExprRef),
}
