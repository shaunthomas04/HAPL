use crate::ast::{Expr, Operator, LiteralValue, StaticType};
use crate::error::{ErrorCode, HaplError, runtime_err};
use std::collections::HashMap;
use indexmap::IndexMap;
use serde_json;

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
        self.scopes.last_mut().expect("scope stack is empty — this is a bug in the interpreter")
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
                    | (StaticType::String,  LiteralValue::String(_))
                    | (StaticType::Map,     LiteralValue::Map { .. }) => {}
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
                    LiteralValue::List { elements, .. } => {
                        let items: Vec<String> = elements.iter().map(|e| format!("{:?}", e)).collect();
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
        
            // -------------------------
            // List Declaration
            // -------------------------
            Expr::ListDeclaration { name, elem_type, elements } => {
                let mut evaled = Vec::new();
                for elem in elements {
                    let val = bubble!(self.eval_value(elem));
                    evaled.push(val);
                }
                let list = LiteralValue::List {
                    elem_type: elem_type.clone(),
                    elements: evaled,
                };
                self.current_scope().insert(name.clone(), list.clone());
                ControlFlow::Value(list)
            }

            // -------------------------
            // List Access
            // -------------------------
            Expr::ListAccess { list, index } => {
                let list_val = bubble!(self.eval_value(list));
                let index_val = bubble!(self.eval_value(index));

                let idx = match index_val {
                    LiteralValue::Integer(i) => i,
                    other => return ControlFlow::Error(
                        runtime_err(ErrorCode::ListIndexNotInt,
                            format!("list index must be Integer, got {:?}", other.type_name()))
                        .with_hint("use an integer as the list index")
                    ),
                };

                match list_val {
                    LiteralValue::List { elements, .. } => {
                        if idx < 0 || idx as usize >= elements.len() {
                            return ControlFlow::Error(
                                runtime_err(ErrorCode::IndexOutOfBounds,
                                    format!("index {} is out of bounds (length {})", idx, elements.len()))
                                .with_hint("make sure the index is within the list's length")
                            );
                        }
                        ControlFlow::Value(elements[idx as usize].clone())
                    }
                    other => ControlFlow::Error(
                        runtime_err(ErrorCode::IndexOutOfBounds,
                            format!("cannot index into {:?}", other.type_name()))
                        .with_hint("only lists can be indexed")
                    ),
                }
            }

            // -------------------------
            // List Assign
            // -------------------------
            Expr::ListAssign { name, index, value } => {
                let index_val = bubble!(self.eval_value(index));
                let new_val = bubble!(self.eval_value(value));

                let idx = match index_val {
                    LiteralValue::Integer(i) => i,
                    other => return ControlFlow::Error(
                        runtime_err(ErrorCode::ListIndexNotInt,
                            format!("list index must be Integer, got {:?}", other.type_name()))
                        .with_hint("use an integer as the list index")
                    ),
                };

                let list = match self.lookup(name) {
                    Ok(v) => v,
                    Err(e) => return ControlFlow::Error(e),
                };

                match list {
                    LiteralValue::List { elem_type, mut elements } => {
                        if idx < 0 || idx as usize >= elements.len() {
                            return ControlFlow::Error(
                                runtime_err(ErrorCode::IndexOutOfBounds,
                                    format!("index {} is out of bounds (length {})", idx, elements.len()))
                                .with_hint("make sure the index is within the list's length")
                            );
                        }
                        elements[idx as usize] = new_val;
                        let updated = LiteralValue::List { elem_type, elements };
                        if let Err(e) = self.assign(name, updated.clone()) {
                            return ControlFlow::Error(e);
                        }
                        ControlFlow::Value(updated)
                    }
                    other => ControlFlow::Error(
                        runtime_err(ErrorCode::IndexOutOfBounds,
                            format!("cannot index into {:?}", other.type_name()))
                        .with_hint("only lists can be index-assigned")
                    ),
                }
            }

            // -------------------------
            // List Push
            // -------------------------
            Expr::ListPush { name, value } => {
                let new_val = bubble!(self.eval_value(value));

                let list = match self.lookup(name) {
                    Ok(v) => v,
                    Err(e) => return ControlFlow::Error(e),
                };

                match list {
                    LiteralValue::List { elem_type, mut elements } => {
                        elements.push(new_val);
                        let updated = LiteralValue::List { elem_type, elements };
                        if let Err(e) = self.assign(name, updated.clone()) {
                            return ControlFlow::Error(e);
                        }
                        ControlFlow::Value(updated)
                    }
                    other => ControlFlow::Error(
                        runtime_err(ErrorCode::IndexOutOfBounds,
                            format!("cannot push to {:?}", other.type_name()))
                        .with_hint("only lists support push")
                    ),
                }
            }

            // -------------------------
            // List Pop
            // -------------------------
            Expr::ListPop { name } => {
                let list = match self.lookup(name) {
                    Ok(v) => v,
                    Err(e) => return ControlFlow::Error(e),
                };

                match list {
                    LiteralValue::List { elem_type, mut elements } => {
                        if elements.is_empty() {
                            return ControlFlow::Error(
                                runtime_err(ErrorCode::EmptyList,
                                    format!("cannot pop from empty list '{}'", name))
                                .with_hint("check that the list has elements before popping")
                            );
                        }
                        let popped = elements.pop().unwrap();
                        let updated = LiteralValue::List { elem_type, elements };
                        if let Err(e) = self.assign(name, updated) {
                            return ControlFlow::Error(e);
                        }
                        ControlFlow::Value(popped)
                    }
                    other => ControlFlow::Error(
                        runtime_err(ErrorCode::EmptyList,
                            format!("cannot pop from {:?}", other.type_name()))
                        .with_hint("only lists support pop")
                    ),
                }
            }
            
            // -------------------------
            // Map Declaration
            // -------------------------
            Expr::MapDeclaration { name, entries } => {
                let mut evaled = IndexMap::new();
                for (key, value_expr) in entries {
                    let val = bubble!(self.eval_value(value_expr));
                    evaled.insert(key.clone(), val);
                }
                let map = LiteralValue::Map { entries: evaled };
                if !name.is_empty() {
                    self.current_scope().insert(name.clone(), map.clone());
                }
                ControlFlow::Value(map)
            }

            // -------------------------
            // Map Get
            // -------------------------
            Expr::MapGet { map, key } => {
                let map_val = bubble!(self.eval_value(map));
                let key_val = bubble!(self.eval_value(key));

                let key_str = match key_val {
                    LiteralValue::String(s) => s,
                    other => return ControlFlow::Error(
                        runtime_err(
                            ErrorCode::BadOperatorTypes,
                            format!("map key must be a String, got {:?}", other.type_name()),
                        )
                        .with_hint("map keys must always be strings"),
                    ),
                };

                match map_val {
                    LiteralValue::Map { entries } => {
                        match entries.get(&key_str) {
                            Some(val) => ControlFlow::Value(val.clone()),
                            None => ControlFlow::Error(
                                runtime_err(
                                    ErrorCode::VariableNotFound,
                                    format!("key '{}' not found in map", key_str),
                                )
                                .with_hint(format!(
                                    "use map-contains to check if '{}' exists before accessing it",
                                    key_str
                                )),
                            ),
                        }
                    }
                    other => ControlFlow::Error(
                        runtime_err(
                            ErrorCode::BadOperatorTypes,
                            format!("cannot use map-get on {:?}", other.type_name()),
                        )
                        .with_hint("map-get can only be used on a map variable"),
                    ),
                }
            }

            // -------------------------
            // Map Set
            // -------------------------
            Expr::MapSet { name, key, value } => {
                let key_val = bubble!(self.eval_value(key));
                let new_val = bubble!(self.eval_value(value));

                let key_str = match key_val {
                    LiteralValue::String(s) => s,
                    other => return ControlFlow::Error(
                        runtime_err(
                            ErrorCode::BadOperatorTypes,
                            format!("map key must be a String, got {:?}", other.type_name()),
                        )
                        .with_hint("map keys must always be strings"),
                    ),
                };

                let map = match self.lookup(name) {
                    Ok(v)  => v,
                    Err(e) => return ControlFlow::Error(e),
                };

                match map {
                    LiteralValue::Map { mut entries } => {
                        entries.insert(key_str, new_val);
                        let updated = LiteralValue::Map { entries };
                        if let Err(e) = self.assign(name, updated.clone()) {
                            return ControlFlow::Error(e);
                        }
                        ControlFlow::Value(updated)
                    }
                    other => ControlFlow::Error(
                        runtime_err(
                            ErrorCode::BadOperatorTypes,
                            format!("cannot use map-set on {:?}", other.type_name()),
                        )
                        .with_hint("map-set can only be used on a map variable"),
                    ),
                }
            }

            // -------------------------
            // Map Remove
            // -------------------------
            Expr::MapRemove { name, key } => {
                let key_val = bubble!(self.eval_value(key));

                let key_str = match key_val {
                    LiteralValue::String(s) => s,
                    other => return ControlFlow::Error(
                        runtime_err(
                            ErrorCode::BadOperatorTypes,
                            format!("map key must be a String, got {:?}", other.type_name()),
                        )
                        .with_hint("map keys must always be strings"),
                    ),
                };

                let map = match self.lookup(name) {
                    Ok(v)  => v,
                    Err(e) => return ControlFlow::Error(e),
                };

                match map {
                    LiteralValue::Map { mut entries } => {
                        if !entries.contains_key(&key_str) {
                            return ControlFlow::Error(
                                runtime_err(
                                    ErrorCode::VariableNotFound,
                                    format!("key '{}' not found in map '{}'", key_str, name),
                                )
                                .with_hint(format!(
                                    "use map-contains to check if '{}' exists before removing it",
                                    key_str
                                )),
                            );
                        }
                        entries.shift_remove(&key_str);
                        let updated = LiteralValue::Map { entries };
                        if let Err(e) = self.assign(name, updated.clone()) {
                            return ControlFlow::Error(e);
                        }
                        ControlFlow::Value(updated)
                    }
                    other => ControlFlow::Error(
                        runtime_err(
                            ErrorCode::BadOperatorTypes,
                            format!("cannot use map-remove on {:?}", other.type_name()),
                        )
                        .with_hint("map-remove can only be used on a map variable"),
                    ),
                }
            }

            // -------------------------
            // Map Contains
            // -------------------------
            Expr::MapContains { map, key } => {
                let map_val = bubble!(self.eval_value(map));
                let key_val = bubble!(self.eval_value(key));

                let key_str = match key_val {
                    LiteralValue::String(s) => s,
                    other => return ControlFlow::Error(
                        runtime_err(
                            ErrorCode::BadOperatorTypes,
                            format!("map key must be a String, got {:?}", other.type_name()),
                        )
                        .with_hint("map keys must always be strings"),
                    ),
                };

                match map_val {
                    LiteralValue::Map { entries } => {
                        ControlFlow::Value(LiteralValue::Boolean(entries.contains_key(&key_str)))
                    }
                    other => ControlFlow::Error(
                        runtime_err(
                            ErrorCode::BadOperatorTypes,
                            format!("cannot use map-contains on {:?}", other.type_name()),
                        )
                        .with_hint("map-contains can only be used on a map variable"),
                    ),
                }
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
            // HTTP Get
            // -------------------------
            Expr::HttpGet { name, url } => {
                let url_val = bubble!(self.eval_value(url));
                let url_str = match url_val {
                    LiteralValue::String(s) => s,
                    other => return ControlFlow::Error(
                        runtime_err(ErrorCode::HttpRequestFailed,
                            format!("http-get url must be a string, got {:?}", other.type_name()))
                        .with_hint("wrap the url in a string literal")
                    ),
                };

                let result_map = match reqwest::blocking::get(&url_str) {
                    Ok(response) => {
                        match response.json::<serde_json::Value>() {
                            Ok(json) => json_to_literal(json),
                            Err(e) => {
                                let mut entries = IndexMap::new();
                                entries.insert("error".to_string(),
                                    LiteralValue::String(format!("invalid JSON response: {}", e)));
                                LiteralValue::Map { entries }
                            }
                        }
                    }
                    Err(e) => {
                        let mut entries = IndexMap::new();
                        entries.insert("error".to_string(),
                            LiteralValue::String(format!("request failed: {}", e)));
                        LiteralValue::Map { entries }
                    }
                };

                self.current_scope().insert(name.clone(), result_map.clone());
                ControlFlow::Value(result_map)
            }

            // -------------------------
            // HTTP Post
            // -------------------------
            Expr::HttpPost { name, url, body } => {
                let url_val = bubble!(self.eval_value(url));
                let url_str = match url_val {
                    LiteralValue::String(s) => s,
                    other => return ControlFlow::Error(
                        runtime_err(ErrorCode::HttpRequestFailed,
                            format!("http-post url must be a string, got {:?}", other.type_name()))
                        .with_hint("wrap the url in a string literal")
                    ),
                };

                let body_val = bubble!(self.eval_value(body));
                let body_json = match literal_to_json(body_val) {
                    Some(json) => json,
                    None => return ControlFlow::Error(
                        runtime_err(ErrorCode::HttpRequestFailed,
                            "http-post body must be a map")
                        .with_hint("pass a map variable as the body")
                    ),
                };

                let client = reqwest::blocking::Client::new();
                let result_map = match client.post(&url_str).json(&body_json).send() {
                    Ok(response) => {
                        match response.json::<serde_json::Value>() {
                            Ok(json) => json_to_literal(json),
                            Err(e) => {
                                let mut entries = IndexMap::new();
                                entries.insert("error".to_string(),
                                    LiteralValue::String(format!("invalid JSON response: {}", e)));
                                LiteralValue::Map { entries }
                            }
                        }
                    }
                    Err(e) => {
                        let mut entries = IndexMap::new();
                        entries.insert("error".to_string(),
                            LiteralValue::String(format!("request failed: {}", e)));
                        LiteralValue::Map { entries }
                    }
                };

                self.current_scope().insert(name.clone(), result_map.clone());
                ControlFlow::Value(result_map)
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
                Operator::Modulo => {
                    if *b == 0 {
                        return Err(runtime_err(
                            ErrorCode::DivisionByZero,
                            "modulo by zero",
                        )
                        .with_hint("check that the divisor is never zero before using modulo"));
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
        Operator::Modulo       => "%",
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
            LiteralValue::List { .. } => "List",
            LiteralValue::Map { .. }  => "Map",
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

fn json_to_literal(val: serde_json::Value) -> LiteralValue {
    match val {
        serde_json::Value::String(s) => LiteralValue::String(s),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                LiteralValue::Integer(i)
            } else {
                LiteralValue::Double(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::Bool(b) => LiteralValue::Boolean(b),
        serde_json::Value::Object(m) => {
            let mut entries = IndexMap::new();
            for (k, v) in m {
                entries.insert(k, json_to_literal(v));
            }
            LiteralValue::Map { entries }
        }
        serde_json::Value::Array(arr) => {
            let mut entries = IndexMap::new();
            for (i, v) in arr.into_iter().enumerate() {
                entries.insert(i.to_string(), json_to_literal(v));
            }
            LiteralValue::Map { entries }
        }
        serde_json::Value::Null => LiteralValue::String("null".to_string()),
    }
}

fn literal_to_json(val: LiteralValue) -> Option<serde_json::Value> {
    match val {
        LiteralValue::String(s)  => Some(serde_json::Value::String(s)),
        LiteralValue::Integer(n) => Some(serde_json::json!(n)),
        LiteralValue::Double(f)  => Some(serde_json::json!(f)),
        LiteralValue::Boolean(b) => Some(serde_json::Value::Bool(b)),
        LiteralValue::Map { entries } => {
            let mut map = serde_json::Map::new();
            for (k, v) in entries {
                map.insert(k, literal_to_json(v)?);
            }
            Some(serde_json::Value::Object(map))
        }
        LiteralValue::List { .. } => None,
    }
}