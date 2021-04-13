use std::fmt::{self, Debug, Display, Formatter};
use std::hash::Hash;

use crate::{Context, Expr, ExprRef};

pub struct DisplayExpr<'ctx, V> {
    pub(crate) ctx: &'ctx Context<V>,
    pub(crate) index: ExprRef,
}

impl<'ctx, V: Clone + Display + Eq + Hash> Debug for DisplayExpr<'ctx, V> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl<'ctx, V: Clone + Display + Eq + Hash> Display for DisplayExpr<'ctx, V> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let display_function = |f: &mut Formatter, name: &str, params: &[ExprRef]| {
            write!(f, "{}(", name)?;
            let mut first = true;
            for param in params {
                if first {
                    first = false;
                } else {
                    write!(f, ", ")?;
                }
                write!(
                    f,
                    "{}",
                    DisplayExpr {
                        ctx: self.ctx,
                        index: *param
                    }
                )?;
            }
            write!(f, ")")
        };

        match self.ctx.get_expr(self.index) {
            Expr::Literal(literal) => write!(f, "0x{:08x}", literal),
            Expr::Variable(variable) => write!(f, "{}", variable),
            Expr::Read(addr) => display_function(f, "read", &[*addr]),
            Expr::Phi(variables) => display_function(f, "phi", variables),
            Expr::Add(exprs) => display_function(f, "add", exprs),
            Expr::Mul(exprs) => display_function(f, "mul", exprs),
            Expr::BitOr(exprs) => display_function(f, "bit_or", exprs),
            Expr::BitAnd(exprs) => display_function(f, "bit_and", exprs),
            Expr::Not(expr) => display_function(f, "not", &[*expr]),
            Expr::Equal(lhs, rhs) => display_function(f, "equal", &[*lhs, *rhs]),
            Expr::LessSigned(lhs, rhs) => display_function(f, "less_i", &[*lhs, *rhs]),
            Expr::LessUnsigned(lhs, rhs) => display_function(f, "less_u", &[*lhs, *rhs]),
        }
    }
}
