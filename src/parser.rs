use crate::lexer::{HaplToken, HaplTokenType, LexerTagType, LoopType};
use crate::ast::{Expr, Operator, StaticType, LiteralValue, ConditionalBlock};
use std::collections::HashMap;

pub struct HaplParser {
    tokens: Vec<HaplToken>,
    position: usize,
    // Scope stack mirrors the interpreter's scopes so that:
    // - nested scopes (functions, loops, conditionals) don't pollute outer scopes
    // - shadowing is allowed inside inner scopes
    scope_stack: Vec<HashMap<String, StaticType>>,
}

impl HaplParser {
    pub fn new(tokens: Vec<HaplToken>) -> Self {
        Self {
            tokens,
            position: 0,
            scope_stack: vec![HashMap::new()], // global scope
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

    fn declare_var(&mut self, name: String, var_type: StaticType) {
        // Only check the *current* scope for duplicates; shadowing outer scopes is fine.
        let current = self.scope_stack.last_mut().unwrap();
        if current.contains_key(&name) {
            panic!("Variable '{}' already declared in this scope", name);
        }
        current.insert(name, var_type);
    }

    fn lookup_var(&self, name: &str) -> Option<StaticType> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(*t);
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
    pub fn parse_expression(&mut self) -> Expr {
        if self.is_at_end() {
            panic!("Unexpected end of token stream while parsing expression");
        }

        match self.current_token() {
            HaplTokenType::Literal(lit) => {
                self.advance();
                Expr::Literal(lit)
            }

            // -------------------------
            // Operator
            // -------------------------
            HaplTokenType::OpenOperator { name } => {
                let operator = self.map_operator(name);
                self.advance(); // consume OpenOperator

                let mut operands = Vec::new();

                while !self.is_at_end() && !self.check_close_operator(name) {
                    operands.push(self.parse_expression());
                }

                if self.is_at_end() {
                    panic!("Operator not properly closed");
                }

                self.advance(); // consume CloseOperator

                match operator {
                    Operator::Not => {
                        if operands.len() != 1 {
                            panic!("'not' operator requires exactly 1 operand");
                        }
                    }

                    Operator::Equal
                    | Operator::NotEqual
                    | Operator::Less
                    | Operator::LessEqual
                    | Operator::Greater
                    | Operator::GreaterEqual => {
                        if operands.len() != 2 {
                            panic!("Comparison operators require exactly 2 operands");
                        }
                    }

                    _ => {
                        if operands.len() < 2 {
                            panic!("Operator requires at least 2 operands");
                        }
                    }
                }

                Expr::Operation { op: operator, operands }
            }

            // -------------------------
            // Variable Declaration
            // -------------------------
            HaplTokenType::OpenVarDec { var_type, name } => {
                let var_name = name.clone();
                let var_type_copy = var_type;

                self.advance(); // consume OpenVarDec

                let value_expr = Box::new(self.parse_expression());

                if self.is_at_end() || !self.check_close_var_dec(var_type_copy, &var_name) {
                    panic!("Variable '{}' declaration not properly closed", var_name);
                }

                self.advance(); // consume CloseVarDec

                // Register in current scope (allows shadowing in inner scopes)
                self.declare_var(var_name.clone(), var_type_copy);

                // Type-check literals at parse time
                match &*value_expr {
                    Expr::Literal(lit_val) => {
                        match (var_type_copy, lit_val) {
                            (StaticType::Integer, LiteralValue::Integer(_))
                            | (StaticType::Double, LiteralValue::Double(_))
                            | (StaticType::String, LiteralValue::String(_))
                            | (StaticType::Boolean, LiteralValue::Boolean(_)) => {}
                            _ => panic!(
                                "Type mismatch in variable '{}' declaration: expected {:?}, got {:?}",
                                var_name, var_type_copy, lit_val
                            ),
                        }
                    }
                    _ => {
                        // Non-literal expressions are type-checked at runtime
                    }
                }

                Expr::VariableDeclaration {
                    name: var_name,
                    var_type: var_type_copy,
                    value: value_expr,
                }
            }

            // -------------------------
            // Variable Reference
            // -------------------------
            HaplTokenType::OpenVarRef { name } => {
                let var_name = name.clone();

                self.advance(); // consume OpenVarRef

                if !self.var_exists(&var_name) {
                    panic!("Variable '{}' used before declaration", var_name);
                }

                if self.is_at_end() || !self.check_close_var_ref(&var_name) {
                    panic!("Variable '{}' reference not properly closed", var_name);
                }

                self.advance(); // consume CloseVarRef

                Expr::VariableReference { name: var_name }
            }

            // -------------------------
            // Variable Assignment
            // -------------------------
            HaplTokenType::OpenVarAssign { name } => {
                let var_name = name.clone();

                self.advance(); // consume OpenVarAssign

                // Variable must already exist (search all scopes)
                let expected_type = match self.lookup_var(&var_name) {
                    Some(t) => t,
                    None => panic!("Variable '{}' assigned before declaration", var_name),
                };

                let value_expr = Box::new(self.parse_expression());

                // If literal, type-check immediately
                if let Expr::Literal(lit_val) = &*value_expr {
                    match (expected_type, lit_val) {
                        (StaticType::Integer, LiteralValue::Integer(_))
                        | (StaticType::Double, LiteralValue::Double(_))
                        | (StaticType::String, LiteralValue::String(_))
                        | (StaticType::Boolean, LiteralValue::Boolean(_)) => {}
                        _ => panic!(
                            "Type mismatch in assignment to '{}': expected {:?}, got {:?}",
                            var_name, expected_type, lit_val
                        ),
                    }
                }

                if self.is_at_end() || !matches!(
                    self.current_token(),
                    HaplTokenType::CloseVarAssign { name: ref n } if n == &var_name
                ) {
                    panic!("Variable '{}' assignment not properly closed", var_name);
                }

                self.advance(); // consume CloseVarAssign

                Expr::Assignment {
                    name: var_name,
                    value: value_expr,
                }
            }

            // -------------------------
            // Function Call
            // -------------------------
            HaplTokenType::OpenFunctionCall { .. } => self.parse_function_call(),

            _ => panic!(
                "Unexpected token {:?} at position {}",
                self.current_token(),
                self.position
            ),
        }
    }

    // --------------------------------------------------
    // Parse a single top-level statement
    // --------------------------------------------------
    pub fn parse_statement(&mut self) -> Expr {
        match self.current_token() {
            // -------------------------
            // Print statement
            // -------------------------
            HaplTokenType::OpenPrint { .. } => {
                self.advance(); // consume <p>

                let inner_expr = self.parse_expression();

                if self.is_at_end() || !matches!(self.current_token(), HaplTokenType::ClosePrint { .. }) {
                    panic!("Print statement not properly closed with </p>");
                }

                self.advance(); // consume </p>
                Expr::Print { value: Box::new(inner_expr) }
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

            // Structural HTML wrapper tokens (html, head, body, title, meta, link)
            // are emitted by the lexer so HAPL can live inside real HTML files.
            // The parser simply skips them and continues parsing.
            HaplTokenType::OpenHtmlTag { .. } | HaplTokenType::CloseHtmlTag { .. } => {
                self.advance();
                // Return a no-op so parse_program can continue; use a dummy literal
                Expr::Literal(LiteralValue::Boolean(false))
            }

            _ => panic!(
                "Unexpected token {:?} at top-level position {}",
                self.current_token(),
                self.position
            ),
        }
    }

    // --------------------------------------------------
    // Parse a conditional statement
    // --------------------------------------------------
    fn parse_conditional(&mut self) -> Expr {
        if !matches!(self.current_token(), HaplTokenType::OpenConditional) {
            panic!("Expected <conditional> but found {:?}", self.current_token());
        }

        self.advance(); // consume OpenConditional

        let mut if_blocks = Vec::new();
        let mut else_block: Option<Vec<Expr>> = None;

        while !matches!(self.current_token(), HaplTokenType::CloseConditional) {
            match self.current_token() {
                HaplTokenType::OpenIf => {
                    if_blocks.push(self.parse_conditional_block(HaplTokenType::OpenIf, HaplTokenType::CloseIf));
                }
                HaplTokenType::OpenElif => {
                    if_blocks.push(self.parse_conditional_block(HaplTokenType::OpenElif, HaplTokenType::CloseElif));
                }
                HaplTokenType::OpenElse => {
                    else_block = Some(self.parse_else_block());
                }
                _ => panic!("Unexpected token inside conditional: {:?}", self.current_token()),
            }
        }

        self.advance(); // consume CloseConditional

        Expr::Conditional { if_blocks, else_block }
    }

    // FIX: Parse condition explicitly first, then parse body statements separately.
    // Previously the condition was assumed to be `statements.remove(0)` which
    // would panic on an empty block and was semantically fragile.
    fn parse_conditional_block(&mut self, _open: HaplTokenType, close: HaplTokenType) -> ConditionalBlock {
        self.advance(); // consume OpenIf or OpenElif

        self.push_scope();

        // The first expression inside the block is always the condition
        let condition = self.parse_expression();

        // Remaining statements form the body
        let mut statements = Vec::new();
        while !self.is_at_end() {
            let at_close = match &close {
                HaplTokenType::CloseIf   => matches!(self.current_token(), HaplTokenType::CloseIf),
                HaplTokenType::CloseElif => matches!(self.current_token(), HaplTokenType::CloseElif),
                _ => false,
            };
            if at_close { break; }
            statements.push(self.parse_statement());
        }

        self.pop_scope();
        self.advance(); // consume CloseIf or CloseElif

        ConditionalBlock { condition, statements }
    }

    fn parse_else_block(&mut self) -> Vec<Expr> {
        self.advance(); // consume OpenElse
        self.push_scope();

        let mut statements = Vec::new();
        while !matches!(self.current_token(), HaplTokenType::CloseElse) {
            statements.push(self.parse_statement());
        }

        self.pop_scope();
        self.advance(); // consume CloseElse
        statements
    }

    // --------------------------------------------------
    // Parse while loop
    // --------------------------------------------------
    fn parse_while(&mut self) -> Expr {
        if let HaplTokenType::OpenLoop { loop_type } = self.current_token() {
            if loop_type != LoopType::While {
                panic!("Expected a while loop but found {:?}", self.current_token());
            }
        } else {
            panic!("Expected OpenLoop but found {:?}", self.current_token());
        }

        self.advance(); // consume OpenLoop

        // Parse condition
        if !matches!(self.current_token(), HaplTokenType::OpenLoopCondition) {
            panic!("Expected <loop_condition> but found {:?}", self.current_token());
        }
        self.advance(); // consume OpenLoopCondition

        let condition_expr = self.parse_expression();

        if !matches!(self.current_token(), HaplTokenType::CloseLoopCondition) {
            panic!("Loop condition not properly closed");
        }
        self.advance(); // consume CloseLoopCondition

        // Parse body (own scope)
        if !matches!(self.current_token(), HaplTokenType::OpenLoopBody) {
            panic!("Expected <loop_body> but found {:?}", self.current_token());
        }
        self.advance(); // consume OpenLoopBody

        self.push_scope();
        let mut body_statements = Vec::new();
        while !matches!(self.current_token(), HaplTokenType::CloseLoopBody) {
            body_statements.push(self.parse_statement());
        }
        self.pop_scope();

        self.advance(); // consume CloseLoopBody

        // Close loop
        if let HaplTokenType::CloseLoop { loop_type } = self.current_token() {
            if loop_type != LoopType::While {
                panic!("Expected CloseLoop(While) but found {:?}", self.current_token());
            }
        } else {
            panic!("Expected CloseLoop but found {:?}", self.current_token());
        }
        self.advance(); // consume CloseLoop

        Expr::WhileLoop {
            condition: Box::new(condition_expr),
            body: body_statements,
        }
    }

    // --------------------------------------------------
    // Parse for loop
    // --------------------------------------------------
    fn parse_for(&mut self) -> Expr {
        if let HaplTokenType::OpenLoop { loop_type } = self.current_token() {
            if loop_type != LoopType::For {
                panic!("Expected a for loop but found {:?}", self.current_token());
            }
        } else {
            panic!("Expected OpenLoop(For) but found {:?}", self.current_token());
        }

        self.advance(); // consume OpenLoop

        // FIX: Push a scope for the for loop so the iterator variable is scoped
        // to the loop and doesn't leak into the surrounding parser scope.
        self.push_scope();

        // Parse Iterator
        if !matches!(self.current_token(), HaplTokenType::OpenLoopIterator) {
            panic!("Expected <iterator> but found {:?}", self.current_token());
        }
        self.advance(); // consume OpenLoopIterator

        let iterator_expr = self.parse_expression();

        let iterator_name = match &iterator_expr {
            Expr::VariableReference { name } => name.clone(),
            _ => panic!("For loop iterator must be a variable reference"),
        };

        // Auto-declare iterator in the loop's own scope if not already there
        if self.scope_stack.last().map_or(true, |s| !s.contains_key(&iterator_name)) {
            self.declare_var(iterator_name.clone(), StaticType::Integer);
        }

        // Enforce iterator is Integer
        match self.lookup_var(&iterator_name) {
            Some(t) if t != StaticType::Integer => {
                panic!("For loop iterator '{}' must be Integer", iterator_name);
            }
            _ => {}
        }

        if !matches!(self.current_token(), HaplTokenType::CloseLoopIterator) {
            panic!("Expected </iterator> but found {:?}", self.current_token());
        }
        self.advance(); // consume CloseLoopIterator

        // Parse Condition
        if !matches!(self.current_token(), HaplTokenType::OpenLoopCondition) {
            panic!("Expected <condition> but found {:?}", self.current_token());
        }
        self.advance(); // consume OpenLoopCondition

        let condition_expr = self.parse_expression();

        // FIX: infer_type is now type-aware for arithmetic so this check is more accurate
        if let Some(cond_type) = self.infer_type(&condition_expr) {
            if cond_type != StaticType::Boolean {
                panic!("For loop condition must evaluate to Boolean");
            }
        }

        if !matches!(self.current_token(), HaplTokenType::CloseLoopCondition) {
            panic!("Expected </condition> but found {:?}", self.current_token());
        }
        self.advance(); // consume CloseLoopCondition

        // Parse Increment
        if !matches!(self.current_token(), HaplTokenType::OpenLoopIncrement) {
            panic!("Expected <increment> but found {:?}", self.current_token());
        }
        self.advance(); // consume OpenLoopIncrement

        let increment_expr = self.parse_expression();

        if let Some(inc_type) = self.infer_type(&increment_expr) {
            if inc_type != StaticType::Integer {
                panic!("For loop increment must evaluate to Integer");
            }
        }

        if !matches!(self.current_token(), HaplTokenType::CloseLoopIncrement) {
            panic!("Expected </increment> but found {:?}", self.current_token());
        }
        self.advance(); // consume CloseLoopIncrement

        // Parse Body
        if !matches!(self.current_token(), HaplTokenType::OpenLoopBody) {
            panic!("Expected <body> but found {:?}", self.current_token());
        }
        self.advance(); // consume OpenLoopBody

        let mut body_statements = Vec::new();
        while !matches!(self.current_token(), HaplTokenType::CloseLoopBody) {
            body_statements.push(self.parse_statement());
        }

        self.advance(); // consume CloseLoopBody

        // Pop the for loop's scope (iterator no longer visible)
        self.pop_scope();

        // Close Loop
        if let HaplTokenType::CloseLoop { loop_type } = self.current_token() {
            if loop_type != LoopType::For {
                panic!("Expected CloseLoop(For) but found {:?}", self.current_token());
            }
        } else {
            panic!("Expected CloseLoop but found {:?}", self.current_token());
        }
        self.advance(); // consume CloseLoop

        Expr::ForLoop {
            iterator: iterator_name,
            condition: Box::new(condition_expr),
            increment: Box::new(increment_expr),
            body: body_statements,
        }
    }

    // --------------------------------------------------
    // Parse an entire program (ALL top-level statements)
    // --------------------------------------------------
    pub fn parse_program(&mut self) -> Vec<Expr> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.parse_statement());
        }

        statements
    }

