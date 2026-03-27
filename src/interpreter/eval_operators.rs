use super::*;
use crate::ast::{LiteralValue, Operator};
use crate::error::{ErrorCode, HaplError, runtime_err};
use helpers::{op_name, TypeName};

impl Interpreter {
    /// Apply a binary operator to two already-evaluated values.
    pub(super) fn apply_operator(
        op: &Operator,
        lhs: &LiteralValue,
        rhs: &LiteralValue,
    ) -> Result<LiteralValue, HaplError> {
        match (lhs, rhs) {
            // Boolean ops
            (LiteralValue::Boolean(a), LiteralValue::Boolean(b)) => match op {
                Operator::And      => Ok(LiteralValue::Boolean(*a && *b)),
                Operator::Or       => Ok(LiteralValue::Boolean(*a || *b)),
                Operator::Equal    => Ok(LiteralValue::Boolean(a == b)),
                Operator::NotEqual => Ok(LiteralValue::Boolean(a != b)),
                _ => Err(runtime_err(
                    ErrorCode::BadOperatorTypes,
                    format!(
                        "operator '{}' cannot be applied to boolean operands",
                        op_name(op)
                    ),
                )
                .with_hint("booleans support: &&, ||, equal, not_equal")),
            },

            // String concat / compare
            (LiteralValue::String(a), LiteralValue::String(b)) => match op {
                Operator::Add      => Ok(LiteralValue::String(format!("{}{}", a, b))),
                Operator::Equal    => Ok(LiteralValue::Boolean(a == b)),
                Operator::NotEqual => Ok(LiteralValue::Boolean(a != b)),
                _ => Err(runtime_err(
                    ErrorCode::BadOperatorTypes,
                    format!(
                        "operator '{}' cannot be applied to string operands",
                        op_name(op)
                    ),
                )
                .with_hint("strings support: + (concat), equal, not_equal")),
            },

            // Integer ops
            (LiteralValue::Integer(a), LiteralValue::Integer(b)) => match op {
                Operator::Add      => Ok(LiteralValue::Integer(a + b)),
                Operator::Subtract => Ok(LiteralValue::Integer(a - b)),
                Operator::Multiply => Ok(LiteralValue::Integer(a * b)),
                Operator::Divide => {
                    if *b == 0 {
                        return Err(runtime_err(ErrorCode::DivisionByZero, "division by zero")
                            .with_hint(
                                "check that the divisor is never zero before dividing",
                            ));
                    }
                    Ok(LiteralValue::Integer(a / b))
                }
                Operator::Modulo => {
                    if *b == 0 {
                        return Err(runtime_err(ErrorCode::DivisionByZero, "modulo by zero")
                            .with_hint(
                                "check that the divisor is never zero before using modulo",
                            ));
                    }
                    Ok(LiteralValue::Integer(a % b))
                }
                Operator::Equal        => Ok(LiteralValue::Boolean(a == b)),
                Operator::NotEqual     => Ok(LiteralValue::Boolean(a != b)),
                Operator::Less         => Ok(LiteralValue::Boolean(a < b)),
                Operator::LessEqual    => Ok(LiteralValue::Boolean(a <= b)),
                Operator::Greater      => Ok(LiteralValue::Boolean(a > b)),
                Operator::GreaterEqual => Ok(LiteralValue::Boolean(a >= b)),
                _ => Err(runtime_err(
                    ErrorCode::BadOperatorTypes,
                    format!(
                        "operator '{}' cannot be applied to integer operands",
                        op_name(op)
                    ),
                )
                .with_hint(
                    "integers support: +, -, *, /, %, equal, not_equal, <, <=, >, >=",
                )),
            },

            // Double ops
            (LiteralValue::Double(a), LiteralValue::Double(b)) => match op {
                Operator::Add      => Ok(LiteralValue::Double(a + b)),
                Operator::Subtract => Ok(LiteralValue::Double(a - b)),
                Operator::Multiply => Ok(LiteralValue::Double(a * b)),
                Operator::Divide => {
                    if *b == 0.0 {
                        return Err(runtime_err(ErrorCode::DivisionByZero, "division by zero")
                            .with_hint(
                                "check that the divisor is never zero before dividing",
                            ));
                    }
                    Ok(LiteralValue::Double(a / b))
                }
                Operator::Equal        => Ok(LiteralValue::Boolean(a == b)),
                Operator::NotEqual     => Ok(LiteralValue::Boolean(a != b)),
                Operator::Less         => Ok(LiteralValue::Boolean(a < b)),
                Operator::LessEqual    => Ok(LiteralValue::Boolean(a <= b)),
                Operator::Greater      => Ok(LiteralValue::Boolean(a > b)),
                Operator::GreaterEqual => Ok(LiteralValue::Boolean(a >= b)),
                _ => Err(runtime_err(
                    ErrorCode::BadOperatorTypes,
                    format!(
                        "operator '{}' cannot be applied to double operands",
                        op_name(op)
                    ),
                )
                .with_hint(
                    "doubles support: +, -, *, /, equal, not_equal, <, <=, >, >=",
                )),
            },

            // Mixed numeric: normalise to Double and recurse once
            (LiteralValue::Integer(a), LiteralValue::Double(_)) => {
                Self::apply_operator(op, &LiteralValue::Double(*a as f64), rhs)
            }
            (LiteralValue::Double(_), LiteralValue::Integer(b)) => {
                Self::apply_operator(op, lhs, &LiteralValue::Double(*b as f64))
            }

            // String + anything coercions
            (LiteralValue::String(a), LiteralValue::Integer(b)) => match op {
                Operator::Add => Ok(LiteralValue::String(format!("{}{}", a, b))),
                _ => Err(runtime_err(
                    ErrorCode::BadOperatorTypes,
                    format!(
                        "operator '{}' cannot be applied to string + integer",
                        op_name(op)
                    ),
                )
                .with_hint(
                    "only '+' (concatenation) is supported between string and integer",
                )),
            },
            (LiteralValue::Integer(a), LiteralValue::String(b)) => match op {
                Operator::Add => Ok(LiteralValue::String(format!("{}{}", a, b))),
                _ => Err(runtime_err(
                    ErrorCode::BadOperatorTypes,
                    format!(
                        "operator '{}' cannot be applied to integer + string",
                        op_name(op)
                    ),
                )
                .with_hint(
                    "only '+' (concatenation) is supported between integer and string",
                )),
            },
            (LiteralValue::String(a), LiteralValue::Double(b)) => match op {
                Operator::Add => Ok(LiteralValue::String(format!("{}{}", a, b))),
                _ => Err(runtime_err(
                    ErrorCode::BadOperatorTypes,
                    format!(
                        "operator '{}' cannot be applied to string + double",
                        op_name(op)
                    ),
                )
                .with_hint(
                    "only '+' (concatenation) is supported between string and double",
                )),
            },
            (LiteralValue::Double(a), LiteralValue::String(b)) => match op {
                Operator::Add => Ok(LiteralValue::String(format!("{}{}", a, b))),
                _ => Err(runtime_err(
                    ErrorCode::BadOperatorTypes,
                    format!(
                        "operator '{}' cannot be applied to double + string",
                        op_name(op)
                    ),
                )
                .with_hint(
                    "only '+' (concatenation) is supported between double and string",
                )),
            },
            (LiteralValue::String(a), LiteralValue::Boolean(b)) => match op {
                Operator::Add => Ok(LiteralValue::String(format!("{}{}", a, b))),
                _ => Err(runtime_err(
                    ErrorCode::BadOperatorTypes,
                    format!(
                        "operator '{}' cannot be applied to string + boolean",
                        op_name(op)
                    ),
                )
                .with_hint(
                    "only '+' (concatenation) is supported between string and boolean",
                )),
            },
            (LiteralValue::Boolean(a), LiteralValue::String(b)) => match op {
                Operator::Add => Ok(LiteralValue::String(format!("{}{}", a, b))),
                _ => Err(runtime_err(
                    ErrorCode::BadOperatorTypes,
                    format!(
                        "operator '{}' cannot be applied to boolean + string",
                        op_name(op)
                    ),
                )
                .with_hint(
                    "only '+' (concatenation) is supported between boolean and string",
                )),
            },

            _ => Err(runtime_err(
                ErrorCode::BadOperatorTypes,
                format!(
                    "operator '{}' cannot be applied to {:?} and {:?}",
                    op_name(op),
                    lhs.type_name(),
                    rhs.type_name()
                ),
            )
            .with_hint(
                "check that both operands are compatible types for this operator",
            )),
        }
    }
}
