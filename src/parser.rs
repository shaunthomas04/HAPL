use crate::lexer::{HaplToken, HaplTokenType, LexerTagType};
use crate::ast::{Expr, Operator, StaticType, LiteralValue, ConditionalBlock};
use std::collections::HashMap;

pub struct HaplParser {
    tokens: Vec<HaplToken>,
    position: usize,
    symbol_table: HashMap<String, StaticType>, // tracks declared variables
}

impl HaplParser {
    pub fn new(tokens: Vec<HaplToken>) -> Self {
        Self {
            tokens,
            position: 0,
            symbol_table: HashMap::new(),
        }
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

                if self.symbol_table.contains_key(&var_name) {
                    panic!("Variable '{}' already declared", var_name);
                }

                // Insert into symbol table
                self.symbol_table.insert(var_name.clone(), var_type_copy);

                // Type-check **only literals** at parse time
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
                        // Allow non-literal expressions (arithmetic or variables) to be declared
                        // Type will be checked at runtime during evaluation
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

                if !self.symbol_table.contains_key(&var_name) {
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

                // Variable must already exist
                let expected_type = match self.symbol_table.get(&var_name) {
                    Some(t) => *t,
                    None => panic!("Variable '{}' assigned before declaration", var_name),
                };

                let value_expr = Box::new(self.parse_expression());

                // If literal, check immediately
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

                let value_expr = Box::new(self.parse_expression());

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

                // Parse the expression inside the print
                let inner_expr = self.parse_expression();

                // Ensure the print is properly closed
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

            // Everything else: parse as expression
            HaplTokenType::OpenVarDec { .. } 
            | HaplTokenType::OpenOperator { .. }
            | HaplTokenType::Literal(_)
            | HaplTokenType::OpenVarRef { .. }
            | HaplTokenType::OpenVarAssign { .. } => self.parse_expression(),

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

    fn parse_conditional_block(&mut self, open: HaplTokenType, close: HaplTokenType) -> ConditionalBlock {
        self.advance(); // consume OpenIf or OpenElif

        let mut statements = Vec::new();
        while !self.is_at_end() {
            match self.current_token() {
                // Check if we reached the closing token
                HaplTokenType::CloseIf if matches!(close, HaplTokenType::CloseIf) => break,
                HaplTokenType::CloseElif if matches!(close, HaplTokenType::CloseElif) => break,
                _ => statements.push(self.parse_statement()),
            }
        }

        self.advance(); // consume CloseIf or CloseElif

        // First statement inside the block is the condition
        let condition = statements.remove(0);

        ConditionalBlock { condition, statements }
    }

    fn parse_else_block(&mut self) -> Vec<Expr> {
        self.advance(); // consume OpenElse
        let mut statements = Vec::new();

        while !matches!(self.current_token(), HaplTokenType::CloseElse) {
            statements.push(self.parse_statement());
        }

        self.advance(); // consume CloseElse
        statements
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
            HaplTokenType::CloseVarDec { var_type: t, name: n } => {
                t == var_type && n == name
            }
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
            LexerTagType::Add => Operator::Add,
            LexerTagType::Subtract => Operator::Subtract,
            LexerTagType::Multiply => Operator::Multiply,
            LexerTagType::Divide => Operator::Divide,
            LexerTagType::And => Operator::And,
            LexerTagType::Or => Operator::Or,
            LexerTagType::Not => Operator::Not,
            LexerTagType::Equal => Operator::Equal,
            LexerTagType::NotEqual => Operator::NotEqual,
            LexerTagType::Less => Operator::Less,
            LexerTagType::LessEqual => Operator::LessEqual,
            LexerTagType::Greater => Operator::Greater,
            LexerTagType::GreaterEqual => Operator::GreaterEqual,
        }
    }
}