    // --------------------------------------------------
    // Parse a function declaration
    // --------------------------------------------------
    fn parse_function(&mut self) -> Expr {
        let (name, return_type) = match self.current_token() {
            HaplTokenType::OpenFunction { name, return_type } => {
                (name.clone(), return_type)
            }
            _ => panic!("Expected OpenFunction"),
        };

        self.advance(); // consume OpenFunction

        // Parse Params
        if !matches!(self.current_token(), HaplTokenType::OpenParams) {
            panic!("Expected OpenParams");
        }
        self.advance();

        let mut params = Vec::new();

        while !matches!(self.current_token(), HaplTokenType::CloseParams) {
            match self.current_token() {
                HaplTokenType::OpenParam { name, param_type } => {
                    let param_name = name.clone();
                    let param_type_copy = param_type;

                    self.advance(); // consume OpenParam

                    if !matches!(
                        self.current_token(),
                        HaplTokenType::CloseParam { name: ref n } if n == &param_name
                    ) {
                        panic!("Parameter '{}' not properly closed", param_name);
                    }

                    self.advance(); // consume CloseParam

                    params.push((param_name.clone(), param_type_copy));
                }
                _ => panic!("Unexpected token in params"),
            }
        }

        self.advance(); // consume CloseParams

        // Parse Body — push an isolated scope for the function
        if !matches!(self.current_token(), HaplTokenType::OpenFunctionBody) {
            panic!("Expected OpenFunctionBody");
        }
        self.advance();

        // FIX: Use an isolated scope for the function body so that variables
        // declared inside the function don't pollute the global parser scope.
        self.push_scope();

        // Pre-declare params inside the function scope
        for (param_name, param_type) in &params {
            self.declare_var(param_name.clone(), *param_type);
        }

        let mut body = Vec::new();
        while !matches!(self.current_token(), HaplTokenType::CloseFunctionBody) {
            body.push(self.parse_statement());
        }

        self.pop_scope();
        self.advance(); // consume CloseFunctionBody

        // Close Function
        if !matches!(
            self.current_token(),
            HaplTokenType::CloseFunction { name: ref n } if n == &name
        ) {
            panic!("Function '{}' not properly closed", name);
        }

        self.advance(); // consume CloseFunction

        Expr::FunctionDeclaration {
            name,
            return_type,
            params,
            body,
        }
    }

