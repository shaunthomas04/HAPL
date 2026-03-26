use super::*;
use crate::ast::{Expr, LiteralValue, Operator, StaticType};
use crate::error::{ErrorCode, HaplError, parser_err};

impl HaplParser {
    // --------------------------------------------------
    // Scope helpers
    // --------------------------------------------------

    pub(super) fn push_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    pub(super) fn pop_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
    }

    pub(super) fn declare_var(&mut self, name: String, var_type: StaticType) -> Result<(), HaplError> {
        let current = self.scope_stack.last_mut().ok_or_else(|| {
            parser_err(
                ErrorCode::UnexpectedEof,
                "internal error: scope stack is empty — cannot declare variable",
            )
        })?;
        if current.contains_key(&name) {
            return Err(
                parser_err(
                    ErrorCode::AlreadyDeclared,
                    format!("variable '{}' is already declared in this scope", name),
                )
                .with_hint(format!(
                    "consider renaming the variable or assigning to it with <var class=\"{}\">",
                    name
                )),
            );
        }
        current.insert(name, var_type);
        Ok(())
    }

    pub(super) fn lookup_var(&self, name: &str) -> Option<StaticType> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(t.clone());
            }
        }
        None
    }

    pub(super) fn var_exists(&self, name: &str) -> bool {
        self.lookup_var(name).is_some()
    }

    // --------------------------------------------------
    // Type inference
    // --------------------------------------------------

    pub(super) fn infer_type(&self, expr: &Expr) -> Option<StaticType> {
        match expr {
            Expr::Literal(LiteralValue::Integer(_)) => Some(StaticType::Integer),
            Expr::Literal(LiteralValue::Double(_))  => Some(StaticType::Double),
            Expr::Literal(LiteralValue::String(_))  => Some(StaticType::String),
            Expr::Literal(LiteralValue::Boolean(_)) => Some(StaticType::Boolean),

            Expr::VariableReference { name } => self.lookup_var(name),

            Expr::FunctionCall { name, .. } => {
                self.function_signatures
                    .get(name)
                    .map(|sig| sig.return_type.clone())
            }

            Expr::Operation { op, operands } => match op {
                Operator::Equal
                | Operator::NotEqual
                | Operator::Less
                | Operator::LessEqual
                | Operator::Greater
                | Operator::GreaterEqual
                | Operator::And
                | Operator::Or
                | Operator::Not => Some(StaticType::Boolean),

                Operator::Add
                | Operator::Subtract
                | Operator::Multiply
                | Operator::Divide
                | Operator::Modulo => {
                    let mut result_type = None;
                    for operand in operands {
                        match self.infer_type(operand) {
                            Some(StaticType::Double) => {
                                result_type = Some(StaticType::Double);
                                break;
                            }
                            Some(t) if result_type.is_none() => {
                                result_type = Some(t);
                            }
                            _ => {}
                        }
                    }
                    result_type
                }
            },

            Expr::Literal(LiteralValue::List { elem_type, .. }) => {
                Some(StaticType::List(Box::new(elem_type.clone())))
            }

            Expr::Literal(LiteralValue::Map { .. }) => Some(StaticType::Map),
            Expr::MapGet { .. }      => Some(StaticType::Map),
            Expr::MapContains { .. } => Some(StaticType::Boolean),
            Expr::Length { .. }      => Some(StaticType::Integer),
            Expr::HttpGet { .. } | Expr::HttpPost { .. } => Some(StaticType::Map),
            Expr::Input              => Some(StaticType::String),
            _ => None,
        }
    }
}
