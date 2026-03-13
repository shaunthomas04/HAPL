use indexmap::IndexMap;

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
        if_blocks: Vec<ConditionalBlock>,
        else_block: Option<Vec<Expr>>,
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
    ///     <div class="key"><span class="integer">0</span></div>
    /// </div>
    ListAccess {
        list:  Box<Expr>,
        index: Box<Expr>,
    },

    /// List index assignment (write):
    /// <div class="index-assign">
    ///     <var class="numbers"></var>
    ///     <div class="key"><span class="integer">0</span></div>
    ///     <span class="integer">99</span>
    /// </div>
    ListAssign {
        name:  String,
        index: Box<Expr>,
        value: Box<Expr>,
    },

    /// Push a value onto the end of a list:
    /// <div class="push">
    ///     <var class="numbers"></var>
    ///     <span class="integer">4</span>
    /// </div>
    ListPush {
        name:  String,
        value: Box<Expr>,
    },

    /// Pop the last value off a list:
    /// <div class="pop">
    ///     <var class="numbers"></var>
    /// </div>
    ListPop {
        name: String,
    },

    /// Map declaration:
    /// <div class="map" id="person">
    ///     <div class="entry" key="name"><span class="string">"alice"</span></div>
    ///     <div class="entry" key="age"><span class="integer">30</span></div>
    /// </div>
    MapDeclaration {
        name:    String,
        entries: Vec<(String, Expr)>,   // key is always a string, value is any Expr
    },

    /// Map get — read a value by key:
    /// <div class="map-get">
    ///     <var class="person"></var>
    ///     <div class="key"><span class="string">"name"</span></div>
    /// </div>
    MapGet {
        map: Box<Expr>,     // evaluates to LiteralValue::Map
        key: Box<Expr>,     // evaluates to String
    },

    /// Map set — insert or update a key:
    /// <div class="map-set">
    ///     <var class="person"></var>
    ///     <div class="key"><span class="string">"age"</span></div>
    ///     <div class="value"><span class="integer">31</span></div>
    /// </div>
    MapSet {
        name:  String,      // variable name of the map
        key:   Box<Expr>,   // evaluates to String
        value: Box<Expr>,   // any type — dynamic like Python
    },

    /// Map remove — delete a key:
    /// <div class="map-remove">
    ///     <var class="person"></var>
    ///     <div class="key"><span class="string">"age"</span></div>
    /// </div>
    MapRemove {
        name: String,       // variable name of the map
        key:  Box<Expr>,    // evaluates to String
    },

    /// Map contains — check if a key exists (returns Boolean):
    /// <div class="map-contains">
    ///     <var class="person"></var>
    ///     <div class="key"><span class="string">"age"</span></div>
    /// </div>
    MapContains {
        map: Box<Expr>,     // evaluates to LiteralValue::Map
        key: Box<Expr>,     // evaluates to String
    },

    /// Get the length of a list, map, or string:
    /// <div class="length">
    ///     <var class="myList"></var>
    /// </div>
    Length {
        value: Box<Expr>,
    },

    HttpGet {
        name: String,
        url: Box<Expr>,
    },
    HttpPost {
        name: String,
        url: Box<Expr>,
        body: Box<Expr>,
    },

    Input,

    ServerDeclaration {
        port: u16,
        endpoints: Vec<Endpoint>,
    },

    Respond {
        value: Box<Expr>,
    },
}

/// Represents a single `if` or `elif` block
#[derive(Debug, Clone)]
pub struct ConditionalBlock {
    pub condition:  Expr,
    pub statements: Vec<Expr>,
}

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    
    // Logical
    Or,
    And,
    Not,

    // Comparison
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Integer(i64),
    Double(f64),
    String(String),
    Boolean(bool),
    /// A typed, ordered, dynamically-sized list.
    List {
        elem_type: StaticType,
        elements:  Vec<LiteralValue>,
    },
    /// A Python-style dynamic map — keys are always Strings,
    /// values can be any LiteralValue including nested Maps and Lists.
    Map {
        entries: IndexMap<String, LiteralValue>,
    },
}

impl LiteralValue {
    pub fn get_type(&self) -> StaticType {
        match self {
            LiteralValue::Integer(_)             => StaticType::Integer,
            LiteralValue::Double(_)              => StaticType::Double,
            LiteralValue::String(_)              => StaticType::String,
            LiteralValue::Boolean(_)             => StaticType::Boolean,
            LiteralValue::List { elem_type, .. } => StaticType::List(Box::new(elem_type.clone())),
            LiteralValue::Map { .. }             => StaticType::Map,
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
    /// A dynamic Python-style map — string keys, any value type.
    Map,
}

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

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub method: HttpMethod,
    pub path: String,
    pub handler: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub enum HttpMethod {
    Get,
    Post,
}