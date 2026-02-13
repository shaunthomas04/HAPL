#[derive(Debug)]
pub enum Expr {
    Number(i64),

    Operation {
        op: Operator,
        operands: Vec<Expr>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}
