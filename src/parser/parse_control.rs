use super::*;
use crate::ast::{ConditionalBlock, Expr, StaticType};
use crate::error::{ErrorCode, HaplError, parser_err};
use crate::lexer::{HaplTokenType, LoopType};

impl HaplParser {
    // --------------------------------------------------
    // Conditional
    // --------------------------------------------------
    pub(super) fn parse_conditional(&mut self) -> Result<Expr, HaplError> {
        self.advance(); // consume OpenConditional

        let mut if_blocks = Vec::new();
        let mut else_block: Option<Vec<Expr>> = None;

        while !matches!(self.current_token(), HaplTokenType::CloseConditional) {
            match self.current_token() {
                HaplTokenType::OpenIf => {
                    if_blocks.push(self.parse_conditional_block(
                        HaplTokenType::OpenIf,
                        HaplTokenType::CloseIf,
                    )?);
                }
                HaplTokenType::OpenElif => {
                    if_blocks.push(self.parse_conditional_block(
                        HaplTokenType::OpenElif,
                        HaplTokenType::CloseElif,
                    )?);
                }
                HaplTokenType::OpenElse => {
                    else_block = Some(self.parse_else_block()?);
                }
                other => {
                    return Err(parser_err(
                        ErrorCode::UnexpectedToken,
                        format!("unexpected token '{:?}' inside <conditional>", other),
                    )
                    .with_hint(
                        "valid children of <conditional>: <if>, <elif>, <else>",
                    ));
                }
            }
        }

        self.advance(); // consume CloseConditional
        Ok(Expr::Conditional { if_blocks, else_block })
    }

    pub(super) fn parse_conditional_block(
        &mut self,
        _open: HaplTokenType,
        close: HaplTokenType,
    ) -> Result<ConditionalBlock, HaplError> {
        self.advance(); // consume OpenIf or OpenElif
        self.push_scope();

        let condition = self.parse_expression()?;

        let mut statements = Vec::new();
        while !self.is_at_end() {
            let at_close = match &close {
                HaplTokenType::CloseIf   => matches!(self.current_token(), HaplTokenType::CloseIf),
                HaplTokenType::CloseElif => matches!(self.current_token(), HaplTokenType::CloseElif),
                _ => false,
            };
            if at_close { break; }
            statements.push(self.parse_statement()?);
        }

        if self.is_at_end() {
            self.pop_scope();
            return Err(parser_err(
                ErrorCode::TagNotClosed,
                "conditional block (if/elif) was never closed",
            )
            .with_hint(
                "add the matching closing tag for the if or elif block",
            ));
        }

        self.pop_scope();
        self.advance(); // consume CloseIf or CloseElif
        Ok(ConditionalBlock { condition, statements })
    }

    pub(super) fn parse_else_block(&mut self) -> Result<Vec<Expr>, HaplError> {
        self.advance(); // consume OpenElse
        self.push_scope();

        let mut statements = Vec::new();
        while !self.is_at_end() && !matches!(self.current_token(), HaplTokenType::CloseElse) {
            statements.push(self.parse_statement()?);
        }

        if self.is_at_end() {
            self.pop_scope();
            return Err(parser_err(ErrorCode::TagNotClosed, "else block was never closed")
                .with_hint("add a matching closing tag for the else block"));
        }

        self.pop_scope();
        self.advance(); // consume CloseElse
        Ok(statements)
    }

