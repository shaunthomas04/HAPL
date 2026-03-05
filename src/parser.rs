use crate::lexer::{HaplToken, HaplTokenType, LexerTagType, LoopType};
use crate::ast::{Expr, Operator, StaticType, LiteralValue, ConditionalBlock};
use crate::error::{ErrorCode, HaplError, parser_err};
use std::collections::HashMap;

/// The parameter list and return type registered for a declared function.
#[derive(Debug, Clone)]
struct FunctionSignature {
    params:      Vec<(String, StaticType)>,
    return_type: StaticType,
}

pub struct HaplParser {
    tokens: Vec<HaplToken>,
    position: usize,
    // Scope stack mirrors the interpreter's scopes so that:
    // - nested scopes (functions, loops, conditionals) don't pollute outer scopes
    // - shadowing is allowed inside inner scopes
    scope_stack: Vec<HashMap<String, StaticType>>,
    // Registered function signatures — populated when a FunctionDeclaration is
    // parsed so that call sites can be type-checked at parse time.
    function_signatures: HashMap<String, FunctionSignature>,
    // The return type of the function currently being parsed, used to
    // type-check return statements and catch void-function-as-value misuse.
    current_fn_return_type: Option<StaticType>,
}

impl HaplParser {
    pub fn new(tokens: Vec<HaplToken>) -> Self {
        Self {
            tokens,
            position: 0,
            scope_stack: vec![HashMap::new()], // global scope
            function_signatures: HashMap::new(),
            current_fn_return_type: None,
        }
    }

    // --------------------------------------------------
    // Scope helpers
    // --------------------------------------------------