    // --------------------------------------------------
    // Parse a function call
    // --------------------------------------------------
    fn parse_function_call(&mut self) -> Expr {
        let name = match self.current_token() {
            HaplTokenType::OpenFunctionCall { name } => name.clone(),
            _ => panic!("Expected OpenFunctionCall"),
        };

        self.advance(); // consume OpenFunctionCall

        let mut args = Vec::new();

        if matches!(self.current_token(), HaplTokenType::OpenArgs) {
            self.advance(); // consume OpenArgs

            while !matches!(self.current_token(), HaplTokenType::CloseArgs) {
                args.push(self.parse_expression());
            }

            self.advance(); // consume CloseArgs
        }

        if !matches!(
            self.current_token(),
            HaplTokenType::CloseFunctionCall { name: ref n } if n == &name
        ) {
            panic!("Function call '{}' not properly closed", name);
        }

        self.advance(); // consume CloseFunctionCall

        Expr::FunctionCall { name, args }
    }

    // --------------------------------------------------
    // Parse a return statement
    // --------------------------------------------------
    fn parse_return(&mut self) -> Expr {
        self.advance(); // consume OpenReturn

        let value = self.parse_expression();

        if !matches!(self.current_token(), HaplTokenType::CloseReturn) {
            panic!("Return not properly closed");
        }

        self.advance(); // consume CloseReturn

        Expr::Return {
            value: Some(Box::new(value)),
        }
    }

