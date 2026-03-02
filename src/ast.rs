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
    /// <var id="x">...</var>
    Assignment {
        name: String,
        value: Box<Expr>
    },

    /// While loop:
    /// <div class="while">...</div>
    WhileLoop {
        condition: Box<Expr>,
        body: Vec<Expr>,
    },

    /// For loop:
    /// <div class="for">...</div>
    ForLoop {
        iterator: String,
        condition: Box<Expr>,
        increment: Box<Expr>,
        body: Vec<Expr>,
    },

    /// Function declaration:
    /// <div class="integer-function" id="add">...</div>
    FunctionDeclaration {
        name: String,
        params: Vec<(String, StaticType)>,
        return_type: StaticType,
        body: Vec<Expr>,
    },

    /// Function call:
    /// <div class="add">...</div>
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },

    /// Return statement:
    /// <div class="return">...</div>
    Return {
        value: Option<Box<Expr>>,
    },

    /// List declaration:
    /// <div class="integer-list" id="numbers">
    ///     <span class="integer">1</span>
    ///     <span class="integer">2</span>
    /// </div>
    ListDeclaration {
        name:      String,
        elem_type: StaticType,
        elements:  Vec<Expr>,
    },

    /// List index access (read):
    /// <div class="index">
    ///     <var class="numbers"></var>
    ///     <span class="integer">0</span>
    /// </div>
    ListAccess {
        list:  Box<Expr>,   // evaluates to a LiteralValue::List
        index: Box<Expr>,   // must evaluate to Integer
    },

    /// List index assignment (write):
    /// <div class="index-assign">
    ///     <var class="numbers"></var>
    ///     <span class="integer">0</span>
    ///     <span class="integer">99</span>
    /// </div>
    ListAssign {
        name:  String,      // variable name of the list
        index: Box<Expr>,   // must evaluate to Integer
        value: Box<Expr>,   // must match the list's element type
    },

    /// Push a value onto the end of a list:
    /// <div class="push">
    ///     <var class="numbers"></var>
    ///     <span class="integer">4</span>
    /// </div>
    ListPush {
        name:  String,      // variable name of the list
        value: Box<Expr>,   // must match the list's element type
    },

    /// Pop the last value off a list:
    /// <div class="pop">
    ///     <var class="numbers"></var>
    /// </div>
    ListPop {
        name: String,       // variable name of the list
    },
}

/// Represents a single `if` or `elif` block
#[derive(Debug, Clone)]
pub struct ConditionalBlock {
    pub condition:  Expr,       // boolean expression
    pub statements: Vec<Expr>,  // statements to execute if true
}

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,

    // Logical
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
    Boolean(bool),
    /// A typed, ordered, dynamically-sized list.
    /// `elem_type` enforces that every element matches the declared type.
    List {
        elem_type: StaticType,
        elements:  Vec<LiteralValue>,
    },
}

impl LiteralValue {
    pub fn get_type(&self) -> StaticType {
        match self {
            LiteralValue::Integer(_)       => StaticType::Integer,
            LiteralValue::Double(_)        => StaticType::Double,
            LiteralValue::String(_)        => StaticType::String,
            LiteralValue::Boolean(_)       => StaticType::Boolean,
            LiteralValue::List { elem_type, .. } => StaticType::List(Box::new(elem_type.clone())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StaticType {
    Integer,
    Double,
    String,
    Boolean,
    Void,
    /// A list whose elements are all of type `elem_type`.
    List(Box<StaticType>),
}

// Convenience — lets you write StaticType::list_of(StaticType::Integer) etc.
impl StaticType {
    pub fn list_of(elem_type: StaticType) -> Self {
        StaticType::List(Box::new(elem_type))
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Double(f64),
    String(String),
    Boolean(bool),
    Void,
}