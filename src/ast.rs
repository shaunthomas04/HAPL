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

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Integer(i64),
    Double(f64),
    String(String),
}