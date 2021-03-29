use std::collections::{HashMap, HashSet};

use crate::{
    DisplayExpr, DisplayVariable, Expr, ExprRef, Variable, VariableKind, VariableName, VariableRef,
};

pub struct VariableAssignment<N: VariableName> {
    pub identity: Variable<N>,
    pub assignment: Option<ExprRef>,
}

pub struct Context<N: VariableName> {
    next_variable_seqs_by_kind: HashMap<VariableKind<N>, usize>,
    variables_by_index: Vec<VariableAssignment<N>>,

    exprs_by_index: Vec<Expr>,
    indices_by_expr: HashMap<Expr, usize>,
}

impl<N: VariableName> Context<N> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_variable(&self, index: VariableRef) -> &VariableAssignment<N> {
        &self.variables_by_index[index.0]
    }

    pub fn display_variable(&self, index: VariableRef) -> DisplayVariable<N> {
        DisplayVariable { ctx: self, index }
    }

    fn take_next_variable_seq(&mut self, kind: VariableKind<N>) -> usize {
        let next_seq = self.next_variable_seqs_by_kind.entry(kind).or_default();
        let seq = *next_seq;
        *next_seq += 1;
        seq
    }

    fn allocate_variable(
        &mut self,
        kind: VariableKind<N>,
        assignment: Option<ExprRef>,
    ) -> VariableRef {
        let variable = Variable {
            kind: kind.clone(),
            seq: self.take_next_variable_seq(kind),
        };
        let index = self.variables_by_index.len();
        self.variables_by_index.push(VariableAssignment {
            identity: variable,
            assignment,
        });
        VariableRef(index)
    }

    pub fn allocate_named_variable(&mut self, name: N, assignment: Option<ExprRef>) -> VariableRef {
        self.allocate_variable(VariableKind::Named(name), assignment)
    }

    pub fn allocate_anonymous_variable(&mut self, assignment: Option<ExprRef>) -> VariableRef {
        self.allocate_variable(VariableKind::Temporary, assignment)
    }

    pub fn get_expr(&self, index: ExprRef) -> &Expr {
        &self.exprs_by_index[index.0]
    }

    pub fn display_expr(&self, index: ExprRef) -> DisplayExpr<N> {
        DisplayExpr { ctx: self, index }
    }

    fn insert_unique_expr(&mut self, expr: Expr) -> ExprRef {
        let index = self.exprs_by_index.len();
        self.exprs_by_index.push(expr.clone());
        self.indices_by_expr.insert(expr, index);
        ExprRef(index)
    }

    fn intern_expr(&mut self, expr: Expr) -> ExprRef {
        match self.indices_by_expr.get(&expr).copied() {
            Some(index) => ExprRef(index),
            None => self.insert_unique_expr(expr),
        }
    }

    pub fn literal_expr(&mut self, literal: u32) -> ExprRef {
        self.intern_expr(Expr::Literal(literal))
    }

    pub fn variable_expr(&mut self, variable: VariableRef) -> ExprRef {
        self.intern_expr(Expr::Variable(variable))
    }

    pub fn add_expr(&mut self, exprs: Vec<ExprRef>) -> ExprRef {
        let mut todo = exprs;
        let mut irreducible_exprs = vec![];
        let mut literal_sum = 0u32;
        while let Some(expr) = todo.pop() {
            match self.get_expr(expr) {
                Expr::Literal(literal) => literal_sum = literal_sum.wrapping_add(*literal),
                Expr::Add(exprs) => todo.extend_from_slice(exprs),
                _ => irreducible_exprs.push(expr),
            }
        }
        if literal_sum != 0 {
            irreducible_exprs.push(self.literal_expr(literal_sum));
        }
        match irreducible_exprs.len() {
            // An empty sum is zero.
            0 => self.literal_expr(0),
            // A singleton sum is just the given term.
            1 => irreducible_exprs[0],
            _ => {
                // Normal form: terms are sorted by their indices.
                irreducible_exprs.sort_unstable_by_key(|expr| expr.0);
                self.intern_expr(Expr::Add(irreducible_exprs))
            }
        }
    }

    pub fn mul_expr(&mut self, exprs: Vec<ExprRef>) -> ExprRef {
        let mut todo = exprs;
        let mut irreducible_exprs = vec![];
        let mut literal_product = 1u32;
        while let Some(expr) = todo.pop() {
            match self.get_expr(expr) {
                Expr::Literal(literal) => literal_product = literal_product.wrapping_mul(*literal),
                Expr::Mul(exprs) => todo.extend_from_slice(exprs),
                _ => irreducible_exprs.push(expr),
            }
        }
        if literal_product == 0 {
            return self.literal_expr(0);
        } else if literal_product != 1 {
            irreducible_exprs.push(self.literal_expr(literal_product));
        }
        match irreducible_exprs.len() {
            // An empty product is one.
            0 => self.literal_expr(1),
            // A singleton product is just the given term.
            1 => irreducible_exprs[0],
            _ => {
                // Normal form: terms are sorted by their indices.
                irreducible_exprs.sort_unstable_by_key(|expr| expr.0);
                self.intern_expr(Expr::Mul(irreducible_exprs))
            }
        }
    }

    pub fn bit_or_expr(&mut self, exprs: Vec<ExprRef>) -> ExprRef {
        assert!(exprs.len() > 0);
        // Eliminate duplicates.
        let exprs = exprs.iter().copied().collect::<HashSet<ExprRef>>();
        if exprs.len() == 1 {
            // A singleton bit-or is just the given term.
            exprs.iter().copied().next().unwrap()
        } else {
            // Normal form: terms are sorted by their indices.
            let mut exprs: Vec<ExprRef> = exprs.iter().copied().collect();
            exprs.sort_unstable_by_key(|expr| expr.0);
            self.intern_expr(Expr::BitOr(exprs))
        }
    }

    pub fn bit_and_expr(&mut self, exprs: Vec<ExprRef>) -> ExprRef {
        assert!(exprs.len() > 0);
        // Eliminate duplicates.
        let exprs = exprs.iter().copied().collect::<HashSet<ExprRef>>();
        if exprs.len() == 1 {
            // A singleton bit-and is just the given term.
            exprs.iter().copied().next().unwrap()
        } else {
            // Normal form: terms are sorted by their indices.
            let mut exprs: Vec<ExprRef> = exprs.iter().copied().collect();
            exprs.sort_unstable_by_key(|expr| expr.0);
            self.intern_expr(Expr::BitAnd(exprs))
        }
    }

    pub fn not_expr(&mut self, expr: ExprRef) -> ExprRef {
        match self.get_expr(expr) {
            Expr::Literal(literal) => {
                let literal = *literal;
                self.intern_expr(Expr::Literal(!literal))
            }
            Expr::Not(expr) => *expr,
            _ => self.intern_expr(Expr::Not(expr)),
        }
    }

    pub fn equal_expr(&mut self, lhs: ExprRef, rhs: ExprRef) -> ExprRef {
        // Identical expressions are equal.
        if lhs == rhs {
            return self.literal_expr(1);
        }
        // Different literals are unequal.
        if let (Expr::Literal(_), Expr::Literal(_)) = (self.get_expr(lhs), self.get_expr(rhs)) {
            return self.literal_expr(0);
        }
        // Normal form: terms are sorted by their indices.
        if lhs.0 < rhs.0 {
            self.intern_expr(Expr::Equal(lhs, rhs))
        } else {
            self.intern_expr(Expr::Equal(rhs, lhs))
        }
    }

    pub fn less_signed_expr(&mut self, lhs: ExprRef, rhs: ExprRef) -> ExprRef {
        self.intern_expr(Expr::LessSigned(lhs, rhs))
    }
}

// NOTE: This cannot be `#[derive]`d because `N` is not necessarily `Default`.
impl<N: VariableName> Default for Context<N> {
    fn default() -> Self {
        Self {
            next_variable_seqs_by_kind: Default::default(),
            variables_by_index: Default::default(),
            exprs_by_index: Default::default(),
            indices_by_expr: Default::default(),
        }
    }
}