    fn push_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
    }

    fn declare_var(&mut self, name: String, var_type: StaticType) -> Result<(), HaplError> {
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

    fn lookup_var(&self, name: &str) -> Option<StaticType> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(t.clone());
            }
        }
        None
    }

    fn var_exists(&self, name: &str) -> bool {
        self.lookup_var(name).is_some()
    }

    // --------------------------------------------------
    // Parse a SINGLE expression (for operators / variables / literals)
    // --------------------------------------------------
    pub fn parse_expression(&mut self) -> Result<Expr, HaplError> {
        if self.is_at_end() {
            return Err(parser_err(
                ErrorCode::UnexpectedEof,
                "unexpected end of token stream while parsing expression",
            ));
        }

        match self.current_token() {
            HaplTokenType::Literal(lit) => {
                self.advance();
                Ok(Expr::Literal(lit))
            }

            // -------------------------
            // Operator
            // -------------------------
            HaplTokenType::OpenOperator { name } => {
                let operator = self.map_operator(name);
                self.advance(); // consume OpenOperator

                let mut operands = Vec::new();

                while !self.is_at_end() && !self.check_close_operator(name) {
                    operands.push(self.parse_expression()?);
                }

                if self.is_at_end() {
                    return Err(parser_err(
                        ErrorCode::OperatorNotClosed,
                        format!(
                            "operator '{}' was never closed",
                            self.operator_name_str(name)
                        ),
                    )
                    .with_hint(format!(
                        "add a matching closing tag for the '{}' operator",
                        self.operator_name_str(name)
                    )));
                }

                self.advance(); // consume CloseOperator

                match operator {
                    Operator::Not => {
                        if operands.len() != 1 {
                            return Err(
                                parser_err(
                                    ErrorCode::BadOperandCount,
                                    format!(
                                        "'not' operator requires exactly 1 operand, got {}",
                                        operands.len()
                                    ),
                                )
                                .with_hint("example: <div class=\"!\"><span class=\"boolean\">true</span></div>"),
                            );
                        }
                    }

                    Operator::Equal
                    | Operator::NotEqual
                    | Operator::Less
                    | Operator::LessEqual
                    | Operator::Greater
                    | Operator::GreaterEqual => {
                        if operands.len() != 2 {
                            return Err(
                                parser_err(
                                    ErrorCode::BadOperandCount,
                                    format!(
                                        "'{}' operator requires exactly 2 operands, got {}",
                                        self.operator_name_str(name),
                                        operands.len()
                                    ),
                                )
                                .with_hint("comparison operators must have exactly 2 operands"),
                            );
                        }
                    }

                    _ => {
                        if operands.len() < 2 {
                            return Err(
                                parser_err(
                                    ErrorCode::BadOperandCount,
                                    format!(
                                        "'{}' operator requires at least 2 operands, got {}",
                                        self.operator_name_str(name),
                                        operands.len()
                                    ),
                                )
                                .with_hint("arithmetic and logical operators need at least 2 operands"),
                            );
                        }
                    }
                }

                Ok(Expr::Operation { op: operator, operands })
            }

            // -------------------------
            // Variable Declaration
            // -------------------------
            HaplTokenType::OpenVarDec { var_type, name } => {
                let var_name = name.clone();
                let var_type_copy = var_type;

                self.advance(); // consume OpenVarDec

                let value_expr = Box::new(self.parse_expression()?);

                // Fix 5: reject void function calls as initialiser values
                if let Expr::FunctionCall { name: ref fn_name, .. } = *value_expr {
                    if let Some(sig) = self.function_signatures.get(fn_name) {
                        if sig.return_type == StaticType::Void {
                            return Err(
                                parser_err(
                                    ErrorCode::TypeMismatch,
                                    format!(
                                        "cannot use void function '{}' as a value in declaration of '{}'",
                                        fn_name, var_name
                                    ),
                                )
                                .with_hint(
                                    "void functions do not return a value and cannot be used as expressions",
                                ),
                            );
                        }
                    }
                }

                if self.is_at_end() || !self.check_close_var_dec(&var_type_copy, &var_name) {
                    return Err(
                        parser_err(
                            ErrorCode::TagNotClosed,
                            format!("variable declaration '{}' was never closed", var_name),
                        )
                        .with_hint(format!(
                            "add </var> or a matching CloseVarDec for '{}'",
                            var_name
                        )),
                    );
                }

                self.advance(); // consume CloseVarDec

                // Register in current scope (allows shadowing in inner scopes)
                self.declare_var(var_name.clone(), var_type_copy.clone())?;

                // Type-check literals at parse time
                if let Expr::Literal(ref lit_val) = *value_expr {
                    match (&var_type_copy, lit_val) {
                        (StaticType::Integer, LiteralValue::Integer(_))
                        | (StaticType::Double, LiteralValue::Double(_))
                        | (StaticType::String, LiteralValue::String(_))
                        | (StaticType::Boolean, LiteralValue::Boolean(_)) => {}
                        _ => {
                            return Err(
                                parser_err(
                                    ErrorCode::TypeMismatch,
                                    format!(
                                        "type mismatch in variable '{}' declaration: expected {:?}, got {:?}",
                                        var_name, var_type_copy, lit_val
                                    ),
                                )
                                .with_hint(format!(
                                    "the declared type is {:?} — make sure the value matches",
                                    var_type_copy
                                )),
                            );
                        }
                    }
                }

                Ok(Expr::VariableDeclaration {
                    name: var_name,
                    var_type: var_type_copy,
                    value: value_expr,
                })
            }

            // -------------------------
            // Variable Reference
            // -------------------------
            HaplTokenType::OpenVarRef { name } => {
                let var_name = name.clone();

                self.advance(); // consume OpenVarRef

                if !self.var_exists(&var_name) {
                    return Err(
                        parser_err(
                            ErrorCode::NotDeclared,
                            format!("variable '{}' used before declaration", var_name),
                        )
                        .with_hint(format!(
                            "declare '{}' with <var class=\"<type>\" id=\"{}\"> before using it",
                            var_name, var_name
                        )),
                    );
                }

                if self.is_at_end() || !self.check_close_var_ref(&var_name) {
                    return Err(
                        parser_err(
                            ErrorCode::TagNotClosed,
                            format!("variable reference '{}' was never closed", var_name),
                        )
                        .with_hint(format!("add a matching CloseVarRef for '{}'", var_name)),
                    );
                }

                self.advance(); // consume CloseVarRef

                Ok(Expr::VariableReference { name: var_name })
            }

            // -------------------------
            // Variable Assignment
            // -------------------------
            HaplTokenType::OpenVarAssign { name } => {
                let var_name = name.clone();

                self.advance(); // consume OpenVarAssign

                let expected_type = match self.lookup_var(&var_name) {
                    Some(t) => t,
                    None => {
                        return Err(
                            parser_err(
                                ErrorCode::NotDeclared,
                                format!(
                                    "cannot assign to '{}' — variable has not been declared",
                                    var_name
                                ),
                            )
                            .with_hint(format!(
                                "declare '{}' before assigning to it",
                                var_name
                            )),
                        );
                    }
                };

                let value_expr = Box::new(self.parse_expression()?);

                // Fix 5: reject void function calls as assignment values
                if let Expr::FunctionCall { name: ref fn_name, .. } = *value_expr {
                    if let Some(sig) = self.function_signatures.get(fn_name) {
                        if sig.return_type == StaticType::Void {
                            return Err(
                                parser_err(
                                    ErrorCode::TypeMismatch,
                                    format!(
                                        "cannot use void function '{}' as a value in assignment to '{}'",
                                        fn_name, var_name
                                    ),
                                )
                                .with_hint(
                                    "void functions do not return a value and cannot be used as expressions",
                                ),
                            );
                        }
                    }
                }

                // If literal, type-check immediately
                if let Expr::Literal(ref lit_val) = *value_expr {
                    match (&expected_type, lit_val) {
                        (StaticType::Integer, LiteralValue::Integer(_))
                        | (StaticType::Double, LiteralValue::Double(_))
                        | (StaticType::String, LiteralValue::String(_))
                        | (StaticType::Boolean, LiteralValue::Boolean(_)) => {}
                        _ => {
                            return Err(
                                parser_err(
                                    ErrorCode::TypeMismatch,
                                    format!(
                                        "type mismatch in assignment to '{}': expected {:?}, got {:?}",
                                        var_name, expected_type, lit_val
                                    ),
                                )
                                .with_hint(format!(
                                    "'{}' was declared as {:?} — the assigned value must match",
                                    var_name, expected_type
                                )),
                            );
                        }
                    }
                }

                if self.is_at_end() || !matches!(
                    self.current_token(),
                    HaplTokenType::CloseVarAssign { name: ref n } if n == &var_name
                ) {
                    return Err(
                        parser_err(
                            ErrorCode::TagNotClosed,
                            format!("variable assignment '{}' was never closed", var_name),
                        )
                        .with_hint(format!("add a matching CloseVarAssign for '{}'", var_name)),
                    );
                }

                self.advance(); // consume CloseVarAssign

                Ok(Expr::Assignment {
                    name: var_name,
                    value: value_expr,
                })
            }

            // -------------------------
            // Function Call
            // -------------------------
            HaplTokenType::OpenFunctionCall { .. } => self.parse_function_call(),

            // -------------------------
            // List Declaration
            // -------------------------
            HaplTokenType::OpenListDec { elem_type, name } => {
                let list_name = name.clone();
                let list_elem_type = elem_type.clone();

                self.advance(); // consume OpenListDec

                let mut elements = Vec::new();
                while !self.is_at_end() {
                    if matches!(self.current_token(),
                        HaplTokenType::CloseListDec { name: ref n } if n == &list_name)
                    {
                        break;
                    }
                    let elem = self.parse_expression()?;
                    // Type-check each element at parse time
                    if let Some(elem_type) = self.infer_type(&elem) {
                        if elem_type != list_elem_type {
                            return Err(
                                parser_err(
                                    ErrorCode::ListElementTypeMismatch,
                                    format!(
                                        "list '{}' expects {:?} elements, got {:?}",
                                        list_name, list_elem_type, elem_type
                                    ),
                                )
                                .with_hint(format!(
                                    "all elements in '{}' must be of type {:?}",
                                    list_name, list_elem_type
                                )),
                            );
                        }
                    }
                    elements.push(elem);
                }

                if self.is_at_end() {
                    return Err(
                        parser_err(
                            ErrorCode::TagNotClosed,
                            format!("list declaration '{}' was never closed", list_name),
                        )
                        .with_hint(format!("add a matching closing tag for list '{}'", list_name)),
                    );
                }

                self.advance(); // consume CloseListDec

                self.declare_var(list_name.clone(), StaticType::List(Box::new(list_elem_type.clone())))?;

                Ok(Expr::ListDeclaration {
                    name: list_name,
                    elem_type: list_elem_type,
                    elements,
                })
            }

            // -------------------------
            // List Access (read by index)
            // -------------------------
            HaplTokenType::OpenListAccess => {
                self.advance(); // consume OpenListAccess

                let list_expr = Box::new(self.parse_expression()?);
                let index_expr = Box::new(self.parse_expression()?);

                // Index must be Integer
                if let Some(idx_type) = self.infer_type(&index_expr) {
                    if idx_type != StaticType::Integer {
                        return Err(
                            parser_err(
                                ErrorCode::ListIndexNotInt,
                                format!("list index must be Integer, got {:?}", idx_type),
                            )
                            .with_hint("use an integer literal or integer variable as the index"),
                        );
                    }
                }

                if self.is_at_end() || !matches!(self.current_token(), HaplTokenType::CloseListAccess) {
                    return Err(
                        parser_err(
                            ErrorCode::TagNotClosed,
                            "list access block was never closed",
                        )
                        .with_hint("add a matching closing tag for the <div class=\"index\"> block"),
                    );
                }

                self.advance(); // consume CloseListAccess

                Ok(Expr::ListAccess { list: list_expr, index: index_expr })
            }

            // -------------------------
            // List Assign (write by index)
            // -------------------------
            HaplTokenType::OpenListAssign { name } => {
                let list_name = name.clone();
                self.advance(); // consume OpenListAssign

                let _list_ref = self.parse_expression()?; // the <var> reference
                let index_expr = Box::new(self.parse_expression()?);
                let value_expr = Box::new(self.parse_expression()?);

                if let Some(idx_type) = self.infer_type(&index_expr) {
                    if idx_type != StaticType::Integer {
                        return Err(
                            parser_err(
                                ErrorCode::ListIndexNotInt,
                                format!("list index must be Integer, got {:?}", idx_type),
                            )
                            .with_hint("use an integer literal or integer variable as the index"),
                        );
                    }
                }

                if self.is_at_end() || !matches!(
                    self.current_token(),
                    HaplTokenType::CloseListAssign { name: ref n } if n == &list_name
                ) {
                    return Err(
                        parser_err(
                            ErrorCode::TagNotClosed,
                            format!("list index-assign '{}' was never closed", list_name),
                        )
                        .with_hint("add a matching closing tag for the <div class=\"index-assign\"> block"),
                    );
                }

                self.advance(); // consume CloseListAssign

                Ok(Expr::ListAssign {
                    name: list_name,
                    index: index_expr,
                    value: value_expr,
                })
            }

            // -------------------------
            // List Push
            // -------------------------
            HaplTokenType::OpenListPush { name } => {
                let list_name = name.clone();
                self.advance(); // consume OpenListPush

                let _list_ref = self.parse_expression()?; // the <var> reference
                let value_expr = Box::new(self.parse_expression()?);

                if self.is_at_end() || !matches!(
                    self.current_token(),
                    HaplTokenType::CloseListPush { name: ref n } if n == &list_name
                ) {
                    return Err(
                        parser_err(
                            ErrorCode::TagNotClosed,
                            format!("list push '{}' was never closed", list_name),
                        )
                        .with_hint("add a matching closing tag for the <div class=\"push\"> block"),
                    );
                }

                self.advance(); // consume CloseListPush

                Ok(Expr::ListPush { name: list_name, value: value_expr })
            }

            // -------------------------
            // List Pop
            // -------------------------
            HaplTokenType::OpenListPop { name } => {
                let list_name = name.clone();
                self.advance(); // consume OpenListPop

                if self.is_at_end() || !matches!(
                    self.current_token(),
                    HaplTokenType::CloseListPop { name: ref n } if n == &list_name
                ) {
                    return Err(
                        parser_err(
                            ErrorCode::TagNotClosed,
                            format!("list pop '{}' was never closed", list_name),
                        )
                        .with_hint("add a matching closing tag for the <div class=\"pop\"> block"),
                    );
                }

                self.advance(); // consume CloseListPop

                Ok(Expr::ListPop { name: list_name })
            }

            // -------------------------
            // Map Declaration
            // -------------------------
            HaplTokenType::OpenMapDec { name } => {
                let map_name = name.clone();
                self.advance(); // consume OpenMapDec

                let mut entries = Vec::new();

                while !self.is_at_end() {
                    if matches!(self.current_token(),
                        HaplTokenType::CloseMapDec { name: ref n } if n == &map_name)
                    {
                        break;
                    }

                    // each entry is: OpenMapEntry, value expr, CloseMapEntry
                    let key = match self.current_token() {
                        HaplTokenType::OpenMapEntry { key } => key.clone(),
                        other => {
                            return Err(parser_err(
                                ErrorCode::UnexpectedToken,
                                format!("expected map entry but found '{:?}'", other),
                            )
                            .with_hint("example: <div class=\"age\"><span class=\"integer\">30</span></div>"));
                        }
                    };

                    self.advance(); // consume OpenMapEntry

                    let value_expr = self.parse_expression()?;

                    if !matches!(self.current_token(),
                        HaplTokenType::CloseMapEntry { key: ref k } if k == &key)
                    {
                        return Err(parser_err(
                            ErrorCode::TagNotClosed,
                            format!("map entry '{}' was never closed", key),
                        )
                        .with_hint(format!("add a matching closing tag for entry '{}'", key)));
                    }

                    self.advance(); // consume CloseMapEntry
                    entries.push((key, value_expr));
                }

                if self.is_at_end() {
                    return Err(parser_err(
                        ErrorCode::TagNotClosed,
                        format!("map declaration '{}' was never closed", map_name),
                    )
                    .with_hint(format!("add a matching closing tag for map '{}'", map_name)));
                }

                self.advance(); // consume CloseMapDec

                if !map_name.is_empty() {
                    self.declare_var(map_name.clone(), StaticType::Map)?;
                }
                
                Ok(Expr::MapDeclaration {
                    name: map_name,
                    entries,
                })
            }

            // -------------------------
            // Map Get
            // -------------------------
            HaplTokenType::OpenMapGet => {
                self.advance(); // consume OpenMapGet

                let map_expr = Box::new(self.parse_expression()?);

                // expect OpenMapKey
                if !matches!(self.current_token(), HaplTokenType::OpenMapKey) {
                    return Err(parser_err(
                        ErrorCode::UnexpectedToken,
                        format!("expected <div class=\"key\"> in map-get but found '{:?}'", self.current_token()),
                    )
                    .with_hint("example: <div class=\"key\"><span class=\"string\">\"name\"</span></div>"));
                }
                self.advance(); // consume OpenMapKey

                let key_expr = Box::new(self.parse_expression()?);

                if !matches!(self.current_token(), HaplTokenType::CloseMapKey) {
                    return Err(parser_err(
                        ErrorCode::TagNotClosed,
                        "map-get key block was never closed",
                    )
                    .with_hint("add a matching closing tag for the <div class=\"key\"> block"));
                }
                self.advance(); // consume CloseMapKey

                if !matches!(self.current_token(), HaplTokenType::CloseMapGet) {
                    return Err(parser_err(
                        ErrorCode::TagNotClosed,
                        "map-get block was never closed",
                    )
                    .with_hint("add a matching closing tag for the <div class=\"map-get\"> block"));
                }
                self.advance(); // consume CloseMapGet

                Ok(Expr::MapGet { map: map_expr, key: key_expr })
            }

            // -------------------------
            // Map Contains
            // -------------------------
            HaplTokenType::OpenMapContains => {
                self.advance(); // consume OpenMapContains

                let map_expr = Box::new(self.parse_expression()?);

                if !matches!(self.current_token(), HaplTokenType::OpenMapKey) {
                    return Err(parser_err(
                        ErrorCode::UnexpectedToken,
                        format!("expected <div class=\"key\"> in map-contains but found '{:?}'", self.current_token()),
                    )
                    .with_hint("example: <div class=\"key\"><span class=\"string\">\"name\"</span></div>"));
                }
                self.advance(); // consume OpenMapKey

                let key_expr = Box::new(self.parse_expression()?);

                if !matches!(self.current_token(), HaplTokenType::CloseMapKey) {
                    return Err(parser_err(
                        ErrorCode::TagNotClosed,
                        "map-contains key block was never closed",
                    )
                    .with_hint("add a matching closing tag for the <div class=\"key\"> block"));
                }
                self.advance(); // consume CloseMapKey

                if !matches!(self.current_token(), HaplTokenType::CloseMapContains) {
                    return Err(parser_err(
                        ErrorCode::TagNotClosed,
                        "map-contains block was never closed",
                    )
                    .with_hint("add a matching closing tag for the <div class=\"map-contains\"> block"));
                }
                self.advance(); // consume CloseMapContains

                Ok(Expr::MapContains { map: map_expr, key: key_expr })
            }

            // -------------------------
            // Map Set
            // -------------------------
            HaplTokenType::OpenMapSet { name } => {
                let map_name = name.clone();
                self.advance(); // consume OpenMapSet

                let _map_ref = self.parse_expression()?; // the <var> reference

                if !matches!(self.current_token(), HaplTokenType::OpenMapKey) {
                    return Err(parser_err(
                        ErrorCode::UnexpectedToken,
                        format!("expected <div class=\"key\"> in map-set but found '{:?}'", self.current_token()),
                    )
                    .with_hint("example: <div class=\"key\"><span class=\"string\">\"age\"</span></div>"));
                }
                self.advance(); // consume OpenMapKey

                let key_expr = Box::new(self.parse_expression()?);

                if !matches!(self.current_token(), HaplTokenType::CloseMapKey) {
                    return Err(parser_err(
                        ErrorCode::TagNotClosed,
                        "map-set key block was never closed",
                    )
                    .with_hint("add a matching closing tag for the <div class=\"key\"> block"));
                }
                self.advance(); // consume CloseMapKey

                if !matches!(self.current_token(), HaplTokenType::OpenMapValue) {
                    return Err(parser_err(
                        ErrorCode::UnexpectedToken,
                        format!("expected <div class=\"value\"> in map-set but found '{:?}'", self.current_token()),
                    )
                    .with_hint("example: <div class=\"value\"><span class=\"integer\">31</span></div>"));
                }
                self.advance(); // consume OpenMapValue

                let value_expr = Box::new(self.parse_expression()?);

                if !matches!(self.current_token(), HaplTokenType::CloseMapValue) {
                    return Err(parser_err(
                        ErrorCode::TagNotClosed,
                        "map-set value block was never closed",
                    )
                    .with_hint("add a matching closing tag for the <div class=\"value\"> block"));
                }
                self.advance(); // consume CloseMapValue

                if !matches!(self.current_token(),
                    HaplTokenType::CloseMapSet { name: ref n } if n == &map_name)
                {
                    return Err(parser_err(
                        ErrorCode::TagNotClosed,
                        format!("map-set '{}' was never closed", map_name),
                    )
                    .with_hint(format!("add a matching closing tag for the map-set block on '{}'", map_name)));
                }
                self.advance(); // consume CloseMapSet

                Ok(Expr::MapSet { name: map_name, key: key_expr, value: value_expr })
            }

            // -------------------------
            // Map Remove
            // -------------------------
            HaplTokenType::OpenMapRemove { name } => {
                let map_name = name.clone();
                self.advance(); // consume OpenMapRemove

                let _map_ref = self.parse_expression()?; // the <var> reference

                if !matches!(self.current_token(), HaplTokenType::OpenMapKey) {
                    return Err(parser_err(
                        ErrorCode::UnexpectedToken,
                        format!("expected <div class=\"key\"> in map-remove but found '{:?}'", self.current_token()),
                    )
                    .with_hint("example: <div class=\"key\"><span class=\"string\">\"age\"</span></div>"));
                }
                self.advance(); // consume OpenMapKey

                let key_expr = Box::new(self.parse_expression()?);

                if !matches!(self.current_token(), HaplTokenType::CloseMapKey) {
                    return Err(parser_err(
                        ErrorCode::TagNotClosed,
                        "map-remove key block was never closed",
                    )
                    .with_hint("add a matching closing tag for the <div class=\"key\"> block"));
                }
                self.advance(); // consume CloseMapKey

                if !matches!(self.current_token(),
                    HaplTokenType::CloseMapRemove { name: ref n } if n == &map_name)
                {
                    return Err(parser_err(
                        ErrorCode::TagNotClosed,
                        format!("map-remove '{}' was never closed", map_name),
                    )
                    .with_hint(format!("add a matching closing tag for the map-remove block on '{}'", map_name)));
                }
                self.advance(); // consume CloseMapRemove

                Ok(Expr::MapRemove { name: map_name, key: key_expr })
            }

            other => Err(
                parser_err(
                    ErrorCode::UnexpectedToken,
                    format!("unexpected token '{:?}' at position {}", other, self.position),
                )
                .with_hint("expected a literal, variable, operator, or function call"),
            ),
        }
    }

    // --------------------------------------------------
    // Parse a single top-level statement
    // --------------------------------------------------
    pub fn parse_statement(&mut self) -> Result<Expr, HaplError> {
        match self.current_token() {
            // -------------------------
            // Print statement
            // -------------------------
            HaplTokenType::OpenPrint { .. } => {
                self.advance(); // consume <p>

                let inner_expr = self.parse_expression()?;

                if self.is_at_end() || !matches!(self.current_token(), HaplTokenType::ClosePrint { .. }) {
                    return Err(
                        parser_err(
                            ErrorCode::TagNotClosed,
                            "print statement was never closed",
                        )
                        .with_hint("add a matching </p> closing tag"),
                    );
                }

                self.advance(); // consume </p>
                Ok(Expr::Print { value: Box::new(inner_expr) })
            }

            // -------------------------
            // Conditional statement
            // -------------------------
            HaplTokenType::OpenConditional => self.parse_conditional(),

            // -------------------------
            // While statement
            // -------------------------
            HaplTokenType::OpenLoop { loop_type } if loop_type == LoopType::While => self.parse_while(),

            // -------------------------
            // For statement
            // -------------------------
            HaplTokenType::OpenLoop { loop_type } if loop_type == LoopType::For => self.parse_for(),

            // -------------------------
            // Function declaration
            // -------------------------
            HaplTokenType::OpenFunction { .. } => self.parse_function(),
            HaplTokenType::OpenReturn => self.parse_return(),

            // Everything else: parse as expression
            HaplTokenType::OpenVarDec { .. }
            | HaplTokenType::OpenOperator { .. }
            | HaplTokenType::Literal(_)
            | HaplTokenType::OpenVarRef { .. }
            | HaplTokenType::OpenVarAssign { .. }
            | HaplTokenType::OpenFunctionCall { .. } => self.parse_expression(),
            | HaplTokenType::OpenListDec { .. }
            | HaplTokenType::OpenListAccess
            | HaplTokenType::OpenListAssign { .. }
            | HaplTokenType::OpenListPush { .. }
            | HaplTokenType::OpenListPop { .. } => self.parse_expression(),

            // Structural HTML wrapper tokens are skipped
            HaplTokenType::OpenHtmlTag { .. } | HaplTokenType::CloseHtmlTag { .. } => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Boolean(false)))
            }

            | HaplTokenType::OpenMapDec { .. }
            | HaplTokenType::OpenMapGet
            | HaplTokenType::OpenMapSet { .. }
            | HaplTokenType::OpenMapRemove { .. }
            | HaplTokenType::OpenMapContains => self.parse_expression(),


            other => Err(
                parser_err(
                    ErrorCode::UnexpectedToken,
                    format!("unexpected token '{:?}' at top-level position {}", other, self.position),
                )
                .with_hint(
                    "expected a statement: variable declaration, print, conditional, loop, or function",
                ),
            ),
        }
    }

    // --------------------------------------------------
    // Parse a conditional statement
    // --------------------------------------------------
    fn parse_conditional(&mut self) -> Result<Expr, HaplError> {
        if !matches!(self.current_token(), HaplTokenType::OpenConditional) {
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!(
                    "expected <conditional> but found '{:?}'",
                    self.current_token()
                ),
            ));
        }

        self.advance(); // consume OpenConditional

        let mut if_blocks = Vec::new();
        let mut else_block: Option<Vec<Expr>> = None;

        while !matches!(self.current_token(), HaplTokenType::CloseConditional) {
            match self.current_token() {
                HaplTokenType::OpenIf => {
                    if_blocks.push(self.parse_conditional_block(
                        HaplTokenType::OpenIf,
                        HaplTokenType::CloseIf,
                    )?);
                }
                HaplTokenType::OpenElif => {
                    if_blocks.push(self.parse_conditional_block(
                        HaplTokenType::OpenElif,
                        HaplTokenType::CloseElif,
                    )?);
                }
                HaplTokenType::OpenElse => {
                    else_block = Some(self.parse_else_block()?);
                }
                other => {
                    return Err(
                        parser_err(
                            ErrorCode::UnexpectedToken,
                            format!(
                                "unexpected token '{:?}' inside <conditional>",
                                other
                            ),
                        )
                        .with_hint("valid children of <conditional>: <if>, <elif>, <else>"),
                    );
                }
            }
        }

        self.advance(); // consume CloseConditional

        Ok(Expr::Conditional { if_blocks, else_block })
    }

    fn parse_conditional_block(
        &mut self,
        _open: HaplTokenType,
        close: HaplTokenType,
    ) -> Result<ConditionalBlock, HaplError> {
        self.advance(); // consume OpenIf or OpenElif

        self.push_scope();

        let condition = self.parse_expression()?;

        let mut statements = Vec::new();
        while !self.is_at_end() {
            let at_close = match &close {
                HaplTokenType::CloseIf   => matches!(self.current_token(), HaplTokenType::CloseIf),
                HaplTokenType::CloseElif => matches!(self.current_token(), HaplTokenType::CloseElif),
                _ => false,
            };
            if at_close { break; }
            statements.push(self.parse_statement()?);
        }

        if self.is_at_end() {
            self.pop_scope();
            return Err(
                parser_err(
                    ErrorCode::TagNotClosed,
                    "conditional block (if/elif) was never closed",
                )
                .with_hint("add the matching closing tag for the if or elif block"),
            );
        }

        self.pop_scope();
        self.advance(); // consume CloseIf or CloseElif

        Ok(ConditionalBlock { condition, statements })
    }

    fn parse_else_block(&mut self) -> Result<Vec<Expr>, HaplError> {
        self.advance(); // consume OpenElse
        self.push_scope();

        let mut statements = Vec::new();
        while !self.is_at_end() && !matches!(self.current_token(), HaplTokenType::CloseElse) {
            statements.push(self.parse_statement()?);
        }

        if self.is_at_end() {
            self.pop_scope();
            return Err(
                parser_err(
                    ErrorCode::TagNotClosed,
                    "else block was never closed",
                )
                .with_hint("add a matching closing tag for the else block"),
            );
        }

        self.pop_scope();
        self.advance(); // consume CloseElse
        Ok(statements)
    }

    // --------------------------------------------------
    // Parse while loop
    // --------------------------------------------------
    fn parse_while(&mut self) -> Result<Expr, HaplError> {
        if let HaplTokenType::OpenLoop { loop_type } = self.current_token() {
            if loop_type != LoopType::While {
                return Err(parser_err(
                    ErrorCode::UnexpectedToken,
                    format!("expected a while loop but found '{:?}'", self.current_token()),
                ));
            }
        } else {
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!("expected OpenLoop but found '{:?}'", self.current_token()),
            ));
        }

        self.advance(); // consume OpenLoop

        if !matches!(self.current_token(), HaplTokenType::OpenLoopCondition) {
            return Err(
                parser_err(
                    ErrorCode::UnexpectedToken,
                    format!(
                        "expected <condition> inside while loop but found '{:?}'",
                        self.current_token()
                    ),
                )
                .with_hint("while loops must have a <div class=\"condition\"> block"),
            );
        }
        self.advance(); // consume OpenLoopCondition

        let condition_expr = self.parse_expression()?;

        if !matches!(self.current_token(), HaplTokenType::CloseLoopCondition) {
            return Err(
                parser_err(
                    ErrorCode::TagNotClosed,
                    "while loop condition was never closed",
                )
                .with_hint("add </div> to close the <div class=\"condition\"> block"),
            );
        }
        self.advance(); // consume CloseLoopCondition

        if !matches!(self.current_token(), HaplTokenType::OpenLoopBody) {
            return Err(
                parser_err(
                    ErrorCode::UnexpectedToken,
                    format!(
                        "expected <body> inside while loop but found '{:?}'",
                        self.current_token()
                    ),
                )
                .with_hint("while loops must have a <div class=\"body\"> block"),
            );
        }
        self.advance(); // consume OpenLoopBody

        self.push_scope();
        let mut body_statements = Vec::new();
        while !self.is_at_end() && !matches!(self.current_token(), HaplTokenType::CloseLoopBody) {
            body_statements.push(self.parse_statement()?);
        }

        if self.is_at_end() {
            self.pop_scope();
            return Err(
                parser_err(
                    ErrorCode::TagNotClosed,
                    "while loop body was never closed",
                )
                .with_hint("add </div> to close the <div class=\"body\"> block"),
            );
        }

        self.pop_scope();
        self.advance(); // consume CloseLoopBody

        if let HaplTokenType::CloseLoop { loop_type } = self.current_token() {
            if loop_type != LoopType::While {
                return Err(parser_err(
                    ErrorCode::TagNotClosed,
                    "expected closing tag for while loop but found a different loop type",
                ));
            }
        } else {
            return Err(
                parser_err(
                    ErrorCode::TagNotClosed,
                    "while loop was never closed",
                )
                .with_hint("add a matching closing tag for the <div class=\"while\"> block"),
            );
        }
        self.advance(); // consume CloseLoop

        Ok(Expr::WhileLoop {
            condition: Box::new(condition_expr),
            body: body_statements,
        })
    }

    // --------------------------------------------------
    // Parse for loop
    // --------------------------------------------------
    fn parse_for(&mut self) -> Result<Expr, HaplError> {
        if let HaplTokenType::OpenLoop { loop_type } = self.current_token() {
            if loop_type != LoopType::For {
                return Err(parser_err(
                    ErrorCode::UnexpectedToken,
                    format!("expected a for loop but found '{:?}'", self.current_token()),
                ));
            }
        } else {
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!("expected OpenLoop(For) but found '{:?}'", self.current_token()),
            ));
        }

        self.advance(); // consume OpenLoop

        self.push_scope();

        // Parse Iterator
        if !matches!(self.current_token(), HaplTokenType::OpenLoopIterator) {
            self.pop_scope();
            return Err(
                parser_err(
                    ErrorCode::UnexpectedToken,
                    format!(
                        "expected <iterator> inside for loop but found '{:?}'",
                        self.current_token()
                    ),
                )
                .with_hint("for loops must have a <div class=\"iterator\"> block"),
            );
        }
        self.advance(); // consume OpenLoopIterator

        let iterator_expr = self.parse_expression()?;

        let iterator_name = match &iterator_expr {
            Expr::VariableReference { name } => name.clone(),
            _ => {
                self.pop_scope();
                return Err(
                    parser_err(
                        ErrorCode::ForIteratorNotInt,
                        "for loop iterator must be a variable reference",
                    )
                    .with_hint("example: <div class=\"iterator\"><var class=\"i\"></var></div>"),
                );
            }
        };

        // Auto-declare iterator in the loop's own scope if not already there
        if self.scope_stack.last().map_or(true, |s| !s.contains_key(&iterator_name)) {
            self.declare_var(iterator_name.clone(), StaticType::Integer)?;
        }

        // Enforce iterator is Integer
        match self.lookup_var(&iterator_name) {
            Some(t) if t != StaticType::Integer => {
                self.pop_scope();
                return Err(
                    parser_err(
                        ErrorCode::ForIteratorNotInt,
                        format!(
                            "for loop iterator '{}' must be of type Integer, got {:?}",
                            iterator_name, t
                        ),
                    )
                    .with_hint("declare the iterator variable as an integer before the loop"),
                );
            }
            _ => {}
        }

        if !matches!(self.current_token(), HaplTokenType::CloseLoopIterator) {
            self.pop_scope();
            return Err(
                parser_err(
                    ErrorCode::TagNotClosed,
                    "for loop iterator block was never closed",
                )
                .with_hint("add </div> to close the <div class=\"iterator\"> block"),
            );
        }
        self.advance(); // consume CloseLoopIterator

        // Parse Condition
        if !matches!(self.current_token(), HaplTokenType::OpenLoopCondition) {
            self.pop_scope();
            return Err(
                parser_err(
                    ErrorCode::UnexpectedToken,
                    format!(
                        "expected <condition> inside for loop but found '{:?}'",
                        self.current_token()
                    ),
                )
                .with_hint("for loops must have a <div class=\"condition\"> block"),
            );
        }
        self.advance(); // consume OpenLoopCondition

        let condition_expr = self.parse_expression()?;

        if let Some(cond_type) = self.infer_type(&condition_expr) {
            if cond_type != StaticType::Boolean {
                self.pop_scope();
                return Err(
                    parser_err(
                        ErrorCode::ForConditionNotBool,
                        format!(
                            "for loop condition must evaluate to Boolean, got {:?}",
                            cond_type
                        ),
                    )
                    .with_hint(
                        "use a comparison operator (e.g. less, greater) to produce a Boolean",
                    ),
                );
            }
        }

        if !matches!(self.current_token(), HaplTokenType::CloseLoopCondition) {
            self.pop_scope();
            return Err(
                parser_err(
                    ErrorCode::TagNotClosed,
                    "for loop condition block was never closed",
                )
                .with_hint("add </div> to close the <div class=\"condition\"> block"),
            );
        }
        self.advance(); // consume CloseLoopCondition

        // Parse Increment
        if !matches!(self.current_token(), HaplTokenType::OpenLoopIncrement) {
            self.pop_scope();
            return Err(
                parser_err(
                    ErrorCode::UnexpectedToken,
                    format!(
                        "expected <increment> inside for loop but found '{:?}'",
                        self.current_token()
                    ),
                )
                .with_hint("for loops must have a <div class=\"increment\"> block"),
            );
        }
        self.advance(); // consume OpenLoopIncrement

        let increment_expr = self.parse_expression()?;

        if let Some(inc_type) = self.infer_type(&increment_expr) {
            if inc_type != StaticType::Integer {
                self.pop_scope();
                return Err(
                    parser_err(
                        ErrorCode::ForIncrementNotInt,
                        format!(
                            "for loop increment must evaluate to Integer, got {:?}",
                            inc_type
                        ),
                    )
                    .with_hint("the increment expression must produce an integer value"),
                );
            }
        }

        if !matches!(self.current_token(), HaplTokenType::CloseLoopIncrement) {
            self.pop_scope();
            return Err(
                parser_err(
                    ErrorCode::TagNotClosed,
                    "for loop increment block was never closed",
                )
                .with_hint("add </div> to close the <div class=\"increment\"> block"),
            );
        }
        self.advance(); // consume CloseLoopIncrement

        // Parse Body
        if !matches!(self.current_token(), HaplTokenType::OpenLoopBody) {
            self.pop_scope();
            return Err(
                parser_err(
                    ErrorCode::UnexpectedToken,
                    format!(
                        "expected <body> inside for loop but found '{:?}'",
                        self.current_token()
                    ),
                )
                .with_hint("for loops must have a <div class=\"body\"> block"),
            );
        }
        self.advance(); // consume OpenLoopBody

        let mut body_statements = Vec::new();
        while !self.is_at_end() && !matches!(self.current_token(), HaplTokenType::CloseLoopBody) {
            body_statements.push(self.parse_statement()?);
        }

        if self.is_at_end() {
            self.pop_scope();
            return Err(
                parser_err(
                    ErrorCode::TagNotClosed,
                    "for loop body was never closed",
                )
                .with_hint("add </div> to close the <div class=\"body\"> block"),
            );
        }

        self.advance(); // consume CloseLoopBody

        self.pop_scope();

        if let HaplTokenType::CloseLoop { loop_type } = self.current_token() {
            if loop_type != LoopType::For {
                return Err(parser_err(
                    ErrorCode::TagNotClosed,
                    "expected closing tag for for loop but found a different loop type",
                ));
            }
        } else {
            return Err(
                parser_err(
                    ErrorCode::TagNotClosed,
                    "for loop was never closed",
                )
                .with_hint("add a matching closing tag for the <div class=\"for\"> block"),
            );
        }
        self.advance(); // consume CloseLoop

        Ok(Expr::ForLoop {
            iterator: iterator_name,
            condition: Box::new(condition_expr),
            increment: Box::new(increment_expr),
            body: body_statements,
        })
    }

    // --------------------------------------------------
    // Parse an entire program (ALL top-level statements)
    // --------------------------------------------------
    pub fn parse_program(&mut self) -> Result<Vec<Expr>, HaplError> {
        // PASS 1 — Pre-scan all top-level function signatures so that forward
        // calls (call site before the declaration in source order) are
        // type-checked in pass 2 just as well as backward calls.
        self.prescan_function_signatures();

        // PASS 2 — Full parse.
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    // --------------------------------------------------
    // Pass-1 helper: scan tokens for OpenFunction tags and register their
    // signatures without consuming tokens — position is saved and restored.
    // --------------------------------------------------
    fn prescan_function_signatures(&mut self) {
        let mut i = 0usize;
        while i < self.tokens.len() {
            if let HaplTokenType::OpenFunction { ref name, return_type } =
                self.tokens[i].token_type.clone()
            {
                let fn_name = name.clone();
                let mut params: Vec<(String, StaticType)> = Vec::new();

                let mut j = i + 1;

                // Skip to OpenParams
                while j < self.tokens.len()
                    && !matches!(self.tokens[j].token_type, HaplTokenType::OpenParams)
                {
                    j += 1;
                }
                j += 1; // skip OpenParams itself

                // Collect params until CloseParams
                while j < self.tokens.len()
                    && !matches!(self.tokens[j].token_type, HaplTokenType::CloseParams)
                {
                    if let HaplTokenType::OpenParam { ref name, param_type } =
                        self.tokens[j].token_type.clone()
                    {
                        params.push((name.clone(), param_type));
                    }
                    j += 1;
                }

                // entry().or_insert so the real parse_function pass still wins
                self.function_signatures
                    .entry(fn_name)
                    .or_insert(FunctionSignature { params, return_type });
            }
            i += 1;
        }
        // position is unchanged — pass 2 starts from 0
    }

    // --------------------------------------------------
    // Parse a function declaration
    // --------------------------------------------------
    fn parse_function(&mut self) -> Result<Expr, HaplError> {
        let (name, return_type) = match self.current_token() {
            HaplTokenType::OpenFunction { name, return_type } => (name.clone(), return_type.clone()),
            other => {
                return Err(parser_err(
                    ErrorCode::UnexpectedToken,
                    format!("expected function declaration but found '{:?}'", other),
                ));
            }
        };

        self.advance(); // consume OpenFunction

        // Parse Params
        if !matches!(self.current_token(), HaplTokenType::OpenParams) {
            return Err(
                parser_err(
                    ErrorCode::UnexpectedToken,
                    format!(
                        "expected <params> in function '{}' but found '{:?}'",
                        name,
                        self.current_token()
                    ),
                )
                .with_hint(format!(
                    "add a <div class=\"params\"> block inside function '{}'",
                    name
                )),
            );
        }
        self.advance();

        let mut params = Vec::new();

        while !matches!(self.current_token(), HaplTokenType::CloseParams) {
            match self.current_token() {
                HaplTokenType::OpenParam { name: param_name, param_type } => {
                    let param_name = param_name.clone();
                    let param_type_copy = param_type;

                    self.advance(); // consume OpenParam

                    if !matches!(
                        self.current_token(),
                        HaplTokenType::CloseParam { name: ref n } if n == &param_name
                    ) {
                        return Err(
                            parser_err(
                                ErrorCode::TagNotClosed,
                                format!("parameter '{}' was never closed", param_name),
                            )
                            .with_hint(format!(
                                "add a matching </div> closing tag for parameter '{}'",
                                param_name
                            )),
                        );
                    }

                    self.advance(); // consume CloseParam

                    params.push((param_name, param_type_copy));
                }
                other => {
                    return Err(
                        parser_err(
                            ErrorCode::UnexpectedToken,
                            format!("unexpected token '{:?}' inside <params>", other),
                        )
                        .with_hint(
                            "params may only contain parameter declarations, \
                             e.g. <div class=\"integer-param\" id=\"x\"></div>",
                        ),
                    );
                }
            }
        }

        self.advance(); // consume CloseParams

        // Register the signature NOW — before parsing the body — so that
        // recursive calls inside the body can also be type-checked.
        self.function_signatures.insert(
            name.clone(),
            FunctionSignature {
                params:      params.clone(),
                return_type: return_type.clone(),
            },
        );

        // Parse Body
        if !matches!(self.current_token(), HaplTokenType::OpenFunctionBody) {
            return Err(
                parser_err(
                    ErrorCode::UnexpectedToken,
                    format!(
                        "expected <body> in function '{}' but found '{:?}'",
                        name,
                        self.current_token()
                    ),
                )
                .with_hint(format!(
                    "add a <div class=\"body\"> block inside function '{}'",
                    name
                )),
            );
        }
        self.advance();

        self.push_scope();

       // Track the return type so parse_return can validate it
        let prev_fn_return_type = self.current_fn_return_type.replace(return_type.clone());

        // Pre-declare params inside the function scope
        for (param_name, param_type) in &params {
            self.declare_var(param_name.clone(), param_type.clone())?;
        }

        let mut body = Vec::new();
        while !self.is_at_end() && !matches!(self.current_token(), HaplTokenType::CloseFunctionBody) {
            body.push(self.parse_statement()?);
        }

        // Restore previous return type context before any early returns
        self.current_fn_return_type = prev_fn_return_type;

        if self.is_at_end() {
            self.pop_scope();
            return Err(
                parser_err(
                    ErrorCode::TagNotClosed,
                    format!("body of function '{}' was never closed", name),
                )
                .with_hint("add </div> to close the <div class=\"body\"> block"),
            );
        }

        self.pop_scope();
        self.advance(); // consume CloseFunctionBody

        if !matches!(
            self.current_token(),
            HaplTokenType::CloseFunction { name: ref n } if n == &name
        ) {
            return Err(
                parser_err(
                    ErrorCode::TagNotClosed,
                    format!("function '{}' was never closed", name),
                )
                .with_hint(format!(
                    "add a matching closing tag for function '{}'",
                    name
                )),
            );
        }

        self.advance(); // consume CloseFunction

        Ok(Expr::FunctionDeclaration {
            name,
            return_type,
            params,
            body,
        })
    }

    // --------------------------------------------------
    // Parse a function call
    // --------------------------------------------------
    fn parse_function_call(&mut self) -> Result<Expr, HaplError> {
        let name = match self.current_token() {
            HaplTokenType::OpenFunctionCall { name } => name.clone(),
            other => {
                return Err(parser_err(
                    ErrorCode::UnexpectedToken,
                    format!("expected function call but found '{:?}'", other),
                ));
            }
        };

        self.advance(); // consume OpenFunctionCall

        let mut args = Vec::new();

        if matches!(self.current_token(), HaplTokenType::OpenArgs) {
            self.advance(); // consume OpenArgs

            while !self.is_at_end() && !matches!(self.current_token(), HaplTokenType::CloseArgs) {
                args.push(self.parse_expression()?);
            }

            if self.is_at_end() {
                return Err(
                    parser_err(
                        ErrorCode::TagNotClosed,
                        format!("argument list for function call '{}' was never closed", name),
                    )
                    .with_hint("add a matching </div> to close the <div class=\"args\"> block"),
                );
            }

            self.advance(); // consume CloseArgs
        }

        if !matches!(
            self.current_token(),
            HaplTokenType::CloseFunctionCall { name: ref n } if n == &name
        ) {
            return Err(
                parser_err(
                    ErrorCode::TagNotClosed,
                    format!("function call '{}' was never closed", name),
                )
                .with_hint(format!(
                    "add a matching closing tag for the '{}' function call",
                    name
                )),
            );
        }

        self.advance(); // consume CloseFunctionCall

        // --------------------------------------------------
        // Static type-checking against the registered signature
        // --------------------------------------------------
        if let Some(sig) = self.function_signatures.get(&name).cloned() {
            // 1. Argument count
            if args.len() != sig.params.len() {
                return Err(
                    parser_err(
                        ErrorCode::WrongArgCount,
                        format!(
                            "function '{}' expects {} argument{}, but {} {} provided",
                            name,
                            sig.params.len(),
                            if sig.params.len() == 1 { "" } else { "s" },
                            args.len(),
                            if args.len() == 1 { "was" } else { "were" },
                        ),
                    )
                    .with_hint(format!(
                        "'{}' signature: ({})",
                        name,
                        sig.params
                            .iter()
                            .map(|(n, t)| format!("{}: {:?}", n, t))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )),
                );
            }

            // 2. Argument types
            for (i, (arg_expr, (param_name, param_type))) in
                args.iter().zip(sig.params.iter()).enumerate()
            {
                if let Some(arg_type) = self.infer_type(arg_expr) {
                    if arg_type != *param_type {
                        return Err(
                            parser_err(
                                ErrorCode::TypeMismatch,
                                format!(
                                    "argument {} ('{}') of call to '{}': expected {:?}, got {:?}",
                                    i + 1,
                                    param_name,
                                    name,
                                    param_type,
                                    arg_type,
                                ),
                            )
                            .with_hint(format!(
                                "parameter '{}' is declared as {:?} — pass a {:?} value",
                                param_name, param_type, param_type
                            )),
                        );
                    }
                }
                // If infer_type returns None we can't check statically — let runtime handle it
            }
        }
        // If the function hasn't been declared yet we also let the runtime surface the error,
        // since HAPL allows forward calls (call site before declaration in source order).

        Ok(Expr::FunctionCall { name, args })
    }

    // --------------------------------------------------
    // Parse a return statement
    // --------------------------------------------------
    fn parse_return(&mut self) -> Result<Expr, HaplError> {
        self.advance(); // consume OpenReturn

        let value = self.parse_expression()?;

        if self.is_at_end() || !matches!(self.current_token(), HaplTokenType::CloseReturn) {
            return Err(
                parser_err(
                    ErrorCode::TagNotClosed,
                    "return statement was never closed",
                )
                .with_hint("add a matching closing tag for the <div class=\"return\"> block"),
            );
        }

        self.advance(); // consume CloseReturn

        if let Some(expected) = &self.current_fn_return_type {
            // void functions must not return a value
            if *expected == StaticType::Void {
                return Err(
                    parser_err(
                        ErrorCode::TypeMismatch,
                        "void function cannot return a value",
                    )
                    .with_hint(
                        "declare the function with a non-void return type,                          or remove the return statement",
                    ),
                );
            }

            // non-void functions: check inferred type of the return expression
            if let Some(actual) = self.infer_type(&value) {
                if actual != *expected {
                    return Err(
                        parser_err(
                            ErrorCode::TypeMismatch,
                            format!(
                                "return type mismatch: function declared as {:?} but returns {:?}",
                                expected, actual
                            ),
                        )
                        .with_hint(format!(
                            "change the return value to a {:?} expression,                              or update the function's return type",
                            expected
                        )),
                    );
                }
            }
            // infer_type returning None means we can't check statically — runtime handles it
        }

        Ok(Expr::Return {
            value: Some(Box::new(value)),
        })
    }

    // --------------------------------------------------
    // Helpers
    // --------------------------------------------------

    fn current_token(&self) -> HaplTokenType {
        self.tokens[self.position].token_type.clone()
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn check_close_operator(&self, expected: LexerTagType) -> bool {
        if self.is_at_end() {
            return false;
        }
        match self.current_token() {
            HaplTokenType::CloseOperator { name } => name == expected,
            _ => false,
        }
    }

    fn check_close_var_dec(&self, var_type: &StaticType, name: &str) -> bool {
        if self.is_at_end() {
            return false;
        }
        match self.current_token() {
            HaplTokenType::CloseVarDec { var_type: t, name: n } => t == *var_type && n == name,
            _ => false,
        }
    }

    fn check_close_var_ref(&self, name: &str) -> bool {
        if self.is_at_end() {
            return false;
        }
        match self.current_token() {
            HaplTokenType::CloseVarRef { name: n } => n == name,
            _ => false,
        }
    }

    fn map_operator(&self, op: LexerTagType) -> Operator {
        match op {
            LexerTagType::Add          => Operator::Add,
            LexerTagType::Subtract     => Operator::Subtract,
            LexerTagType::Multiply     => Operator::Multiply,
            LexerTagType::Divide       => Operator::Divide,
            LexerTagType::And          => Operator::And,
            LexerTagType::Or           => Operator::Or,
            LexerTagType::Not          => Operator::Not,
            LexerTagType::Equal        => Operator::Equal,
            LexerTagType::NotEqual     => Operator::NotEqual,
            LexerTagType::Less         => Operator::Less,
            LexerTagType::LessEqual    => Operator::LessEqual,
            LexerTagType::Greater      => Operator::Greater,
            LexerTagType::GreaterEqual => Operator::GreaterEqual,
        }
    }

    fn operator_name_str(&self, op: LexerTagType) -> &'static str {
        match op {
            LexerTagType::Add          => "+",
            LexerTagType::Subtract     => "-",
            LexerTagType::Multiply     => "*",
            LexerTagType::Divide       => "/",
            LexerTagType::And          => "&&",
            LexerTagType::Or           => "||",
            LexerTagType::Not          => "!",
            LexerTagType::Equal        => "equal",
            LexerTagType::NotEqual     => "not_equal",
            LexerTagType::Less         => "less",
            LexerTagType::LessEqual    => "less_equal",
            LexerTagType::Greater      => "greater",
            LexerTagType::GreaterEqual => "greater_equal",
        }
    }

    fn infer_type(&self, expr: &Expr) -> Option<StaticType> {
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

            Expr::Operation { op, operands } => {
                match op {
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
                    | Operator::Divide => {
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
                }
            }


            Expr::Literal(LiteralValue::List { elem_type, .. }) => {
                Some(StaticType::List(Box::new(elem_type.clone())))
            }

            Expr::Literal(LiteralValue::Map { .. }) => Some(StaticType::Map),
            Expr::MapGet { .. }      => None,
            Expr::MapContains { .. } => Some(StaticType::Boolean),

            _ => None,
        }
    }
}