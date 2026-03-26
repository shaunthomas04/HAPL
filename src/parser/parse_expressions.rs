use super::*;
use crate::ast::{Expr, LiteralValue, StaticType};
use crate::error::{ErrorCode, HaplError, parser_err};
use crate::lexer::HaplTokenType;

impl HaplParser {
    pub fn parse_expression(&mut self) -> Result<Expr, HaplError> {
        if self.is_at_end() {
            return Err(parser_err(
                ErrorCode::UnexpectedEof,
                "unexpected end of token stream while parsing expression",
            ));
        }

        match self.current_token() {
            // -------------------------
            // Literal
            // -------------------------
            HaplTokenType::Literal(lit) => {
                self.advance();
                Ok(Expr::Literal(lit))
            }

            // -------------------------
            // Operator
            // -------------------------
            HaplTokenType::OpenOperator { name } => {
                let operator = self.map_operator(name);
                self.advance();

                let mut operands = Vec::new();
                while !self.is_at_end() && !self.check_close_operator(name) {
                    operands.push(self.parse_expression()?);
                }

                if self.is_at_end() {
                    return Err(parser_err(
                        ErrorCode::OperatorNotClosed,
                        format!("operator '{}' was never closed", self.operator_name_str(name)),
                    )
                    .with_hint(format!(
                        "add a matching closing tag for the '{}' operator",
                        self.operator_name_str(name)
                    )));
                }

                self.advance(); // consume CloseOperator

                match operator {
                    crate::ast::Operator::Not => {
                        if operands.len() != 1 {
                            return Err(parser_err(
                                ErrorCode::BadOperandCount,
                                format!("'not' operator requires exactly 1 operand, got {}", operands.len()),
                            )
                            .with_hint(
                                "example: <div class=\"!\"><span class=\"boolean\">true</span></div>",
                            ));
                        }
                    }
                    crate::ast::Operator::Equal
                    | crate::ast::Operator::NotEqual
                    | crate::ast::Operator::Less
                    | crate::ast::Operator::LessEqual
                    | crate::ast::Operator::Greater
                    | crate::ast::Operator::GreaterEqual => {
                        if operands.len() != 2 {
                            return Err(parser_err(
                                ErrorCode::BadOperandCount,
                                format!(
                                    "'{}' operator requires exactly 2 operands, got {}",
                                    self.operator_name_str(name),
                                    operands.len()
                                ),
                            )
                            .with_hint("comparison operators must have exactly 2 operands"));
                        }
                    }
                    _ => {
                        if operands.len() < 2 {
                            return Err(parser_err(
                                ErrorCode::BadOperandCount,
                                format!(
                                    "'{}' operator requires at least 2 operands, got {}",
                                    self.operator_name_str(name),
                                    operands.len()
                                ),
                            )
                            .with_hint(
                                "arithmetic and logical operators need at least 2 operands",
                            ));
                        }
                    }
                }

                Ok(Expr::Operation { op: operator, operands })
            }

            // -------------------------
            // Variable Declaration
            // -------------------------
            HaplTokenType::OpenVarDec { var_type, name } => {
                let var_name = name.clone();
                let var_type_copy = var_type;
                self.advance();

                let value_expr = Box::new(self.parse_expression()?);

                // Reject void function calls as initialiser values
                if let Expr::FunctionCall { name: ref fn_name, .. } = *value_expr {
                    if let Some(sig) = self.function_signatures.get(fn_name) {
                        if sig.return_type == StaticType::Void {
                            return Err(parser_err(
                                ErrorCode::TypeMismatch,
                                format!(
                                    "cannot use void function '{}' as a value in declaration of '{}'",
                                    fn_name, var_name
                                ),
                            )
                            .with_hint(
                                "void functions do not return a value and cannot be used as expressions",
                            ));
                        }
                    }
                }

                if self.is_at_end() || !self.check_close_var_dec(&var_type_copy, &var_name) {
                    return Err(parser_err(
                        ErrorCode::TagNotClosed,
                        format!("variable declaration '{}' was never closed", var_name),
                    )
                    .with_hint(format!(
                        "add </var> or a matching CloseVarDec for '{}'",
                        var_name
                    )));
                }
                self.advance(); // consume CloseVarDec

                self.declare_var(var_name.clone(), var_type_copy.clone())?;

                // Type-check literals at parse time
                if let Expr::Literal(ref lit_val) = *value_expr {
                    match (&var_type_copy, lit_val) {
                        (StaticType::Integer, LiteralValue::Integer(_))
                        | (StaticType::Double, LiteralValue::Double(_))
                        | (StaticType::String, LiteralValue::String(_))
                        | (StaticType::Boolean, LiteralValue::Boolean(_)) => {}
                        _ => {
                            return Err(parser_err(
                                ErrorCode::TypeMismatch,
                                format!(
                                    "type mismatch in variable '{}' declaration: expected {:?}, got {:?}",
                                    var_name, var_type_copy, lit_val
                                ),
                            )
                            .with_hint(format!(
                                "the declared type is {:?} — make sure the value matches",
                                var_type_copy
                            )));
                        }
                    }
                }

                Ok(Expr::VariableDeclaration {
                    name: var_name,
                    var_type: var_type_copy,
                    value: value_expr,
                })
            }

            // -------------------------
            // Variable Reference
            // -------------------------
            HaplTokenType::OpenVarRef { name } => {
                let var_name = name.clone();
                self.advance();

                if !self.var_exists(&var_name) {
                    return Err(parser_err(
                        ErrorCode::NotDeclared,
                        format!("variable '{}' used before declaration", var_name),
                    )
                    .with_hint(format!(
                        "declare '{}' with <var class=\"<type>\" id=\"{}\"> before using it",
                        var_name, var_name
                    )));
                }

                if self.is_at_end() || !self.check_close_var_ref(&var_name) {
                    return Err(parser_err(
                        ErrorCode::TagNotClosed,
                        format!("variable reference '{}' was never closed", var_name),
                    )
                    .with_hint(format!(
                        "add a matching CloseVarRef for '{}'",
                        var_name
                    )));
                }
                self.advance(); // consume CloseVarRef

                Ok(Expr::VariableReference { name: var_name })
            }

            // -------------------------
            // Variable Assignment
            // -------------------------
            HaplTokenType::OpenVarAssign { name } => {
                let var_name = name.clone();
                self.advance();

                let expected_type = match self.lookup_var(&var_name) {
                    Some(t) => t,
                    None => {
                        return Err(parser_err(
                            ErrorCode::NotDeclared,
                            format!(
                                "cannot assign to '{}' — variable has not been declared",
                                var_name
                            ),
                        )
                        .with_hint(format!("declare '{}' before assigning to it", var_name)));
                    }
                };

                let value_expr = Box::new(self.parse_expression()?);

                // Reject void function calls as assignment values
                if let Expr::FunctionCall { name: ref fn_name, .. } = *value_expr {
                    if let Some(sig) = self.function_signatures.get(fn_name) {
                        if sig.return_type == StaticType::Void {
                            return Err(parser_err(
                                ErrorCode::TypeMismatch,
                                format!(
                                    "cannot use void function '{}' as a value in assignment to '{}'",
                                    fn_name, var_name
                                ),
                            )
                            .with_hint(
                                "void functions do not return a value and cannot be used as expressions",
                            ));
                        }
                    }
                }

                if let Expr::Literal(ref lit_val) = *value_expr {
                    match (&expected_type, lit_val) {
                        (StaticType::Integer, LiteralValue::Integer(_))
                        | (StaticType::Double, LiteralValue::Double(_))
                        | (StaticType::String, LiteralValue::String(_))
                        | (StaticType::Boolean, LiteralValue::Boolean(_)) => {}
                        _ => {
                            return Err(parser_err(
                                ErrorCode::TypeMismatch,
                                format!(
                                    "type mismatch in assignment to '{}': expected {:?}, got {:?}",
                                    var_name, expected_type, lit_val
                                ),
                            )
                            .with_hint(format!(
                                "'{}' was declared as {:?} — the assigned value must match",
                                var_name, expected_type
                            )));
                        }
                    }
                }

                if self.is_at_end()
                    || !matches!(
                        self.current_token(),
                        HaplTokenType::CloseVarAssign { name: ref n } if n == &var_name
                    )
                {
                    return Err(parser_err(
                        ErrorCode::TagNotClosed,
                        format!("variable assignment '{}' was never closed", var_name),
                    )
                    .with_hint(format!(
                        "add a matching CloseVarAssign for '{}'",
                        var_name
                    )));
                }
                self.advance(); // consume CloseVarAssign

                Ok(Expr::Assignment {
                    name: var_name,
                    value: value_expr,
                })
            }

            // -------------------------
            // Function Call
            // -------------------------
            HaplTokenType::OpenFunctionCall { .. } => self.parse_function_call(),

            // -------------------------
            // Length
            // -------------------------
            HaplTokenType::OpenLength => {
                self.advance();
                let value_expr = Box::new(self.parse_expression()?);

                if self.is_at_end() || !matches!(self.current_token(), HaplTokenType::CloseLength) {
                    return Err(parser_err(ErrorCode::TagNotClosed, "length block was never closed")
                        .with_hint(
                            "add a matching closing tag for the <div class=\"length\"> block",
                        ));
                }
                self.advance();
                Ok(Expr::Length { value: value_expr })
            }

            // -------------------------
            // Respond
            // -------------------------
            HaplTokenType::OpenRespond => {
                self.advance();
                let value_expr = Box::new(self.parse_expression()?);

                if !matches!(self.current_token(), HaplTokenType::CloseRespond) {
                    return Err(
                        parser_err(ErrorCode::TagNotClosed, "respond block was never closed")
                            .with_hint(
                                "add a matching closing tag for the <div class=\"respond\"> block",
                            ),
                    );
                }
                self.advance();
                Ok(Expr::Respond { value: value_expr })
            }

            // -------------------------
            // Input
            // -------------------------
            HaplTokenType::OpenInput => {
                self.advance();
                if !matches!(self.current_token(), HaplTokenType::CloseInput) {
                    return Err(
                        parser_err(ErrorCode::TagNotClosed, "input block was never closed")
                            .with_hint("input takes no children: <div class=\"input\"></div>"),
                    );
                }
                self.advance();
                Ok(Expr::Input)
            }

            // Delegate list / map / http to their own modules
            HaplTokenType::OpenListDec { .. }
            | HaplTokenType::OpenListAccess
            | HaplTokenType::OpenListAssign { .. }
            | HaplTokenType::OpenListPush { .. }
            | HaplTokenType::OpenListPop { .. } => self.parse_collection_expression(),

            HaplTokenType::OpenMapDec { .. }
            | HaplTokenType::OpenMapGet
            | HaplTokenType::OpenMapContains
            | HaplTokenType::OpenMapSet { .. }
            | HaplTokenType::OpenMapRemove { .. } => self.parse_collection_expression(),

            HaplTokenType::OpenHttpGet { .. } | HaplTokenType::OpenHttpPost { .. } => {
                self.parse_io_expression()
            }

            other => Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!("unexpected token '{:?}' at position {}", other, self.position),
            )
            .with_hint(
                "expected a literal, variable, operator, or function call",
            )),
        }
    }
}
