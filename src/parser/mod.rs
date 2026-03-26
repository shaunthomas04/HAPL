mod scope;
mod parse_expressions;
mod parse_control;
mod parse_functions;
mod parse_collections;
mod parse_io;

use std::collections::HashMap;
use crate::lexer::{HaplToken, HaplTokenType, LexerTagType, LoopType};
use crate::ast::{Expr, LiteralValue, Operator, StaticType};
use crate::error::{ErrorCode, HaplError, parser_err};

/// Parameter list and return type registered for a declared function.
#[derive(Debug, Clone)]
pub(super) struct FunctionSignature {
    pub params:      Vec<(String, StaticType)>,
    pub return_type: StaticType,
}

pub struct HaplParser {
    pub(super) tokens:               Vec<HaplToken>,
    pub(super) position:             usize,
    pub(super) scope_stack:          Vec<HashMap<String, StaticType>>,
    pub(super) function_signatures:  HashMap<String, FunctionSignature>,
    pub(super) current_fn_return_type: Option<StaticType>,
}

impl HaplParser {
    pub fn new(tokens: Vec<HaplToken>) -> Self {
        Self {
            tokens,
            position: 0,
            scope_stack: vec![HashMap::new()],
            function_signatures: HashMap::new(),
            current_fn_return_type: None,
        }
    }

    // --------------------------------------------------
    // Parse an entire program (two-pass)
    // --------------------------------------------------
    pub fn parse_program(&mut self) -> Result<Vec<Expr>, HaplError> {
        // Pass 1 — pre-register all function signatures for forward-call checking
        self.prescan_function_signatures();

        // Pass 2 — full parse
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    // --------------------------------------------------
    // Top-level statement dispatcher
    // --------------------------------------------------
    pub fn parse_statement(&mut self) -> Result<Expr, HaplError> {
        match self.current_token() {
            // Print
            HaplTokenType::OpenPrint { .. } => {
                self.advance();
                let inner = self.parse_expression()?;
                if self.is_at_end() || !matches!(self.current_token(), HaplTokenType::ClosePrint { .. }) {
                    return Err(
                        parser_err(ErrorCode::TagNotClosed, "print statement was never closed")
                            .with_hint("add a matching </p> closing tag"),
                    );
                }
                self.advance();
                Ok(Expr::Print { value: Box::new(inner) })
            }

            // Control flow
            HaplTokenType::OpenConditional => self.parse_conditional(),
            HaplTokenType::OpenLoop { loop_type } if loop_type == LoopType::While => self.parse_while(),
            HaplTokenType::OpenLoop { loop_type } if loop_type == LoopType::For   => self.parse_for(),

            // Functions
            HaplTokenType::OpenFunction { .. } => self.parse_function(),
            HaplTokenType::OpenReturn          => self.parse_return(),

            // Server
            HaplTokenType::OpenServer { .. } => self.parse_server(),

            // Structural HTML wrappers — skip silently
            HaplTokenType::OpenHtmlTag { .. } | HaplTokenType::CloseHtmlTag { .. } => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Boolean(false)))
            }

            // Everything else falls through to parse_expression
            HaplTokenType::OpenVarDec { .. }
            | HaplTokenType::OpenOperator { .. }
            | HaplTokenType::Literal(_)
            | HaplTokenType::OpenVarRef { .. }
            | HaplTokenType::OpenVarAssign { .. }
            | HaplTokenType::OpenFunctionCall { .. }
            | HaplTokenType::OpenListDec { .. }
            | HaplTokenType::OpenListAccess
            | HaplTokenType::OpenListAssign { .. }
            | HaplTokenType::OpenListPush { .. }
            | HaplTokenType::OpenListPop { .. }
            | HaplTokenType::OpenLength
            | HaplTokenType::OpenHttpGet { .. }
            | HaplTokenType::OpenHttpPost { .. }
            | HaplTokenType::OpenInput
            | HaplTokenType::OpenMapDec { .. }
            | HaplTokenType::OpenMapGet
            | HaplTokenType::OpenMapSet { .. }
            | HaplTokenType::OpenMapRemove { .. }
            | HaplTokenType::OpenMapContains
            | HaplTokenType::OpenRespond => self.parse_expression(),

            other => Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!(
                    "unexpected token '{:?}' at top-level position {}",
                    other, self.position
                ),
            )
            .with_hint(
                "expected a statement: variable declaration, print, conditional, loop, or function",
            )),
        }
    }

    // --------------------------------------------------
    // Low-level token navigation helpers
    // --------------------------------------------------

    pub(super) fn current_token(&self) -> HaplTokenType {
        self.tokens[self.position].token_type.clone()
    }

    pub(super) fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
        }
    }

    pub(super) fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    pub(super) fn check_close_operator(&self, expected: LexerTagType) -> bool {
        if self.is_at_end() { return false; }
        matches!(self.current_token(), HaplTokenType::CloseOperator { name } if name == expected)
    }

    pub(super) fn check_close_var_dec(&self, var_type: &StaticType, name: &str) -> bool {
        if self.is_at_end() { return false; }
        matches!(
            self.current_token(),
            HaplTokenType::CloseVarDec { var_type: t, name: n } if t == *var_type && n == name
        )
    }

    pub(super) fn check_close_var_ref(&self, name: &str) -> bool {
        if self.is_at_end() { return false; }
        matches!(self.current_token(), HaplTokenType::CloseVarRef { name: n } if n == name)
    }

    pub(super) fn map_operator(&self, op: LexerTagType) -> Operator {
        match op {
            LexerTagType::Add          => Operator::Add,
            LexerTagType::Subtract     => Operator::Subtract,
            LexerTagType::Multiply     => Operator::Multiply,
            LexerTagType::Divide       => Operator::Divide,
            LexerTagType::Modulo       => Operator::Modulo,
            LexerTagType::And          => Operator::And,
            LexerTagType::Or           => Operator::Or,
            LexerTagType::Not          => Operator::Not,
            LexerTagType::Equal        => Operator::Equal,
            LexerTagType::NotEqual     => Operator::NotEqual,
            LexerTagType::Less         => Operator::Less,
            LexerTagType::LessEqual    => Operator::LessEqual,
            LexerTagType::Greater      => Operator::Greater,
            LexerTagType::GreaterEqual => Operator::GreaterEqual,
        }
    }

    pub(super) fn operator_name_str(&self, op: LexerTagType) -> &'static str {
        match op {
            LexerTagType::Add          => "+",
            LexerTagType::Subtract     => "-",
            LexerTagType::Multiply     => "*",
            LexerTagType::Divide       => "/",
            LexerTagType::Modulo       => "%",
            LexerTagType::And          => "&&",
            LexerTagType::Or           => "||",
            LexerTagType::Not          => "!",
            LexerTagType::Equal        => "equal",
            LexerTagType::NotEqual     => "not_equal",
            LexerTagType::Less         => "less",
            LexerTagType::LessEqual    => "less_equal",
            LexerTagType::Greater      => "greater",
            LexerTagType::GreaterEqual => "greater_equal",
        }
    }
}
