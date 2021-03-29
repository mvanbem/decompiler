type Context = crate::Context<crate::NoNamedVariables>;

#[test]
fn literal_dedupes() {
    let mut ctx = Context::new();
    let five_a = ctx.literal_expr(5);
    let five_b = ctx.literal_expr(5);
    assert_eq!(five_a, five_b);
}

#[test]
fn symbol_dedupes() {
    let mut ctx = Context::new();
    let vx = ctx.allocate_anonymous_variable(None);
    let x_a = ctx.variable_expr(vx);
    let x_b = ctx.variable_expr(vx);
    assert_eq!(x_a, x_b);
}

#[test]
fn double_not_cancels() {
    let mut ctx = Context::new();
    let vx = ctx.allocate_anonymous_variable(None);
    let x = ctx.variable_expr(vx);
    let not_x = ctx.not_expr(x);
    let not_not_x = ctx.not_expr(not_x);
    assert_eq!(x, not_not_x);
}

#[test]
fn not_literal() {
    let mut ctx = Context::new();
    let literal = ctx.literal_expr(0x87654321);
    let inv_literal = ctx.literal_expr(0x789abcde);
    let not_literal = ctx.not_expr(literal);
    assert_eq!(inv_literal, not_literal);
}

#[test]
fn add_none_is_zero() {
    let mut ctx = Context::new();
    let zero = ctx.literal_expr(0);
    let sum = ctx.add_expr(vec![]);
    assert_eq!(zero, sum);
}

#[test]
fn add_singleton_reduces() {
    let mut ctx = Context::new();
    let vx = ctx.allocate_anonymous_variable(None);
    let x = ctx.variable_expr(vx);
    let sum = ctx.add_expr(vec![x]);
    assert_eq!(x, sum);
}

#[test]
fn add_is_order_independent() {
    let mut ctx = Context::new();
    let vx = ctx.allocate_anonymous_variable(None);
    let vy = ctx.allocate_anonymous_variable(None);
    let x = ctx.variable_expr(vx);
    let y = ctx.variable_expr(vy);
    let sum_a = ctx.add_expr(vec![x, y]);
    let sum_b = ctx.add_expr(vec![y, x]);
    assert_eq!(sum_a, sum_b);
}

#[test]
fn add_folds_literals() {
    let mut ctx = Context::new();
    let vx = ctx.allocate_anonymous_variable(None);
    let vy = ctx.allocate_anonymous_variable(None);
    let x = ctx.variable_expr(vx);
    let y = ctx.variable_expr(vy);
    let three = ctx.literal_expr(3);
    let five = ctx.literal_expr(5);
    let eight = ctx.literal_expr(8);
    let sum_a = ctx.add_expr(vec![x, y, three, five]);
    let sum_b = ctx.add_expr(vec![x, y, eight]);
    assert_eq!(sum_a, sum_b);
}

#[test]
fn add_drops_zero() {
    let mut ctx = Context::new();
    let vx = ctx.allocate_anonymous_variable(None);
    let x = ctx.variable_expr(vx);
    let zero = ctx.literal_expr(0);
    let sum = ctx.add_expr(vec![x, zero]);
    assert_eq!(x, sum);
}

#[test]
fn add_flattens() {
    let mut ctx = Context::new();
    let vx = ctx.allocate_anonymous_variable(None);
    let vy = ctx.allocate_anonymous_variable(None);
    let vz = ctx.allocate_anonymous_variable(None);
    let x = ctx.variable_expr(vx);
    let y = ctx.variable_expr(vy);
    let z = ctx.variable_expr(vz);
    let xy = ctx.add_expr(vec![x, y]);
    let xyz_a = ctx.add_expr(vec![xy, z]);
    let xyz_b = ctx.add_expr(vec![x, y, z]);
    assert_eq!(xyz_a, xyz_b);
}

#[test]
fn mul_none_is_one() {
    let mut ctx = Context::new();
    let one = ctx.literal_expr(1);
    let product = ctx.mul_expr(vec![]);
    assert_eq!(one, product);
}

#[test]
fn mul_singleton_reduces() {
    let mut ctx = Context::new();
    let vx = ctx.allocate_anonymous_variable(None);
    let x = ctx.variable_expr(vx);
    let product = ctx.mul_expr(vec![x]);
    assert_eq!(x, product);
}

#[test]
fn mul_is_order_independent() {
    let mut ctx = Context::new();
    let vx = ctx.allocate_anonymous_variable(None);
    let vy = ctx.allocate_anonymous_variable(None);
    let x = ctx.variable_expr(vx);
    let y = ctx.variable_expr(vy);
    let product_a = ctx.mul_expr(vec![x, y]);
    let product_b = ctx.mul_expr(vec![y, x]);
    assert_eq!(product_a, product_b);
}

#[test]
fn mul_folds_literals() {
    let mut ctx = Context::new();
    let vx = ctx.allocate_anonymous_variable(None);
    let vy = ctx.allocate_anonymous_variable(None);
    let x = ctx.variable_expr(vx);
    let y = ctx.variable_expr(vy);
    let three = ctx.literal_expr(3);
    let five = ctx.literal_expr(5);
    let fifteen = ctx.literal_expr(15);
    let product_a = ctx.mul_expr(vec![x, y, three, five]);
    let product_b = ctx.mul_expr(vec![x, y, fifteen]);
    assert_eq!(product_a, product_b);
}

#[test]
fn mul_drops_one() {
    let mut ctx = Context::new();
    let vx = ctx.allocate_anonymous_variable(None);
    let x = ctx.variable_expr(vx);
    let one = ctx.literal_expr(1);
    let product = ctx.mul_expr(vec![x, one]);
    assert_eq!(x, product);
}

#[test]
fn mul_zero_dominates() {
    let mut ctx = Context::new();
    let vx = ctx.allocate_anonymous_variable(None);
    let x = ctx.variable_expr(vx);
    let zero = ctx.literal_expr(0);
    let product = ctx.mul_expr(vec![x, zero]);
    assert_eq!(zero, product);
}

#[test]
fn mul_flattens() {
    let mut ctx = Context::new();
    let vx = ctx.allocate_anonymous_variable(None);
    let vy = ctx.allocate_anonymous_variable(None);
    let vz = ctx.allocate_anonymous_variable(None);
    let x = ctx.variable_expr(vx);
    let y = ctx.variable_expr(vy);
    let z = ctx.variable_expr(vz);
    let xy = ctx.mul_expr(vec![x, y]);
    let xyz_a = ctx.mul_expr(vec![xy, z]);
    let xyz_b = ctx.mul_expr(vec![x, y, z]);
    assert_eq!(xyz_a, xyz_b);
}
