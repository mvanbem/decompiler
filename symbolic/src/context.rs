use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;

use crate::{DisplayExpr, Expr, ExprRef};

pub struct Context<V> {
    exprs_by_index: Vec<Expr<V>>,
    indices_by_expr: HashMap<Expr<V>, usize>,
    variable_assignments: HashMap<ExprRef, ExprRef>,
}

impl<V: Clone + Eq + Hash> Context<V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_variable(&self, variable: ExprRef) -> bool {
        if let Expr::Variable(_) = self.exprs_by_index[variable.0] {
            true
        } else {
            false
        }
    }

    pub fn get_variable_assignment(&self, variable: ExprRef) -> Option<ExprRef> {
        assert!(self.is_variable(variable));
        self.variable_assignments.get(&variable).copied()
    }

    pub fn assign_variable(&mut self, variable: ExprRef, assignment: ExprRef) {
        assert!(self.is_variable(variable));
        self.variable_assignments.insert(variable, assignment);
    }

    pub fn iter_variables(&self) -> impl Iterator<Item = (ExprRef, &V)> + '_ {
        self.exprs_by_index
            .iter()
            .enumerate()
            .filter_map(|(index, expr)| match expr {
                Expr::Variable(variable) => Some((ExprRef(index), variable)),
                _ => None,
            })
    }

    pub fn get_expr(&self, index: ExprRef) -> &Expr<V> {
        &self.exprs_by_index[index.0]
    }

    pub fn display_expr(&self, index: ExprRef) -> DisplayExpr<V>
    where
        V: Display,
    {
        DisplayExpr { ctx: self, index }
    }

    fn insert_unique_expr(&mut self, expr: Expr<V>) -> ExprRef {
        let index = self.exprs_by_index.len();
        self.exprs_by_index.push(expr.clone());
        self.indices_by_expr.insert(expr, index);
        ExprRef(index)
    }

    fn intern_expr(&mut self, expr: Expr<V>) -> ExprRef {
        match self.indices_by_expr.get(&expr).copied() {
            Some(index) => ExprRef(index),
            None => self.insert_unique_expr(expr),
        }
    }

    pub fn literal_expr(&mut self, literal: u32) -> ExprRef {
        self.intern_expr(Expr::Literal(literal))
    }

    pub fn variable_expr(&mut self, variable: V) -> ExprRef {
        self.intern_expr(Expr::Variable(variable))
    }

    pub fn read_expr(&mut self, addr: ExprRef) -> ExprRef {
        self.intern_expr(Expr::Read(addr))
    }

    pub fn phi_expr(&mut self, variables: Vec<ExprRef>) -> ExprRef {
        let mut todo = variables;
        let mut variables = Vec::new();
        while let Some(expr) = todo.pop() {
            match self.get_expr(expr) {
                Expr::Phi(params) => variables.extend_from_slice(params),
                _ => variables.push(expr),
            }
        }
        variables.sort_unstable_by_key(|expr| expr.0);
        self.intern_expr(Expr::Phi(variables))
    }

    pub fn add_expr(&mut self, exprs: Vec<ExprRef>) -> ExprRef {
        let mut todo = exprs;
        let mut irreducible_exprs = Vec::new();
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
        let mut irreducible_exprs = Vec::new();
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
        // Identical expressions are not less than each other.
        if lhs == rhs {
            return self.literal_expr(0);
        }
        // Compare literals.
        if let (Expr::Literal(lhs), Expr::Literal(rhs)) = (self.get_expr(lhs), self.get_expr(rhs)) {
            let lhs = (*lhs) as i32;
            let rhs = (*rhs) as i32;
            return self.literal_expr(if lhs < rhs { 1 } else { 0 });
        }
        self.intern_expr(Expr::LessSigned(lhs, rhs))
    }

    pub fn less_unsigned_expr(&mut self, lhs: ExprRef, rhs: ExprRef) -> ExprRef {
        // Identical expressions are not less than each other.
        if lhs == rhs {
            return self.literal_expr(0);
        }
        // Compare literals.
        if let (Expr::Literal(lhs), Expr::Literal(rhs)) = (self.get_expr(lhs), self.get_expr(rhs)) {
            let lhs = *lhs;
            let rhs = *rhs;
            return self.literal_expr(if lhs < rhs { 1 } else { 0 });
        }
        self.intern_expr(Expr::LessUnsigned(lhs, rhs))
    }

    pub fn get_expr_leaves(&self, expr: ExprRef) -> Vec<ExprRef> {
        match self.get_expr(expr) {
            Expr::Literal(_) | Expr::Variable(_) => Vec::new(),
            Expr::Read(param) | Expr::Not(param) => vec![*param],
            Expr::Phi(params)
            | Expr::Add(params)
            | Expr::Mul(params)
            | Expr::BitOr(params)
            | Expr::BitAnd(params) => params.clone(),
            Expr::Equal(lhs, rhs) | Expr::LessSigned(lhs, rhs) | Expr::LessUnsigned(lhs, rhs) => {
                vec![*lhs, *rhs]
            }
        }
    }

    /// Note that this function recurses into the mapped expression.
    pub fn map_leaves<F>(&mut self, expr: ExprRef, f: &F) -> ExprRef
    where
        F: for<'r> Fn(&'r mut Self, ExprRef) -> ExprRef,
    {
        // First, map the expression if it's a leaf.
        let expr = match self.get_expr(expr) {
            Expr::Literal(_) | Expr::Variable(_) => f(self, expr),
            _ => expr,
        };

        // Second, even if it was just mapped, recurse into its subexpressions.
        match self.get_expr(expr) {
            Expr::Literal(_) | Expr::Variable(_) => {
                // Already mapped once, don't map it again.
                expr
            }

            // Recursive expression types.
            Expr::Read(addr) => {
                let addr = *addr;
                let addr = self.map_leaves(addr, f);
                self.read_expr(addr)
            }
            Expr::Phi(exprs) => {
                let exprs = exprs.clone();
                let exprs = exprs
                    .into_iter()
                    .map(|term| self.map_leaves(term, f))
                    .collect();
                self.phi_expr(exprs)
            }
            Expr::Add(exprs) => {
                let exprs = exprs.clone();
                let exprs = exprs
                    .into_iter()
                    .map(|expr| self.map_leaves(expr, f))
                    .collect();
                self.add_expr(exprs)
            }
            Expr::Mul(exprs) => {
                let exprs = exprs.clone();
                let exprs = exprs
                    .into_iter()
                    .map(|expr| self.map_leaves(expr, f))
                    .collect();
                self.mul_expr(exprs)
            }
            Expr::BitOr(exprs) => {
                let exprs = exprs.clone();
                let exprs = exprs
                    .into_iter()
                    .map(|expr| self.map_leaves(expr, f))
                    .collect();
                self.bit_or_expr(exprs)
            }
            Expr::BitAnd(exprs) => {
                let exprs = exprs.clone();
                let exprs = exprs
                    .into_iter()
                    .map(|expr| self.map_leaves(expr, f))
                    .collect();
                self.bit_and_expr(exprs)
            }
            Expr::Not(expr) => {
                let expr = *expr;
                let expr = self.map_leaves(expr, f);
                self.read_expr(expr)
            }
            Expr::Equal(lhs, rhs) => {
                let lhs = *lhs;
                let rhs = *rhs;
                let lhs = self.map_leaves(lhs, f);
                let rhs = self.map_leaves(rhs, f);
                self.equal_expr(lhs, rhs)
            }
            Expr::LessSigned(lhs, rhs) => {
                let lhs = *lhs;
                let rhs = *rhs;
                let lhs = self.map_leaves(lhs, f);
                let rhs = self.map_leaves(rhs, f);
                self.less_signed_expr(lhs, rhs)
            }
            Expr::LessUnsigned(lhs, rhs) => {
                let lhs = *lhs;
                let rhs = *rhs;
                let lhs = self.map_leaves(lhs, f);
                let rhs = self.map_leaves(rhs, f);
                self.less_unsigned_expr(lhs, rhs)
            }
        }
    }
}

// NOTE: This cannot be `#[derive]`d because `V` is not necessarily `Default`.
impl<V> Default for Context<V> {
    fn default() -> Self {
        Self {
            exprs_by_index: Default::default(),
            indices_by_expr: Default::default(),
            variable_assignments: Default::default(),
        }
    }
}
