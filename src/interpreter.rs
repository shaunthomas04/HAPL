use crate::ast::{Expr, Operator, LiteralValue, StaticType};
use crate::error::{ErrorCode, HaplError, runtime_err};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum ControlFlow {
    Value(LiteralValue),
    Return(LiteralValue),
    // Carries a runtime error up the call stack so it can be reported at the
    // top level without unwinding via panic.
    Error(HaplError),
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

    // --------------------------------------------------
    // Scope helpers
    // --------------------------------------------------

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

    fn lookup(&self, name: &str) -> Result<LiteralValue, HaplError> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Ok(val.clone());
            }
        }
        Err(runtime_err(
            ErrorCode::VariableNotFound,
            format!("variable '{}' not found", name),
        )
        .with_hint(format!(
            "make sure '{}' is declared before it is used",
            name
        )))
    }

    fn assign(&mut self, name: &str, val: LiteralValue) -> Result<(), HaplError> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), val);
                return Ok(());
            }
        }
        Err(runtime_err(
            ErrorCode::AssignBeforeDeclare,
            format!("cannot assign to '{}' — variable has not been declared", name),
        )
        .with_hint(format!(
            "declare '{}' with <var class=\"<type>\" id=\"{}\"> before assigning to it",
            name, name
        )))
    }

    // --------------------------------------------------
    // Public entry point — reports errors and exits
    // --------------------------------------------------

    pub fn run(&mut self, program: &[Expr], filename: &str) {
        for expr in program {
            match self.eval(expr) {
                ControlFlow::Error(e) => {
                    e.report_and_exit(filename);
                }
                _ => {}
            }
        }
    }

    // --------------------------------------------------
    // Core evaluator
    // --------------------------------------------------

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
                match self.lookup(name) {
                    Ok(v)  => ControlFlow::Value(v),
                    Err(e) => ControlFlow::Error(e),
                }
            }

            // -------------------------
            // Variable Declaration
            // -------------------------
            Expr::VariableDeclaration { name, var_type, value } => {
                let val = bubble!(self.eval_value(value));

                match (var_type, &val) {
                    (StaticType::Integer, LiteralValue::Integer(_))
                    | (StaticType::Double,  LiteralValue::Double(_))
                    | (StaticType::Boolean, LiteralValue::Boolean(_))
                    | (StaticType::String,  LiteralValue::String(_)) => {}
                    _ => {
                        return ControlFlow::Error(
                            runtime_err(
                                ErrorCode::BadOperatorTypes,
                                format!(
                                    "type mismatch in declaration of '{}': \
                                     expected {:?}, got {:?}",
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
                        LiteralValue::Boolean(b) => {
                            ControlFlow::Value(LiteralValue::Boolean(!b))
                        }
                        _ => ControlFlow::Error(
                            runtime_err(
                                ErrorCode::NotRequiresBool,
                                format!(
                                    "'not' operator requires a boolean, got {:?}",
                                    v.type_name()
                                ),
                            )
                            .with_hint("wrap the operand in a comparison that produces a boolean"),
                        ),
                    };
                }

                let mut vals = Vec::new();
                for o in operands {
                    let v = bubble!(self.eval_value(o));
                    vals.push(v);
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
                        Ok(v)    => v,
                        Err(cf)  => {
                            self.pop_scope();
                            return cf;
                        }
                    };
                    match cond {
                        LiteralValue::Boolean(false) => break,
                        LiteralValue::Boolean(true) => {
                            self.push_scope();
                            let result = self.eval_block(body);
                            self.pop_scope();

                            match result {
                                ControlFlow::Return(v) => {
                                    self.pop_scope();
                                    return ControlFlow::Return(v);
                                }
                                ControlFlow::Error(e) => {
                                    self.pop_scope();
                                    return ControlFlow::Error(e);
                                }
                                ControlFlow::Value(_) => {}
                            }

                            let step = match self.eval_value(increment) {
                                Ok(v)   => v,
                                Err(cf) => {
                                    self.pop_scope();
                                    return cf;
                                }
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
                                        .with_hint("the increment expression must evaluate to an integer"),
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
                                Err(e) => {
                                    self.pop_scope();
                                    return ControlFlow::Error(e);
                                }
                            };

                            if let Err(e) =
                                self.assign(iterator, LiteralValue::Integer(current + step_n))
                            {
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
            Expr::FunctionDeclaration { name, params, body, .. } => {
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
                    .insert(name.clone(), (params.clone(), body.clone()));
                ControlFlow::Value(LiteralValue::Boolean(true))
            }

            // -------------------------
            // Function Call
            // -------------------------
            Expr::FunctionCall { name, args } => {
                let (params, body) = match self.function_table.get(name).cloned() {
                    Some(f) => f,
                    None => {
                        return ControlFlow::Error(
                            runtime_err(
                                ErrorCode::FunctionNotFound,
                                format!("function '{}' has not been declared", name),
                            )
                            .with_hint(format!(
                                "declare '{}' with <div class=\"<type>-function\" id=\"{}\"> \
                                 before calling it",
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
                    let val = bubble!(self.eval_value(arg));
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
                    ControlFlow::Error(e)  => {
                        self.pop_scope();
                        return ControlFlow::Error(e);
                    }
                };

                self.pop_scope();
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
        }
    }

    // --------------------------------------------------
    // Evaluate a block, propagating Return and Error early
    // --------------------------------------------------
    fn eval_block(&mut self, stmts: &[Expr]) -> ControlFlow {
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
    // Evaluate an expression, converting Return/Error into Err(ControlFlow)
    // so callers can bubble them with the `bubble!` macro.
    // --------------------------------------------------
    fn eval_value(&mut self, expr: &Expr) -> Result<LiteralValue, ControlFlow> {
        match self.eval(expr) {
            ControlFlow::Value(v)  => Ok(v),
            other                  => Err(other),
        }
    }

    // --------------------------------------------------
    // Operator application — returns HaplError on type errors
    // --------------------------------------------------
    fn apply_operator(
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
                Operator::Add          => Ok(LiteralValue::Integer(a + b)),
                Operator::Subtract     => Ok(LiteralValue::Integer(a - b)),
                Operator::Multiply     => Ok(LiteralValue::Integer(a * b)),
                Operator::Divide       => {
                    if *b == 0 {
                        return Err(runtime_err(
                            ErrorCode::DivisionByZero,
                            "division by zero",
                        )
                        .with_hint("check that the divisor is never zero before dividing"));
                    }
                    Ok(LiteralValue::Integer(a / b))
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
                .with_hint("integers support: +, -, *, /, equal, not_equal, <, <=, >, >=")),
            },

            // Double ops
            (LiteralValue::Double(a), LiteralValue::Double(b)) => match op {
                Operator::Add          => Ok(LiteralValue::Double(a + b)),
                Operator::Subtract     => Ok(LiteralValue::Double(a - b)),
                Operator::Multiply     => Ok(LiteralValue::Double(a * b)),
                Operator::Divide       => {
                    if *b == 0.0 {
                        return Err(runtime_err(
                            ErrorCode::DivisionByZero,
                            "division by zero",
                        )
                        .with_hint("check that the divisor is never zero before dividing"));
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
                .with_hint("doubles support: +, -, *, /, equal, not_equal, <, <=, >, >=")),
            },

            // Mixed numeric: normalise both to Double and recurse once
            (LiteralValue::Integer(a), LiteralValue::Double(_)) => {
                Self::apply_operator(op, &LiteralValue::Double(*a as f64), rhs)
            }
            (LiteralValue::Double(_), LiteralValue::Integer(b)) => {
                Self::apply_operator(op, lhs, &LiteralValue::Double(*b as f64))
            }

            // String + anything (coerce rhs to string via Display)
            (LiteralValue::String(a), LiteralValue::Integer(b)) => match op {
                Operator::Add => Ok(LiteralValue::String(format!("{}{}", a, b))),
                _ => Err(runtime_err(
                    ErrorCode::BadOperatorTypes,
                    format!("operator '{}' cannot be applied to string + integer", op_name(op)),
                )
                .with_hint("only '+' (concatenation) is supported between string and integer")),
            },
            (LiteralValue::Integer(a), LiteralValue::String(b)) => match op {
                Operator::Add => Ok(LiteralValue::String(format!("{}{}", a, b))),
                _ => Err(runtime_err(
                    ErrorCode::BadOperatorTypes,
                    format!("operator '{}' cannot be applied to integer + string", op_name(op)),
                )
                .with_hint("only '+' (concatenation) is supported between integer and string")),
            },
            (LiteralValue::String(a), LiteralValue::Double(b)) => match op {
                Operator::Add => Ok(LiteralValue::String(format!("{}{}", a, b))),
                _ => Err(runtime_err(
                    ErrorCode::BadOperatorTypes,
                    format!("operator '{}' cannot be applied to string + double", op_name(op)),
                )
                .with_hint("only '+' (concatenation) is supported between string and double")),
            },
            (LiteralValue::Double(a), LiteralValue::String(b)) => match op {
                Operator::Add => Ok(LiteralValue::String(format!("{}{}", a, b))),
                _ => Err(runtime_err(
                    ErrorCode::BadOperatorTypes,
                    format!("operator '{}' cannot be applied to double + string", op_name(op)),
                )
                .with_hint("only '+' (concatenation) is supported between double and string")),
            },
            (LiteralValue::String(a), LiteralValue::Boolean(b)) => match op {
                Operator::Add => Ok(LiteralValue::String(format!("{}{}", a, b))),
                _ => Err(runtime_err(
                    ErrorCode::BadOperatorTypes,
                    format!("operator '{}' cannot be applied to string + boolean", op_name(op)),
                )
                .with_hint("only '+' (concatenation) is supported between string and boolean")),
            },
            (LiteralValue::Boolean(a), LiteralValue::String(b)) => match op {
                Operator::Add => Ok(LiteralValue::String(format!("{}{}", a, b))),
                _ => Err(runtime_err(
                    ErrorCode::BadOperatorTypes,
                    format!("operator '{}' cannot be applied to boolean + string", op_name(op)),
                )
                .with_hint("only '+' (concatenation) is supported between boolean and string")),
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
            .with_hint("check that both operands are compatible types for this operator")),
        }
    }
}

// --------------------------------------------------
// Helpers
// --------------------------------------------------

/// Short operator name used in runtime error messages.
fn op_name(op: &Operator) -> &'static str {
    match op {
        Operator::Add          => "+",
        Operator::Subtract     => "-",
        Operator::Multiply     => "*",
        Operator::Divide       => "/",
        Operator::And          => "&&",
        Operator::Or           => "||",
        Operator::Not          => "!",
        Operator::Equal        => "equal",
        Operator::NotEqual     => "not_equal",
        Operator::Less         => "less",
        Operator::LessEqual    => "less_equal",
        Operator::Greater      => "greater",
        Operator::GreaterEqual => "greater_equal",
    }
}

/// Extension trait so LiteralValue can describe its own type in error messages.
trait TypeName {
    fn type_name(&self) -> &'static str;
}

impl TypeName for LiteralValue {
    fn type_name(&self) -> &'static str {
        match self {
            LiteralValue::Integer(_) => "Integer",
            LiteralValue::Double(_)  => "Double",
            LiteralValue::String(_)  => "String",
            LiteralValue::Boolean(_) => "Boolean",
        }
    }
}

// --------------------------------------------------
// Macro: bubble a Result<LiteralValue, ControlFlow> up.
// On Ok returns the value; on Err immediately returns the ControlFlow
// (which is either a Return or an Error).
// --------------------------------------------------
macro_rules! bubble {
    ($e:expr) => {
        match $e {
            Ok(v)   => v,
            Err(cf) => return cf,
        }
    };
}
use bubble;