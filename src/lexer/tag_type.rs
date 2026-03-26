#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LexerTagType {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    And,
    Or,
    Not,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopType {
    While,
    For,
}
