use crate::ast::{Expr, Operator};

pub struct Interpreter;

impl Interpreter {
    pub fn eval(expr: &Expr) -> i64 {
        match expr {
            Expr::Number(n) => *n,

            Expr::Operation { op, operands } => {
                let mut values = operands.iter().map(Self::eval);

                let first = values.next().unwrap();

                match op {
                    Operator::Add => {
                        values.fold(first, |acc, x| acc + x)
                    }
                    Operator::Subtract => {
                        values.fold(first, |acc, x| acc - x)
                    }
                    Operator::Multiply => {
                        values.fold(first, |acc, x| acc * x)
                    }
                    Operator::Divide => {
                        values.fold(first, |acc, x| acc / x)
                    }
                }
            }
        }
    }
}
