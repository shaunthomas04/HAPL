use crate::ast::{Expr, Operator, LiteralValue, StaticType};
use std::collections::HashMap;

pub struct Interpreter {
    runtime_symbol_table: HashMap<String, LiteralValue>, // runtime environment for variables
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            runtime_symbol_table: HashMap::new(),
        }
    }

    pub fn eval(&mut self, expr: &Expr) -> LiteralValue {
        match expr {
            // -------------------------
            // Literal values
            // -------------------------
            Expr::Literal(lit) => lit.clone(),

            // -------------------------
            // Arithmetic operation
            // -------------------------
            Expr::Operation { op, operands } => {
                let mut values: Vec<LiteralValue> =
                    operands.iter().map(|e| self.eval(e)).collect();

                if values.len() < 2 {
                    panic!("Operator requires at least 2 operands");
                }

                // Use first value as starting point
                let mut result = values.remove(0);

                for val in values {
                    result = Self::apply_operator(op, &result, &val);
                }

                result
            }

            // -------------------------
            // Variable declaration
            // -------------------------
            Expr::VariableDeclaration { name, var_type, value } => {
                let val = self.eval(value);

                // Check type consistency
                match (var_type, &val) {
                    (StaticType::Integer, LiteralValue::Integer(_))
                    | (StaticType::Double, LiteralValue::Double(_))
                    | (StaticType::String, LiteralValue::String(_)) => {}
                    _ => panic!(
                        "Type mismatch for variable '{}': declared {:?}, got {:?}",
                        name, var_type, val
                    ),
                }

                // Store in environment
                if self.runtime_symbol_table.contains_key(name) {
                    panic!("Variable '{}' already declared", name);
                }
                self.runtime_symbol_table.insert(name.clone(), val.clone());

                val
            }

            // -------------------------
            // Variable reference
            // -------------------------
            Expr::VariableReference { name } => {
                self.runtime_symbol_table
                    .get(name)
                    .unwrap_or_else(|| panic!("Variable '{}' used before declaration", name))
                    .clone()
            }
        }
    }

    fn apply_operator(op: &Operator, lhs: &LiteralValue, rhs: &LiteralValue) -> LiteralValue {
        match (lhs, rhs) {
            (LiteralValue::Integer(a), LiteralValue::Integer(b)) => match op {
                Operator::Add => LiteralValue::Integer(a + b),
                Operator::Subtract => LiteralValue::Integer(a - b),
                Operator::Multiply => LiteralValue::Integer(a * b),
                Operator::Divide => {
                    if *b == 0 {
                        panic!("Division by zero");
                    }
                    LiteralValue::Integer(a / b)
                }
            },
            (LiteralValue::Double(a), LiteralValue::Double(b)) => match op {
                Operator::Add => LiteralValue::Double(a + b),
                Operator::Subtract => LiteralValue::Double(a - b),
                Operator::Multiply => LiteralValue::Double(a * b),
                Operator::Divide => {
                    if *b == 0.0 {
                        panic!("Division by zero");
                    }
                    LiteralValue::Double(a / b)
                }
            },
            // Mixed types (Integer + Double) → promote to double
            (LiteralValue::Integer(a), LiteralValue::Double(b)) => match op {
                Operator::Add => LiteralValue::Double(*a as f64 + b),
                Operator::Subtract => LiteralValue::Double(*a as f64 - b),
                Operator::Multiply => LiteralValue::Double(*a as f64 * b),
                Operator::Divide => LiteralValue::Double(*a as f64 / b),
            },
            (LiteralValue::Double(a), LiteralValue::Integer(b)) => match op {
                Operator::Add => LiteralValue::Double(a + *b as f64),
                Operator::Subtract => LiteralValue::Double(a - *b as f64),
                Operator::Multiply => LiteralValue::Double(a * *b as f64),
                Operator::Divide => LiteralValue::Double(a / *b as f64),
            },
            (LiteralValue::String(a), LiteralValue::String(b)) => match op {
                Operator::Add => LiteralValue::String(format!("{}{}", a, b)),
                _ => panic!("Only '+' is supported for strings"),
            },
            _ => panic!("Arithmetic operations only allowed on numeric types"),
        }
    }
}
