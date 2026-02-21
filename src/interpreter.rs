use crate::ast::{Expr, Operator, LiteralValue, StaticType};
use std::collections::HashMap;

pub struct Interpreter {
    runtime_symbol_table: HashMap<String, LiteralValue>,
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
            // Operations (Arithmetic + Boolean)
            // -------------------------
            Expr::Operation { op, operands } => {
                // Special case: unary NOT
                if let Operator::Not = op {
                    if operands.len() != 1 {
                        panic!("'!' operator requires exactly 1 operand");
                    }

                    let value = self.eval(&operands[0]);

                    return match value {
                        LiteralValue::Boolean(b) => LiteralValue::Boolean(!b),
                        _ => panic!("'!' operator only works on booleans"),
                    };
                }

                let mut values: Vec<LiteralValue> =
                    operands.iter().map(|e| self.eval(e)).collect();

                if values.len() < 2 {
                    panic!("Operator requires at least 2 operands");
                }

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

                match (var_type, &val) {
                    (StaticType::Integer, LiteralValue::Integer(_))
                    | (StaticType::Double, LiteralValue::Double(_))
                    | (StaticType::Boolean, LiteralValue::Boolean(_))
                    | (StaticType::String, LiteralValue::String(_)) => {}
                    _ => panic!(
                        "Type mismatch for variable '{}': declared {:?}, got {:?}",
                        name, var_type, val
                    ),
                }

                if self.runtime_symbol_table.contains_key(name) {
                    panic!("Variable '{}' already declared", name);
                }

                self.runtime_symbol_table.insert(name.clone(), val.clone());
                val
            }

