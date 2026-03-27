mod control_flow;
mod helpers;
mod scope;
mod eval_operators;
mod eval_collections;
mod eval_io;

use std::collections::HashMap;
use crate::ast::{Expr, LiteralValue, Operator, StaticType};
use crate::error::{ErrorCode, runtime_err};
use control_flow::{ControlFlow, bubble};
use helpers::TypeName;

pub struct Interpreter {
    pub(super) scopes:         Vec<HashMap<String, LiteralValue>>,
    pub(super) function_table: HashMap<String, (Vec<(String, StaticType)>, StaticType, Vec<Expr>)>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            scopes:         vec![HashMap::new()],
            function_table: HashMap::new(),
        }
    }

    // --------------------------------------------------
    // Public entry point
    // --------------------------------------------------

    pub fn run(&mut self, program: &[Expr], filename: &str) {
        for expr in program {
            if let ControlFlow::Error(e) = self.eval(expr) {
                e.report_and_exit(filename);
            }
        }
    }

    // --------------------------------------------------
    // Core evaluator — dispatches every Expr variant
    // --------------------------------------------------

    pub(super) fn eval(&mut self, expr: &Expr) -> ControlFlow {
        match expr {
            // -------------------------
            // Literal
            // -------------------------
            Expr::Literal(lit) => ControlFlow::Value(lit.clone()),

            // -------------------------
            // Variable Reference
            // -------------------------
            Expr::VariableReference { name } => match self.lookup(name) {
                Ok(v)  => ControlFlow::Value(v),
                Err(e) => ControlFlow::Error(e),
            },

            // -------------------------
            // Variable Declaration
            // -------------------------
            Expr::VariableDeclaration { name, var_type, value } => {
                let val = bubble!(self.eval_value(value));

                match (var_type, &val) {
                    (StaticType::Integer, LiteralValue::Integer(_))
                    | (StaticType::Double,  LiteralValue::Double(_))
                    | (StaticType::Boolean, LiteralValue::Boolean(_))
                    | (StaticType::String,  LiteralValue::String(_))
                    | (StaticType::Map,     LiteralValue::Map { .. }) => {}
                    _ => {
                        return ControlFlow::Error(
                            runtime_err(
                                ErrorCode::BadOperatorTypes,
                                format!(
                                    "type mismatch in declaration of '{}': expected {:?}, got {:?}",
                                    name, var_type, val.type_name()
                                ),
                            )
                            .with_hint(format!(
                                "the declared type is {:?} — make sure the value matches",
                                var_type
                            )),
                        );
                    }
                }

                self.current_scope().insert(name.clone(), val.clone());
                ControlFlow::Value(val)
            }

            // -------------------------
            // Assignment
            // -------------------------
            Expr::Assignment { name, value } => {
                let val = bubble!(self.eval_value(value));
                match self.assign(name, val.clone()) {
                    Ok(())  => ControlFlow::Value(val),
                    Err(e)  => ControlFlow::Error(e),
                }
            }

            // -------------------------
            // Operation
            // -------------------------
            Expr::Operation { op, operands } => {
                // Unary NOT
                if let Operator::Not = op {
                    let v = bubble!(self.eval_value(&operands[0]));
                    return match v {
                        LiteralValue::Boolean(b) => ControlFlow::Value(LiteralValue::Boolean(!b)),
                        _ => ControlFlow::Error(
                            runtime_err(
                                ErrorCode::NotRequiresBool,
                                format!(
                                    "'not' operator requires a boolean, got {:?}",
                                    v.type_name()
                                ),
                            )
                            .with_hint(
                                "wrap the operand in a comparison that produces a boolean",
                            ),
                        ),
                    };
                }

                // Short-circuit AND: stop at the first false operand
                if let Operator::And = op {
                    for operand in operands {
                        let v = bubble!(self.eval_value(operand));
                        match v {
                            LiteralValue::Boolean(false) => {
                                return ControlFlow::Value(LiteralValue::Boolean(false));
                            }
                            LiteralValue::Boolean(true) => continue,
                            _ => return ControlFlow::Error(
                                runtime_err(
                                    ErrorCode::NotRequiresBool,
                                    format!(
                                        "'&&' operator requires boolean operands, got {:?}",
                                        v.type_name()
                                    ),
                                )
                                .with_hint("all operands of '&&' must be boolean"),
                            ),
                        }
                    }
                    return ControlFlow::Value(LiteralValue::Boolean(true));
                }

                // Short-circuit OR: stop at the first true operand
                if let Operator::Or = op {
                    for operand in operands {
                        let v = bubble!(self.eval_value(operand));
                        match v {
                            LiteralValue::Boolean(true) => {
                                return ControlFlow::Value(LiteralValue::Boolean(true));
                            }
                            LiteralValue::Boolean(false) => continue,
                            _ => return ControlFlow::Error(
                                runtime_err(
                                    ErrorCode::NotRequiresBool,
                                    format!(
                                        "'||' operator requires boolean operands, got {:?}",
                                        v.type_name()
                                    ),
                                )
                                .with_hint("all operands of '||' must be boolean"),
                            ),
                        }
                    }
                    return ControlFlow::Value(LiteralValue::Boolean(false));
                }

                // All other operators: evaluate all operands eagerly
                let mut vals = Vec::new();
                for o in operands {
                    vals.push(bubble!(self.eval_value(o)));
                }

                let mut result = vals.remove(0);
                for v in vals {
                    result = match Self::apply_operator(op, &result, &v) {
                        Ok(val)  => val,
                        Err(e)   => return ControlFlow::Error(e),
                    };
                }
                ControlFlow::Value(result)
            }

            // -------------------------
            // Print
            // -------------------------
            Expr::Print { value } => {
                let val = bubble!(self.eval_value(value));
                match &val {
                    LiteralValue::Integer(n) => println!("{}", n),
                    LiteralValue::Double(f)  => println!("{}", f),
                    LiteralValue::String(s)  => println!("{}", s),
                    LiteralValue::Boolean(b) => println!("{}", b),
                    LiteralValue::List { elements, .. } => {
                        let items: Vec<String> =
                            elements.iter().map(|e| format!("{:?}", e)).collect();
                        println!("[{}]", items.join(", "));
                    }
                    LiteralValue::Map { entries } => {
                        let items: Vec<String> = entries
                            .iter()
                            .map(|(k, v)| format!("{}: {:?}", k, v))
                            .collect();
                        println!("{{{}}}", items.join(", "));
                    }
                }
                ControlFlow::Value(val)
            }

            // -------------------------
            // Conditional
            // -------------------------
            Expr::Conditional { if_blocks, else_block } => {
                for block in if_blocks {
                    let cond = bubble!(self.eval_value(&block.condition));
                    match cond {
                        LiteralValue::Boolean(true) => {
                            self.push_scope();
                            let result = self.eval_block(&block.statements);
                            self.pop_scope();
                            return result;
                        }
                        LiteralValue::Boolean(false) => continue,
                        _ => {
                            return ControlFlow::Error(
                                runtime_err(
                                    ErrorCode::NotRequiresBool,
                                    format!(
                                        "conditional condition must be boolean, got {:?}",
                                        cond.type_name()
                                    ),
                                )
                                .with_hint(
                                    "use a comparison or boolean expression as the condition",
                                ),
                            );
                        }
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
                    let cond = bubble!(self.eval_value(condition));
                    match cond {
                        LiteralValue::Boolean(true) => {
                            self.push_scope();
                            let result = self.eval_block(body);
                            self.pop_scope();
                            match result {
                                ControlFlow::Return(v) => return ControlFlow::Return(v),
                                ControlFlow::Error(e)  => return ControlFlow::Error(e),
                                ControlFlow::Value(_)  => {}
                            }
                        }
                        LiteralValue::Boolean(false) => break,
                        _ => {
                            return ControlFlow::Error(
                                runtime_err(
                                    ErrorCode::NotRequiresBool,
                                    format!(
                                        "while condition must be boolean, got {:?}",
                                        cond.type_name()
                                    ),
                                )
                                .with_hint(
                                    "use a comparison or boolean expression as the while condition",
                                ),
                            );
                        }
                    }
                }
                ControlFlow::Value(LiteralValue::Boolean(false))
            }

            // -------------------------
            // For Loop
            // -------------------------
            Expr::ForLoop { iterator, condition, increment, body } => {
                self.push_scope();
                self.current_scope()
                    .insert(iterator.clone(), LiteralValue::Integer(0));

                loop {
                    let cond = match self.eval_value(condition) {
                        Ok(v)   => v,
                        Err(cf) => { self.pop_scope(); return cf; }
                    };

                    match cond {
                        LiteralValue::Boolean(false) => break,
                        LiteralValue::Boolean(true) => {
                            self.push_scope();
                            let result = self.eval_block(body);
                            self.pop_scope();

                            match result {
                                ControlFlow::Return(v) => { self.pop_scope(); return ControlFlow::Return(v); }
                                ControlFlow::Error(e)  => { self.pop_scope(); return ControlFlow::Error(e); }
                                ControlFlow::Value(_)  => {}
                            }

                            let step = match self.eval_value(increment) {
                                Ok(v)   => v,
                                Err(cf) => { self.pop_scope(); return cf; }
                            };

                            let step_n = match step {
                                LiteralValue::Integer(n) => n,
                                _ => {
                                    self.pop_scope();
                                    return ControlFlow::Error(
                                        runtime_err(
                                            ErrorCode::BadOperatorTypes,
                                            format!(
                                                "for loop increment must be Integer, got {:?}",
                                                step.type_name()
                                            ),
                                        )
                                        .with_hint(
                                            "the increment expression must evaluate to an integer",
                                        ),
                                    );
                                }
                            };

                            let current = match self.lookup(iterator) {
                                Ok(LiteralValue::Integer(n)) => n,
                                Ok(other) => {
                                    self.pop_scope();
                                    return ControlFlow::Error(
                                        runtime_err(
                                            ErrorCode::BadOperatorTypes,
                                            format!(
                                                "for loop iterator '{}' must be Integer, got {:?}",
                                                iterator, other.type_name()
                                            ),
                                        )
                                        .with_hint("for loop iterators must be declared as integer"),
                                    );
                                }
                                Err(e) => { self.pop_scope(); return ControlFlow::Error(e); }
                            };

                            if let Err(e) = self.assign(iterator, LiteralValue::Integer(current + step_n)) {
                                self.pop_scope();
                                return ControlFlow::Error(e);
                            }
                        }
                        _ => {
                            self.pop_scope();
                            return ControlFlow::Error(
                                runtime_err(
                                    ErrorCode::NotRequiresBool,
                                    format!(
                                        "for loop condition must be boolean, got {:?}",
                                        cond.type_name()
                                    ),
                                )
                                .with_hint(
                                    "use a comparison or boolean expression as the for condition",
                                ),
                            );
                        }
                    }
                }

                self.pop_scope();
                ControlFlow::Value(LiteralValue::Boolean(false))
            }

            // -------------------------
            // Function Declaration
            // -------------------------
            Expr::FunctionDeclaration { name, params, return_type, body } => {
                if self.function_table.contains_key(name) {
                    return ControlFlow::Error(
                        runtime_err(
                            ErrorCode::FunctionAlreadyDeclared,
                            format!("function '{}' has already been declared", name),
                        )
                        .with_hint("each function name must be unique within the program"),
                    );
                }
                self.function_table
                    .insert(name.clone(), (params.clone(), return_type.clone(), body.clone()));
                ControlFlow::Value(LiteralValue::Boolean(true))
            }

            // -------------------------
            // Function Call
            // -------------------------
            Expr::FunctionCall { name, args } => {
                let (params, return_type, body) = match self.function_table.get(name).cloned() {
                    Some(f) => f,
                    None => {
                        return ControlFlow::Error(
                            runtime_err(
                                ErrorCode::FunctionNotFound,
                                format!("function '{}' has not been declared", name),
                            )
                            .with_hint(format!(
                                "declare '{}' with <div class=\"<type>-function\" id=\"{}\"> before calling it",
                                name, name
                            )),
                        );
                    }
                };

                if params.len() != args.len() {
                    return ControlFlow::Error(
                        runtime_err(
                            ErrorCode::WrongArgCount,
                            format!(
                                "function '{}' expects {} argument{}, but {} {} provided",
                                name,
                                params.len(),
                                if params.len() == 1 { "" } else { "s" },
                                args.len(),
                                if args.len() == 1 { "was" } else { "were" },
                            ),
                        )
                        .with_hint(format!(
                            "check the declaration of '{}' for its expected parameters",
                            name
                        )),
                    );
                }

                // Evaluate args BEFORE pushing the new scope
                let mut arg_vals = Vec::new();
                for arg in args {
                    arg_vals.push(bubble!(self.eval_value(arg)));
                }

                self.push_scope();
                for ((param_name, _), val) in params.iter().zip(arg_vals) {
                    self.current_scope().insert(param_name.clone(), val);
                }

                let ret = match self.eval_block(&body) {
                    ControlFlow::Return(v) => v,
                    ControlFlow::Value(v)  => v,
                    ControlFlow::Error(e)  => {
                        self.pop_scope();
                        return ControlFlow::Error(e);
                    }
                };

                self.pop_scope();

                // Validate return type at runtime
                if return_type != StaticType::Void {
                    if !helpers::matches_type(&ret, &return_type) {
                        return ControlFlow::Error(
                            runtime_err(
                                ErrorCode::BadOperatorTypes,
                                format!(
                                    "function '{}' declared as {:?} but returned {:?}",
                                    name, return_type, ret.type_name()
                                ),
                            )
                            .with_hint(format!(
                                "make sure all return paths in '{}' return a {:?} value",
                                name, return_type
                            )),
                        );
                    }
                }

                ControlFlow::Value(ret)
            }

            // -------------------------
            // Return
            // -------------------------
            Expr::Return { value } => {
                let val = match value {
                    Some(expr) => bubble!(self.eval_value(expr)),
                    None       => LiteralValue::Boolean(false),
                };
                ControlFlow::Return(val)
            }

            // -------------------------
            // Length
            // -------------------------
            Expr::Length { value } => {
                let val = bubble!(self.eval_value(value));
                match val {
                    LiteralValue::List { elements, .. } => {
                        ControlFlow::Value(LiteralValue::Integer(elements.len() as i64))
                    }
                    LiteralValue::Map { entries } => {
                        ControlFlow::Value(LiteralValue::Integer(entries.len() as i64))
                    }
                    LiteralValue::String(s) => {
                        ControlFlow::Value(LiteralValue::Integer(s.len() as i64))
                    }
                    other => ControlFlow::Error(
                        runtime_err(
                            ErrorCode::BadOperatorTypes,
                            format!("length is not supported for {:?}", other.type_name()),
                        )
                        .with_hint("length can only be used on a list, map, or string"),
                    ),
                }
            }

            // -------------------------
            // Input
            // -------------------------
            Expr::Input => {
                let mut input = String::new();
                match std::io::stdin().read_line(&mut input) {
                    Ok(_)  => ControlFlow::Value(LiteralValue::String(input.trim().to_string())),
                    Err(e) => ControlFlow::Error(
                        runtime_err(
                            ErrorCode::BadOperatorTypes,
                            format!("failed to read input: {}", e),
                        )
                        .with_hint("make sure stdin is available"),
                    ),
                }
            }

            // -------------------------
            // Respond
            // -------------------------
            Expr::Respond { value } => {
                let val = bubble!(self.eval_value(value));
                ControlFlow::Value(val)
            }

            // Delegate collections and IO to their own modules
            Expr::ListDeclaration { .. }
            | Expr::ListAccess { .. }
            | Expr::ListAssign { .. }
            | Expr::ListPush { .. }
            | Expr::ListPop { .. } => self.eval_list(expr),

            Expr::MapDeclaration { .. }
            | Expr::MapGet { .. }
            | Expr::MapSet { .. }
            | Expr::MapRemove { .. }
            | Expr::MapContains { .. } => self.eval_map(expr),

            Expr::HttpGet { .. }  => self.eval_http_get(expr),
            Expr::HttpPost { .. } => self.eval_http_post(expr),
            Expr::ServerDeclaration { .. } => self.eval_server(expr),
        }
    }

    // --------------------------------------------------
    // Evaluate a block, propagating Return and Error early
    // --------------------------------------------------
    pub(super) fn eval_block(&mut self, stmts: &[Expr]) -> ControlFlow {
        let mut last = ControlFlow::Value(LiteralValue::Boolean(false));
        for stmt in stmts {
            last = self.eval(stmt);
            match &last {
                ControlFlow::Return(_) | ControlFlow::Error(_) => return last,
                ControlFlow::Value(_) => {}
            }
        }
        last
    }

    // --------------------------------------------------
    // Evaluate an expression, converting ControlFlow into Err
    // so callers can bubble with the `bubble!` macro.
    // --------------------------------------------------
    pub(super) fn eval_value(&mut self, expr: &Expr) -> Result<LiteralValue, ControlFlow> {
        match self.eval(expr) {
            ControlFlow::Value(v) => Ok(v),
            other                 => Err(other),
        }
    }
}