    // --------------------------------------------------
    // While loop
    // --------------------------------------------------
    pub(super) fn parse_while(&mut self) -> Result<Expr, HaplError> {
        self.advance(); // consume OpenLoop(While)

        if !matches!(self.current_token(), HaplTokenType::OpenLoopCondition) {
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!(
                    "expected <condition> inside while loop but found '{:?}'",
                    self.current_token()
                ),
            )
            .with_hint("while loops must have a <div class=\"condition\"> block"));
        }
        self.advance(); // consume OpenLoopCondition

        let condition_expr = self.parse_expression()?;

        if !matches!(self.current_token(), HaplTokenType::CloseLoopCondition) {
            return Err(
                parser_err(ErrorCode::TagNotClosed, "while loop condition was never closed")
                    .with_hint(
                        "add </div> to close the <div class=\"condition\"> block",
                    ),
            );
        }
        self.advance(); // consume CloseLoopCondition

        if !matches!(self.current_token(), HaplTokenType::OpenLoopBody) {
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!(
                    "expected <body> inside while loop but found '{:?}'",
                    self.current_token()
                ),
            )
            .with_hint("while loops must have a <div class=\"body\"> block"));
        }
        self.advance(); // consume OpenLoopBody

        self.push_scope();
        let mut body = Vec::new();
        while !self.is_at_end() && !matches!(self.current_token(), HaplTokenType::CloseLoopBody) {
            body.push(self.parse_statement()?);
        }

        if self.is_at_end() {
            self.pop_scope();
            return Err(
                parser_err(ErrorCode::TagNotClosed, "while loop body was never closed")
                    .with_hint("add </div> to close the <div class=\"body\"> block"),
            );
        }

        self.pop_scope();
        self.advance(); // consume CloseLoopBody

        if let HaplTokenType::CloseLoop { loop_type } = self.current_token() {
            if loop_type != LoopType::While {
                return Err(parser_err(
                    ErrorCode::TagNotClosed,
                    "expected closing tag for while loop but found a different loop type",
                ));
            }
        } else {
            return Err(
                parser_err(ErrorCode::TagNotClosed, "while loop was never closed")
                    .with_hint(
                        "add a matching closing tag for the <div class=\"while\"> block",
                    ),
            );
        }
        self.advance(); // consume CloseLoop

        Ok(Expr::WhileLoop {
            condition: Box::new(condition_expr),
            body,
        })
    }

    // --------------------------------------------------
    // For loop
    // --------------------------------------------------
    pub(super) fn parse_for(&mut self) -> Result<Expr, HaplError> {
        self.advance(); // consume OpenLoop(For)
        self.push_scope();

        // Iterator
        if !matches!(self.current_token(), HaplTokenType::OpenLoopIterator) {
            self.pop_scope();
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!(
                    "expected <iterator> inside for loop but found '{:?}'",
                    self.current_token()
                ),
            )
            .with_hint("for loops must have a <div class=\"iterator\"> block"));
        }
        self.advance(); // consume OpenLoopIterator

        let iterator_expr = self.parse_expression()?;

        let iterator_name = match &iterator_expr {
            Expr::VariableReference { name } => name.clone(),
            _ => {
                self.pop_scope();
                return Err(
                    parser_err(
                        ErrorCode::ForIteratorNotInt,
                        "for loop iterator must be a variable reference",
                    )
                    .with_hint(
                        "example: <div class=\"iterator\"><var class=\"i\"></var></div>",
                    ),
                );
            }
        };

        if self.scope_stack.last().map_or(true, |s| !s.contains_key(&iterator_name)) {
            self.declare_var(iterator_name.clone(), StaticType::Integer)?;
        }

        match self.lookup_var(&iterator_name) {
            Some(t) if t != StaticType::Integer => {
                self.pop_scope();
                return Err(parser_err(
                    ErrorCode::ForIteratorNotInt,
                    format!(
                        "for loop iterator '{}' must be of type Integer, got {:?}",
                        iterator_name, t
                    ),
                )
                .with_hint(
                    "declare the iterator variable as an integer before the loop",
                ));
            }
            _ => {}
        }

        if !matches!(self.current_token(), HaplTokenType::CloseLoopIterator) {
            self.pop_scope();
            return Err(
                parser_err(ErrorCode::TagNotClosed, "for loop iterator block was never closed")
                    .with_hint(
                        "add </div> to close the <div class=\"iterator\"> block",
                    ),
            );
        }
        self.advance(); // consume CloseLoopIterator

        // Condition
        if !matches!(self.current_token(), HaplTokenType::OpenLoopCondition) {
            self.pop_scope();
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!(
                    "expected <condition> inside for loop but found '{:?}'",
                    self.current_token()
                ),
            )
            .with_hint("for loops must have a <div class=\"condition\"> block"));
        }
        self.advance(); // consume OpenLoopCondition

        let condition_expr = self.parse_expression()?;

        if let Some(cond_type) = self.infer_type(&condition_expr) {
            if cond_type != StaticType::Boolean {
                self.pop_scope();
                return Err(parser_err(
                    ErrorCode::ForConditionNotBool,
                    format!(
                        "for loop condition must evaluate to Boolean, got {:?}",
                        cond_type
                    ),
                )
                .with_hint(
                    "use a comparison operator (e.g. less, greater) to produce a Boolean",
                ));
            }
        }

        if !matches!(self.current_token(), HaplTokenType::CloseLoopCondition) {
            self.pop_scope();
            return Err(
                parser_err(ErrorCode::TagNotClosed, "for loop condition block was never closed")
                    .with_hint(
                        "add </div> to close the <div class=\"condition\"> block",
                    ),
            );
        }
        self.advance(); // consume CloseLoopCondition

        // Increment
        if !matches!(self.current_token(), HaplTokenType::OpenLoopIncrement) {
            self.pop_scope();
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!(
                    "expected <increment> inside for loop but found '{:?}'",
                    self.current_token()
                ),
            )
            .with_hint("for loops must have a <div class=\"increment\"> block"));
        }
        self.advance(); // consume OpenLoopIncrement

        let increment_expr = self.parse_expression()?;

        if let Some(inc_type) = self.infer_type(&increment_expr) {
            if inc_type != StaticType::Integer {
                self.pop_scope();
                return Err(parser_err(
                    ErrorCode::ForIncrementNotInt,
                    format!(
                        "for loop increment must evaluate to Integer, got {:?}",
                        inc_type
                    ),
                )
                .with_hint(
                    "the increment expression must produce an integer value",
                ));
            }
        }

        if !matches!(self.current_token(), HaplTokenType::CloseLoopIncrement) {
            self.pop_scope();
            return Err(
                parser_err(ErrorCode::TagNotClosed, "for loop increment block was never closed")
                    .with_hint(
                        "add </div> to close the <div class=\"increment\"> block",
                    ),
            );
        }
        self.advance(); // consume CloseLoopIncrement

        // Body
        if !matches!(self.current_token(), HaplTokenType::OpenLoopBody) {
            self.pop_scope();
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!(
                    "expected <body> inside for loop but found '{:?}'",
                    self.current_token()
                ),
            )
            .with_hint("for loops must have a <div class=\"body\"> block"));
        }
        self.advance(); // consume OpenLoopBody

        let mut body = Vec::new();
        while !self.is_at_end() && !matches!(self.current_token(), HaplTokenType::CloseLoopBody) {
            body.push(self.parse_statement()?);
        }

        if self.is_at_end() {
            self.pop_scope();
            return Err(
                parser_err(ErrorCode::TagNotClosed, "for loop body was never closed")
                    .with_hint("add </div> to close the <div class=\"body\"> block"),
            );
        }

        self.advance(); // consume CloseLoopBody
        self.pop_scope();

        if let HaplTokenType::CloseLoop { loop_type } = self.current_token() {
            if loop_type != LoopType::For {
                return Err(parser_err(
                    ErrorCode::TagNotClosed,
                    "expected closing tag for for loop but found a different loop type",
                ));
            }
        } else {
            return Err(
                parser_err(ErrorCode::TagNotClosed, "for loop was never closed")
                    .with_hint(
                        "add a matching closing tag for the <div class=\"for\"> block",
                    ),
            );
        }
        self.advance(); // consume CloseLoop

        Ok(Expr::ForLoop {
            iterator: iterator_name,
            condition: Box::new(condition_expr),
            increment: Box::new(increment_expr),
            body,
        })
    }
}
