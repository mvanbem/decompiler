use std::fmt::{self, Debug, Display, Formatter};

use crate::{Context, Expr, ExprRef, VariableName};

pub struct DisplayExpr<'ctx, N: VariableName> {
    pub(crate) ctx: &'ctx Context<N>,
    pub(crate) index: ExprRef,
}

impl<'ctx, N: VariableName> Debug for DisplayExpr<'ctx, N> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl<'ctx, N: VariableName> Display for DisplayExpr<'ctx, N> {
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
            Expr::Variable(variable) => write!(f, "{}", self.ctx.display_variable(*variable)),
            Expr::Add(exprs) => display_function(f, "add", exprs),
            Expr::Mul(exprs) => display_function(f, "mul", exprs),
            Expr::BitOr(exprs) => display_function(f, "bit_or", exprs),
            Expr::BitAnd(exprs) => display_function(f, "bit_and", exprs),
            Expr::Not(expr) => display_function(f, "not", &[*expr]),
            Expr::Equal(lhs, rhs) => display_function(f, "equal", &[*lhs, *rhs]),
            Expr::LessSigned(lhs, rhs) => display_function(f, "less", &[*lhs, *rhs]),
        }
    }
}
