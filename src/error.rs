// ANSI color codes
const RED: &str    = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str   = "\x1b[36m";
const BOLD: &str   = "\x1b[1m";
const RESET: &str  = "\x1b[0m";

// ------------------------------------------------------------------
// Stage
// ------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum HaplStage {
    Lexer,
    Parser,
    Runtime,
}

impl std::fmt::Display for HaplStage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HaplStage::Lexer   => write!(f, "Lexer"),
            HaplStage::Parser  => write!(f, "Parser"),
            HaplStage::Runtime => write!(f, "Runtime"),
        }
    }
}

// ------------------------------------------------------------------
// Error codes
// ------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum ErrorCode {
    // Lexer  E0xx
    UnknownTag,
    UnknownType,
    UnknownReturnType,
    UnknownParamType,
    InvalidLiteral,
    MissingClass,
    MissingId,
    BadConditionalChild,
    BadLoopChild,
    BadFunctionChild,
    StringNotQuoted,
    BadListChild,          

    // Parser  E1xx
    UnexpectedEof,
    UnexpectedToken,
    OperatorNotClosed,
    BadOperandCount,
    AlreadyDeclared,
    NotDeclared,
    TagNotClosed,
    TypeMismatch,
    ForIteratorNotInt,
    ForConditionNotBool,
    ForIncrementNotInt,
    ListElementTypeMismatch, 
    ListIndexNotInt,         

    // Runtime  E2xx
    VariableNotFound,
    AssignBeforeDeclare,
    FunctionNotFound,
    FunctionAlreadyDeclared,
    WrongArgCount,
    DivisionByZero,
    BadOperatorTypes,
    NotRequiresBool,
    IndexOutOfBounds,        
    EmptyList,               
    ListTypeMismatch,        
}

impl ErrorCode {
    fn code(&self) -> &'static str {
        match self {
            ErrorCode::UnknownTag              => "E001",
            ErrorCode::UnknownType             => "E002",
            ErrorCode::UnknownReturnType       => "E003",
            ErrorCode::UnknownParamType        => "E004",
            ErrorCode::InvalidLiteral          => "E005",
            ErrorCode::MissingClass            => "E006",
            ErrorCode::MissingId               => "E007",
            ErrorCode::BadConditionalChild     => "E008",
            ErrorCode::BadLoopChild            => "E009",
            ErrorCode::BadFunctionChild        => "E010",
            ErrorCode::StringNotQuoted         => "E011",
            ErrorCode::BadListChild            => "E012",

            ErrorCode::UnexpectedEof           => "E101",
            ErrorCode::UnexpectedToken         => "E102",
            ErrorCode::OperatorNotClosed       => "E103",
            ErrorCode::BadOperandCount         => "E104",
            ErrorCode::AlreadyDeclared         => "E105",
            ErrorCode::NotDeclared             => "E106",
            ErrorCode::TagNotClosed            => "E107",
            ErrorCode::TypeMismatch            => "E108",
            ErrorCode::ForIteratorNotInt       => "E109",
            ErrorCode::ForConditionNotBool     => "E110",
            ErrorCode::ForIncrementNotInt      => "E111",
            ErrorCode::ListElementTypeMismatch => "E112",
            ErrorCode::ListIndexNotInt         => "E113",

            ErrorCode::VariableNotFound        => "E201",
            ErrorCode::AssignBeforeDeclare     => "E202",
            ErrorCode::FunctionNotFound        => "E203",
            ErrorCode::FunctionAlreadyDeclared => "E204",
            ErrorCode::WrongArgCount           => "E205",
            ErrorCode::DivisionByZero          => "E206",
            ErrorCode::BadOperatorTypes        => "E207",
            ErrorCode::NotRequiresBool         => "E208",
            ErrorCode::IndexOutOfBounds        => "E209",
            ErrorCode::EmptyList               => "E210",
            ErrorCode::ListTypeMismatch        => "E211",
        }
    }
}

// ------------------------------------------------------------------
// HaplError
// ------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct HaplError {
    stage:       HaplStage,
    code:        ErrorCode,
    message:     String,
    /// The raw HTML snippet shown in the context block, e.g. `<var class="x"></var>`
    context_tag: Option<String>,
    /// The label shown under the `^^^` pointer, e.g. `'x' has not been declared`
    pointer_msg: Option<String>,
    hint:        Option<String>,
}

impl HaplError {
    pub fn new(stage: HaplStage, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            stage,
            code,
            message:     message.into(),
            context_tag: None,
            pointer_msg: None,
            hint:        None,
        }
    }

    /// Attach the HTML tag snippet that caused the error.
    /// `tag`  — the raw HTML string shown in the context block,
    ///            e.g. `<var class="myAge"></var>`
    /// `label` — text shown under the `^^^` pointer,
    ///            e.g. `'myAge' has not been declared`
    pub fn with_tag(mut self, tag: impl Into<String>, label: impl Into<String>) -> Self {
        self.context_tag = Some(tag.into());
        self.pointer_msg = Some(label.into());
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Print to stderr in Rust-style colored formatting and exit.
    pub fn report_and_exit(&self, filename: &str) -> ! {
        eprintln!("{}", self);
        eprintln!("  {}{}--> {}{}", BOLD, CYAN, filename, RESET);
        std::process::exit(1);
    }
}

impl std::fmt::Display for HaplError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // error[E001]: message
        writeln!(
            f,
            "{}{}error[{}]{}: {}{}{}",
            BOLD, RED, self.code.code(), RESET,
            BOLD, self.message, RESET
        )?;

        // (Stage)
        writeln!(
            f,
            "  {}{}--> ({}){}",
            BOLD, CYAN, self.stage, RESET
        )?;

        // context block
        if let Some(tag) = &self.context_tag {
            let label = self.pointer_msg.as_deref().unwrap_or("");

            let pointer_len = {
                let inner = tag.trim_start_matches('<');
                let end = inner.find(|c: char| c == ' ' || c == '>' || c == '/')
                    .unwrap_or(inner.len());
                (end + 1).max(3)
            };

            let carets = "^".repeat(pointer_len);

            writeln!(f, "   {}{}|{}", BOLD, CYAN, RESET)?;
            writeln!(f, "   {}{}|{}  {}", BOLD, CYAN, RESET, tag)?;
            writeln!(
                f,
                "   {}{}|{}  {}{}{}{}",
                BOLD, CYAN, RESET,
                BOLD, RED, carets, RESET
            )?;

            if !label.is_empty() {
                writeln!(
                    f,
                    "   {}{}|{}  {}{}{}{}",
                    BOLD, CYAN, RESET,
                    " ".repeat(pointer_len + 1),
                    BOLD, label, RESET
                )?;
            }

            writeln!(f, "   {}{}|{}", BOLD, CYAN, RESET)?;
        }

        if let Some(hint) = &self.hint {
            writeln!(
                f,
                "   {}{}= hint:{} {}",
                BOLD, YELLOW, RESET, hint
            )?;
        }

        Ok(())
    }
}

impl std::error::Error for HaplError {}

// ------------------------------------------------------------------
// Shorthand constructors
// ------------------------------------------------------------------
pub fn lexer_err(code: ErrorCode, msg: impl Into<String>) -> HaplError {
    HaplError::new(HaplStage::Lexer, code, msg)
}

pub fn parser_err(code: ErrorCode, msg: impl Into<String>) -> HaplError {
    HaplError::new(HaplStage::Parser, code, msg)
}

pub fn runtime_err(code: ErrorCode, msg: impl Into<String>) -> HaplError {
    HaplError::new(HaplStage::Runtime, code, msg)
}