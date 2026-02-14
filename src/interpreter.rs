use crate::ast::{Expr, Operator, LiteralValue, StaticType};
use std::collections::HashMap;

pub struct Interpreter {
    // For now, a simple environment for variable values
    env: HashMap<String, LiteralValue>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
        }
    }

    pub fn eval(&mut self, expr: &Expr) -> LiteralValue {
        match expr {
            Expr::Literal(lit) => lit.clone(),

            Expr::Operation { op, operands } => {
                let mut values = operands.iter().map(|e| self.eval(e));

                // Take first value
                let first = values.next().expect("Operation needs at least one operand");

                // For simplicity, only handle integers here
                if let LiteralValue::Integer(mut acc) = first {
                    for v in values {
                        match (acc, v) {
                            (acc_int, LiteralValue::Integer(n)) => {
                                acc = match op {
                                    Operator::Add => acc_int + n,
                                    Operator::Subtract => acc_int - n,
                                    Operator::Multiply => acc_int * n,
                                    Operator::Divide => acc_int / n,
                                };
                            }
                            _ => panic!("Non-integer operand in arithmetic"),
                        }
                    }
                    LiteralValue::Integer(acc)
                } else {
                    panic!("Non-integer first operand in arithmetic")
                }
            }

            Expr::VariableDeclaration { name, var_type: _, value } => {
                let val = self.eval(value);
                self.env.insert(name.clone(), val.clone());
                val
            }

            Expr::VariableReference { name } => {
                self.env.get(name)
                    .expect(&format!("Undefined variable {}", name))
                    .clone()
            }
        }
    }
}