    // --------------------------------------------------
    // Helpers
    // --------------------------------------------------

    fn current_token(&self) -> HaplTokenType {
        if self.is_at_end() {
            panic!("Tried to access token beyond end of stream");
        }
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

    fn check_close_var_dec(&self, var_type: StaticType, name: &str) -> bool {
        if self.is_at_end() {
            return false;
        }
        match self.current_token() {
            HaplTokenType::CloseVarDec { var_type: t, name: n } => t == var_type && n == name,
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

    // FIX: infer_type now inspects operands to determine the actual output type
    // of arithmetic operations instead of blindly returning Integer.
    fn infer_type(&self, expr: &Expr) -> Option<StaticType> {
        match expr {
            Expr::Literal(LiteralValue::Integer(_)) => Some(StaticType::Integer),
            Expr::Literal(LiteralValue::Double(_))  => Some(StaticType::Double),
            Expr::Literal(LiteralValue::String(_))  => Some(StaticType::String),
            Expr::Literal(LiteralValue::Boolean(_)) => Some(StaticType::Boolean),

            Expr::VariableReference { name } => self.lookup_var(name),

            Expr::Operation { op, operands } => {
                match op {
                    // These always produce Boolean
                    Operator::Equal
                    | Operator::NotEqual
                    | Operator::Less
                    | Operator::LessEqual
                    | Operator::Greater
                    | Operator::GreaterEqual
                    | Operator::And
                    | Operator::Or
                    | Operator::Not => Some(StaticType::Boolean),

                    // Arithmetic: infer from the first operand so Double + Double => Double
                    Operator::Add
                    | Operator::Subtract
                    | Operator::Multiply
                    | Operator::Divide => {
                        // Walk operands and promote: if any is Double, result is Double
                        let mut result_type = None;
                        for operand in operands {
                            match self.infer_type(operand) {
                                Some(StaticType::Double) => {
                                    result_type = Some(StaticType::Double);
                                    break; // Double wins, stop early
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

            _ => None,
        }
    }
}