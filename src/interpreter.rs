use crate::ast::{Expr, Operator, LiteralValue, StaticType};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum ControlFlow {
    Value(LiteralValue),
    Return(LiteralValue),
}

pub struct Interpreter {
    scopes: Vec<HashMap<String, LiteralValue>>,
    function_table: HashMap<String, (Vec<(String, StaticType)>, Vec<Expr>)>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            function_table: HashMap::new(),
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    fn current_scope(&mut self) -> &mut HashMap<String, LiteralValue> {
        self.scopes.last_mut().unwrap()
    }

    fn lookup(&self, name: &str) -> LiteralValue {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return val.clone();
            }
        }
        panic!("Variable '{}' not found", name);
    }

    fn assign(&mut self, name: &str, val: LiteralValue) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), val);
                return;
            }
        }
        panic!("Variable '{}' assigned before declaration", name);
    }

    pub fn run(&mut self, program: &[Expr]) {
        for expr in program {
            self.eval(expr);
        }
    }

    fn eval(&mut self, expr: &Expr) -> ControlFlow {
        match expr {

            // -------------------------
            // Literal
            // -------------------------
            Expr::Literal(lit) => ControlFlow::Value(lit.clone()),

            // -------------------------
            // Variable Reference
            // -------------------------
            Expr::VariableReference { name } => {
                ControlFlow::Value(self.lookup(name))
            }

            // -------------------------
            // Variable Declaration
            // -------------------------
            Expr::VariableDeclaration { name, var_type, value } => {
                let val = match self.eval_value(value) {
                    Ok(v)   => v,
                    Err(ret) => return ControlFlow::Return(ret),
                };

                match (var_type, &val) {
                    (StaticType::Integer, LiteralValue::Integer(_))
                    | (StaticType::Double,  LiteralValue::Double(_))
                    | (StaticType::Boolean, LiteralValue::Boolean(_))
                    | (StaticType::String,  LiteralValue::String(_)) => {}
                    _ => panic!("Type mismatch in declaration of '{}'", name),
                }

                self.current_scope().insert(name.clone(), val.clone());
                ControlFlow::Value(val)
            }

            // -------------------------
            // Assignment
            // -------------------------
            Expr::Assignment { name, value } => {
                let val = match self.eval_value(value) {
                    Ok(v)    => v,
                    Err(ret) => return ControlFlow::Return(ret),
                };

                self.assign(name, val.clone());
                ControlFlow::Value(val)
            }

            // -------------------------
            // Operation
            // -------------------------
            Expr::Operation { op, operands } => {
                // Unary NOT
                if let Operator::Not = op {
                    let v = match self.eval_value(&operands[0]) {
                        Ok(v)    => v,
                        Err(ret) => return ControlFlow::Return(ret),
                    };
                    return match v {
                        LiteralValue::Boolean(b) => ControlFlow::Value(LiteralValue::Boolean(!b)),
                        _ => panic!("'not' requires a boolean"),
                    };
                }

                let mut vals = Vec::new();
                for o in operands {
                    let v = match self.eval_value(o) {
                        Ok(v)    => v,
                        Err(ret) => return ControlFlow::Return(ret),
                    };
                    vals.push(v);
                }

                let mut result = vals.remove(0);
                for v in vals {
                    result = Self::apply_operator(op, &result, &v);
                }

                ControlFlow::Value(result)
            }

            // -------------------------
            // Print
            // -------------------------
            Expr::Print { value } => {
                let val = match self.eval_value(value) {
                    Ok(v)    => v,
                    Err(ret) => return ControlFlow::Return(ret),
                };
                match &val {
                    LiteralValue::Integer(n) => println!("{}", n),
                    LiteralValue::Double(f)  => println!("{}", f),
                    LiteralValue::String(s)  => println!("{}", s),
                    LiteralValue::Boolean(b) => println!("{}", b),
                }
                ControlFlow::Value(val)
            }

            // -------------------------
            // Conditional
            // -------------------------
            Expr::Conditional { if_blocks, else_block } => {
                for block in if_blocks {
                    let cond = match self.eval_value(&block.condition) {
                        Ok(v)    => v,
                        Err(ret) => return ControlFlow::Return(ret),
                    };
                    if let LiteralValue::Boolean(true) = cond {
                        self.push_scope();
                        let result = self.eval_block(&block.statements);
                        self.pop_scope();
                        return result;
                    }
                }

                if let Some(stmts) = else_block {
                    self.push_scope();
                    let result = self.eval_block(stmts);
                    self.pop_scope();
                    return result;
                }

                ControlFlow::Value(LiteralValue::Boolean(false))
            }

            // -------------------------
            // While Loop
            // -------------------------
            Expr::WhileLoop { condition, body } => {
                loop {
                    let cond = match self.eval_value(condition) {
                        Ok(v)    => v,
                        Err(ret) => return ControlFlow::Return(ret),
                    };
                    match cond {
                        LiteralValue::Boolean(true) => {
                            self.push_scope();
                            let result = self.eval_block(body);
                            self.pop_scope();
                            if let ControlFlow::Return(v) = result {
                                return ControlFlow::Return(v);
                            }
                        }
                        LiteralValue::Boolean(false) => break,
                        _ => panic!("While condition must be boolean"),
                    }
                }
                ControlFlow::Value(LiteralValue::Boolean(false))
            }

            // -------------------------
            // For Loop
            // -------------------------
            Expr::ForLoop { iterator, condition, increment, body } => {
                // FIX: Push a dedicated scope for the for loop so that the iterator
                // variable is contained and does not leak into the surrounding scope
                // after the loop completes.
                self.push_scope();

                // Auto-declare iterator at 0 inside the loop's own scope
                self.current_scope()
                    .insert(iterator.clone(), LiteralValue::Integer(0));

                loop {
                    let cond = match self.eval_value(condition) {
                        Ok(v)    => v,
                        Err(ret) => {
                            self.pop_scope();
                            return ControlFlow::Return(ret);
                        }
                    };
                    match cond {
                        LiteralValue::Boolean(false) => break,
                        LiteralValue::Boolean(true) => {
                            // Body gets its own inner scope
                            self.push_scope();
                            let result = self.eval_block(body);
                            self.pop_scope();

                            if let ControlFlow::Return(v) = result {
                                self.pop_scope(); // pop loop scope before returning
                                return ControlFlow::Return(v);
                            }

                            // Apply increment to the iterator
                            let step = match self.eval_value(increment) {
                                Ok(v)    => v,
                                Err(ret) => {
                                    self.pop_scope();
                                    return ControlFlow::Return(ret);
                                }
                            };
                            let step_n = match step {
                                LiteralValue::Integer(n) => n,
                                _ => panic!("For loop increment must be Integer"),
                            };

                            let current = match self.lookup(iterator) {
                                LiteralValue::Integer(n) => n,
                                _ => panic!("For loop iterator must be Integer"),
                            };

                            self.assign(iterator, LiteralValue::Integer(current + step_n));
                        }
                        _ => panic!("For loop condition must be boolean"),
                    }
                }

                // Pop the for loop's scope — iterator is now gone
                self.pop_scope();

                ControlFlow::Value(LiteralValue::Boolean(false))
            }

            // -------------------------
            // Function Declaration
            // -------------------------
            Expr::FunctionDeclaration { name, params, body, .. } => {
                if self.function_table.contains_key(name) {
                    panic!("Function '{}' already declared", name);
                }
                self.function_table
                    .insert(name.clone(), (params.clone(), body.clone()));
                ControlFlow::Value(LiteralValue::Boolean(true))
            }

            // -------------------------
            // Function Call
            // -------------------------
            Expr::FunctionCall { name, args } => {
                let (params, body) = self
                    .function_table
                    .get(name)
                    .unwrap_or_else(|| panic!("Function '{}' not found", name))
                    .clone();

                if params.len() != args.len() {
                    panic!(
                        "Function '{}' expects {} args, got {}",
                        name,
                        params.len(),
                        args.len()
                    );
                }

                // Evaluate args BEFORE pushing the new scope
                let mut arg_vals = Vec::new();
                for arg in args {
                    let val = match self.eval_value(arg) {
                        Ok(v)    => v,
                        Err(ret) => return ControlFlow::Return(ret),
                    };
                    arg_vals.push(val);
                }

                // New isolated scope for the function
                self.push_scope();

                for ((param_name, _), val) in params.iter().zip(arg_vals) {
                    self.current_scope().insert(param_name.clone(), val);
                }

                let ret = match self.eval_block(&body) {
                    ControlFlow::Return(v) => v,
                    ControlFlow::Value(v)  => v,
                };

                self.pop_scope();

                ControlFlow::Value(ret)
            }

            // -------------------------
            // Return
            // -------------------------
            Expr::Return { value } => {
                let val = match value {
                    Some(expr) => match self.eval_value(expr) {
                        Ok(v)    => v,
                        Err(ret) => return ControlFlow::Return(ret),
                    },
                    None => LiteralValue::Boolean(false),
                };
                ControlFlow::Return(val)
            }
        }
    }

    /// Evaluate a block of statements, propagating Return early.
    fn eval_block(&mut self, stmts: &[Expr]) -> ControlFlow {
        let mut last = ControlFlow::Value(LiteralValue::Boolean(false));
        for stmt in stmts {
            last = self.eval(stmt);
            if let ControlFlow::Return(_) = &last {
                return last;
            }
        }
        last
    }

    /// Evaluate an expression and unwrap its value, propagating Return via Err.
    fn eval_value(&mut self, expr: &Expr) -> Result<LiteralValue, LiteralValue> {
        match self.eval(expr) {
            ControlFlow::Value(v)  => Ok(v),
            ControlFlow::Return(v) => Err(v),
        }
    }

    fn apply_operator(op: &Operator, lhs: &LiteralValue, rhs: &LiteralValue) -> LiteralValue {
        match (lhs, rhs) {
            // Boolean ops
            (LiteralValue::Boolean(a), LiteralValue::Boolean(b)) => match op {
                Operator::And      => LiteralValue::Boolean(*a && *b),
                Operator::Or       => LiteralValue::Boolean(*a || *b),
                Operator::Equal    => LiteralValue::Boolean(a == b),
                Operator::NotEqual => LiteralValue::Boolean(a != b),
                _ => panic!("Invalid operator for booleans"),
            },

            // String concat / compare
            (LiteralValue::String(a), LiteralValue::String(b)) => match op {
                Operator::Add      => LiteralValue::String(format!("{}{}", a, b)),
                Operator::Equal    => LiteralValue::Boolean(a == b),
                Operator::NotEqual => LiteralValue::Boolean(a != b),
                _ => panic!("Invalid operator for strings"),
            },

            // Integer ops
            (LiteralValue::Integer(a), LiteralValue::Integer(b)) => match op {
                Operator::Add          => LiteralValue::Integer(a + b),
                Operator::Subtract     => LiteralValue::Integer(a - b),
                Operator::Multiply     => LiteralValue::Integer(a * b),
                Operator::Divide       => {
                    if *b == 0 { panic!("Division by zero"); }
                    LiteralValue::Integer(a / b)
                }
                Operator::Equal        => LiteralValue::Boolean(a == b),
                Operator::NotEqual     => LiteralValue::Boolean(a != b),
                Operator::Less         => LiteralValue::Boolean(a < b),
                Operator::LessEqual    => LiteralValue::Boolean(a <= b),
                Operator::Greater      => LiteralValue::Boolean(a > b),
                Operator::GreaterEqual => LiteralValue::Boolean(a >= b),
                _ => panic!("Invalid operator for integers"),
            },

            // Double ops
            (LiteralValue::Double(a), LiteralValue::Double(b)) => match op {
                Operator::Add          => LiteralValue::Double(a + b),
                Operator::Subtract     => LiteralValue::Double(a - b),
                Operator::Multiply     => LiteralValue::Double(a * b),
                Operator::Divide       => {
                    if *b == 0.0 { panic!("Division by zero"); }
                    LiteralValue::Double(a / b)
                }
                Operator::Equal        => LiteralValue::Boolean(a == b),
                Operator::NotEqual     => LiteralValue::Boolean(a != b),
                Operator::Less         => LiteralValue::Boolean(a < b),
                Operator::LessEqual    => LiteralValue::Boolean(a <= b),
                Operator::Greater      => LiteralValue::Boolean(a > b),
                Operator::GreaterEqual => LiteralValue::Boolean(a >= b),
                _ => panic!("Invalid operator for doubles"),
            },

            // Mixed numeric: normalize both sides to Double, then recurse once
            (LiteralValue::Integer(a), LiteralValue::Double(_)) => {
                Self::apply_operator(op, &LiteralValue::Double(*a as f64), rhs)
            }
            (LiteralValue::Double(_), LiteralValue::Integer(b)) => {
                Self::apply_operator(op, lhs, &LiteralValue::Double(*b as f64))
            }

            // String + Integer
            (LiteralValue::String(a), LiteralValue::Integer(b)) => match op {
                Operator::Add => LiteralValue::String(format!("{}{}", a, b)),
                _ => panic!("Invalid operator for string+integer"),
            },

            // Integer + String
            (LiteralValue::Integer(a), LiteralValue::String(b)) => match op {
                Operator::Add => LiteralValue::String(format!("{}{}", a, b)),
                _ => panic!("Invalid operator for integer+string"),
            },

            // String + Double
            (LiteralValue::String(a), LiteralValue::Double(b)) => match op {
                Operator::Add => LiteralValue::String(format!("{}{}", a, b)),
                _ => panic!("Invalid operator for string+double"),
            },

            // Double + String
            (LiteralValue::Double(a), LiteralValue::String(b)) => match op {
                Operator::Add => LiteralValue::String(format!("{}{}", a, b)),
                _ => panic!("Invalid operator for double+string"),
            },

            // String + Boolean
            (LiteralValue::String(a), LiteralValue::Boolean(b)) => match op {
                Operator::Add => LiteralValue::String(format!("{}{}", a, b)),
                _ => panic!("Invalid operator for string+boolean"),
            },

            // Boolean + String
            (LiteralValue::Boolean(a), LiteralValue::String(b)) => match op {
                Operator::Add => LiteralValue::String(format!("{}{}", a, b)),
                _ => panic!("Invalid operator for boolean+string"),
            },


            _ => panic!("Unsupported operand types for {:?}", op),
        }
    }
}