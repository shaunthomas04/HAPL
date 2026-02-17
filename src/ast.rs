// ast.rs

#[derive(Debug, Clone)]
pub enum Expr {
    /// Literal value
    Literal(LiteralValue),

    /// Arithmetic operation
    Operation {
        op: Operator,
        operands: Vec<Expr>,
    },

    /// Variable declaration:
    /// <var class="integer" id="x">...</var>
    VariableDeclaration {
        name: String,
        var_type: StaticType,
        value: Box<Expr>,
    },

    /// Variable reference:
    /// <var class="x"></var>
    VariableReference {
        name: String,
    },

    /// Print Operation
    Print { value: Box<Expr> },
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
    Boolean(bool)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StaticType {
    Integer,
    Double,
    String,
    Boolean
}