            // -------------------------
            // Variable assignment
            // -------------------------
            Expr::Assignment { name, value } => {
                // Evaluate the expression first
                let val = self.eval(value);

                // Get expected type from symbol table
                let expected_type = self
                    .runtime_symbol_table
                    .get(name)
                    .map(|v| Self::expr_type(v))
                    .unwrap_or_else(|| panic!("Variable '{}' assigned before declaration", name));

                // Get actual type
                let value_type = Self::expr_type(&val);

                if expected_type != value_type {
                    panic!(
                        "Type mismatch in assignment to '{}': expected {:?}, got {:?}",
                        name, expected_type, value_type
                    );
                }

                // Update variable in runtime symbol table
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

            // -------------------------
            // Print
            // -------------------------
            Expr::Print { value } => {
                let val = self.eval(value);

                match &val {
                    LiteralValue::Integer(n) => println!("{}", n),
                    LiteralValue::Double(f) => println!("{}", f),
                    LiteralValue::String(s) => println!("{}", s),
                    LiteralValue::Boolean(b) => println!("{}", b),
                }

                val
            }

            // -------------------------
            // Conditional statements
            // -------------------------
            Expr::Conditional { if_blocks, else_block } => {
                for block in if_blocks {
                    let cond_value = self.eval(&block.condition);
                    match cond_value {
                        LiteralValue::Boolean(true) => {
                            let mut last_val = LiteralValue::Boolean(true); // placeholder
                            for stmt in &block.statements {
                                last_val = self.eval(stmt);
                            }
                            return last_val; // Stop after first true condition
                        }
                        LiteralValue::Boolean(false) => continue,
                        _ => panic!("Conditional expression must evaluate to a boolean"),
                    }
                }

                // If no if/elif was true, execute else block if present
                if let Some(stmts) = else_block {
                    let mut last_val = LiteralValue::Boolean(true); // placeholder
                    for stmt in stmts {
                        last_val = self.eval(stmt);
                    }
                    return last_val;
                }

                // Default return if nothing executed
                LiteralValue::Boolean(false)
            }
        
            // -------------------------
            // While loop
            // -------------------------
            Expr::WhileLoop { condition, body } => {
                let mut last_val = LiteralValue::Boolean(false);

                loop {
                    let cond_value = self.eval(condition);

                    match cond_value {
                        LiteralValue::Boolean(true) => {
                            for stmt in body {
                                last_val = self.eval(stmt);
                            }
                        }
                        LiteralValue::Boolean(false) => break,
                        _ => panic!("While condition must evaluate to a boolean"),
                    }
                }
                last_val
            }

            // -------------------------
            // For loop
            // -------------------------
            Expr::ForLoop {
                iterator,
                initializer,
                condition,
                increment,
                body,
            } => {
                let mut last_val = LiteralValue::Boolean(false);

                // Initialize iterator variable if provided
                if let Some(init_expr) = initializer {
                    let val = self.eval(init_expr);
                    self.runtime_symbol_table.insert(iterator.clone(), val);
                } else if !self.runtime_symbol_table.contains_key(iterator) {
                    // Default to 0 if no initializer and iterator doesn't exist yet
                    self.runtime_symbol_table.insert(iterator.clone(), LiteralValue::Integer(0));
                }

                // Loop while condition is true
                loop {
                    let cond_val = self.eval(condition);
                    match cond_val {
                        LiteralValue::Boolean(true) => {
                            // Execute body
                            for stmt in body {
                                last_val = self.eval(stmt);
                            }

                            // Apply increment if present
                            if let Some(inc_expr) = increment {
                                let val = self.eval(inc_expr);
                                self.runtime_symbol_table.insert(iterator.clone(), val);
                            }
                        }
                        LiteralValue::Boolean(false) => break,
                        _ => panic!("For loop condition must evaluate to a boolean"),
                    }
                }

                last_val
            }
        }
    }

    fn literal_to_string(val: &LiteralValue) -> String {
        match val {
            LiteralValue::Integer(n) => n.to_string(),
            LiteralValue::Double(f) => f.to_string(),
            LiteralValue::String(s) => s.clone(),
            LiteralValue::Boolean(b) => b.to_string(),
        }
    }

    fn apply_operator(
        op: &Operator,
        lhs: &LiteralValue,
        rhs: &LiteralValue,
    ) -> LiteralValue {
        // ---------------------------------
        // BOOLEAN OPERATIONS
        // ---------------------------------
        match (lhs, rhs) {
            (LiteralValue::Boolean(a), LiteralValue::Boolean(b)) => {
                match op {
                    Operator::And => return LiteralValue::Boolean(*a && *b),
                    Operator::Or  => return LiteralValue::Boolean(*a || *b),

                    Operator::Equal => return LiteralValue::Boolean(a == b),
                    Operator::NotEqual => return LiteralValue::Boolean(a != b),

                    _ => {}
                }
            }
            _ => {}
        }

        // ---------------------------------
        // STRING CONCATENATION (Add only)
        // ---------------------------------
        if let Operator::Add = op {
            match (lhs, rhs) {
                (LiteralValue::String(a), LiteralValue::String(b)) => {
                    return LiteralValue::String(format!("{}{}", a, b));
                }
                (LiteralValue::String(a), b) => {
                    return LiteralValue::String(format!("{}{}", a, Self::literal_to_string(b)));
                }
                (a, LiteralValue::String(b)) => {
                    return LiteralValue::String(format!("{}{}", Self::literal_to_string(a), b));
                }
                _ => {}
            }
        }

        // ---------------------------------
        // STRING COMPARISON
        // ---------------------------------
        match (lhs, rhs) {
            (LiteralValue::String(a), LiteralValue::String(b)) => match op {
                Operator::Equal => return LiteralValue::Boolean(a == b),
                Operator::NotEqual => return LiteralValue::Boolean(a != b),
                _ => {}
            },
            _ => {}
        }

        // ---------------------------------
        // NUMERIC OPERATIONS
        // ---------------------------------
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
                Operator::Equal => LiteralValue::Boolean(a == b),
                Operator::NotEqual => LiteralValue::Boolean(a != b),
                Operator::Less => LiteralValue::Boolean(a < b),
                Operator::LessEqual => LiteralValue::Boolean(a <= b),
                Operator::Greater => LiteralValue::Boolean(a > b),
                Operator::GreaterEqual => LiteralValue::Boolean(a >= b),

                _ => panic!("Invalid operator for integers"),
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
                Operator::Equal => LiteralValue::Boolean(a == b),
                Operator::NotEqual => LiteralValue::Boolean(a != b),
                Operator::Less => LiteralValue::Boolean(a < b),
                Operator::LessEqual => LiteralValue::Boolean(a <= b),
                Operator::Greater => LiteralValue::Boolean(a > b),
                Operator::GreaterEqual => LiteralValue::Boolean(a >= b),

                _ => panic!("Invalid operator for doubles"),
            },

            // Mixed numeric → promote to double
            (LiteralValue::Integer(a), LiteralValue::Double(b)) => match op {
                Operator::Add => LiteralValue::Double(*a as f64 + b),
                Operator::Subtract => LiteralValue::Double(*a as f64 - b),
                Operator::Multiply => LiteralValue::Double(*a as f64 * b),
                Operator::Divide => LiteralValue::Double(*a as f64 / b),
                Operator::Equal => LiteralValue::Boolean((*a as f64) == *b),
                Operator::NotEqual => LiteralValue::Boolean((*a as f64) != *b),
                Operator::Less => LiteralValue::Boolean((*a as f64) < *b),
                Operator::LessEqual => LiteralValue::Boolean((*a as f64) <= *b),
                Operator::Greater => LiteralValue::Boolean((*a as f64) > *b),
                Operator::GreaterEqual => LiteralValue::Boolean((*a as f64) >= *b),
                _ => panic!("Invalid operator for numeric types"),
            },

            (LiteralValue::Double(a), LiteralValue::Integer(b)) => match op {
                Operator::Add => LiteralValue::Double(a + *b as f64),
                Operator::Subtract => LiteralValue::Double(a - *b as f64),
                Operator::Multiply => LiteralValue::Double(a * *b as f64),
                Operator::Divide => LiteralValue::Double(a / *b as f64),
                Operator::Equal => LiteralValue::Boolean((*b as f64) == *a),
                Operator::NotEqual => LiteralValue::Boolean((*b as f64) != *a),
                Operator::Less => LiteralValue::Boolean((*b as f64) < *a),
                Operator::LessEqual => LiteralValue::Boolean((*b as f64) <= *a),
                Operator::Greater => LiteralValue::Boolean((*b as f64) > *a),
                Operator::GreaterEqual => LiteralValue::Boolean((*b as f64) >= *a),
                _ => panic!("Invalid operator for numeric types"),
            },

            _ => panic!("Invalid operand types for operator {:?}", op),
        }
    }

    // Helper: get the static type of a LiteralValue
    fn expr_type(val: &LiteralValue) -> StaticType {
        match val {
            LiteralValue::Integer(_) => StaticType::Integer,
            LiteralValue::Double(_) => StaticType::Double,
            LiteralValue::String(_) => StaticType::String,
            LiteralValue::Boolean(_) => StaticType::Boolean,
        }
    }
}
