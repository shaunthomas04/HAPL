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

    /// Conditional (if / elif / else)
    Conditional {
        if_blocks: Vec<ConditionalBlock>, // first one is the "if", rest are "elif"
        else_block: Option<Vec<Expr>>,    // optional else statements
    },

    /// Variable reassignment:
    /// <set id="x">...</set>
    Assignment {
        name: String,
        value: Box<Expr>
    },

    /// While loop:
    /// <while>...</while>
    WhileLoop {
        condition: Box<Expr>,
        body: Vec<Expr>,
    },

    ForLoop {
        iterator: String,
        condition: Box<Expr>,
        increment: Box<Expr>,   // required
        body: Vec<Expr>,
    }
}

/// Represents a single `if` or `elif` block
#[derive(Debug, Clone)]
pub struct ConditionalBlock {
    pub condition: Expr,      // boolean expression
    pub statements: Vec<Expr>, // statements to execute if true
}

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    //arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    
    //logical
    Or,
    And,
    Not,

    // Comparison
    Equal,          // ==
    NotEqual,       // !=
    Less,           // <
    LessEqual,      // <=
    Greater,        // >
    GreaterEqual,   // >=
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
