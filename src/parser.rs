use crate::lexer::{HaplToken, HaplTokenType, LexerTagType};
use crate::ast::{Expr, Operator, StaticType, LiteralValue};
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
                            panic!("'!' operator requires exactly 1 operand");
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

            // Everything else: parse as expression
            HaplTokenType::OpenVarDec { .. } 
            | HaplTokenType::OpenOperator { .. }
            | HaplTokenType::Literal(_)
            | HaplTokenType::OpenVarRef { .. } => self.parse_expression(),

            _ => panic!(
                "Unexpected token {:?} at top-level position {}",
                self.current_token(),
                self.position
            ),
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

        }
    }
}
