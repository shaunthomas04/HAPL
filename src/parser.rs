use crate::lexer::{HaplToken, HaplTokenType, LexerTagType};
use crate::ast::{Expr, Operator, StaticType};
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

    pub fn parse(&mut self) -> Expr {
        self.parse_expression()
    }

    fn parse_expression(&mut self) -> Expr {
        match self.current_token() {
            HaplTokenType::Literal(lit) => {
                self.advance();
                Expr::Literal(lit)
            }

            HaplTokenType::OpenOperator { name } => {
                let operator = self.map_operator(name);
                self.advance(); // consume OpenOperator

                let mut operands = Vec::new();

                while !self.check_close_operator(name) {
                    operands.push(self.parse_expression());
                }

                self.advance(); // consume CloseOperator

                if operands.len() < 2 {
                    panic!("Operator requires at least 2 operands");
                }

                Expr::Operation { op: operator, operands }
            }

            // -------------------------
            // Variable Declaration
            // -------------------------
            HaplTokenType::OpenVarDec { var_type, name } => {
                let var_name = name.clone();
                let var_type_copy = var_type; // Copy because StaticType is Copy
                self.advance(); // consume OpenVarDec

                // Parse the value inside <var>
                let value_expr = Box::new(self.parse_expression());

                if !self.check_close_var_dec(var_type_copy, &var_name) {
                    panic!("Variable '{}' declaration not properly closed", var_name);
                }
                self.advance(); // consume CloseVarDec

                // Register in symbol table
                if self.symbol_table.contains_key(&var_name) {
                    panic!("Variable '{}' already declared", var_name);
                }
                self.symbol_table.insert(var_name.clone(), var_type_copy);

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

                if !self.check_close_var_ref(&var_name) {
                    panic!("Variable '{}' reference not properly closed", var_name);
                }
                self.advance(); // consume CloseVarRef

                Expr::VariableReference { name: var_name }
            }

            _ => panic!("Unexpected token in parser at position {}", self.position),
        }
    }

    fn current_token(&self) -> HaplTokenType {
        self.tokens[self.position].token_type.clone()
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn check_close_operator(&self, expected: LexerTagType) -> bool {
        match self.current_token() {
            HaplTokenType::CloseOperator { name } => name == expected,
            _ => false,
        }
    }

    fn check_close_var_dec(&self, var_type: StaticType, name: &str) -> bool {
        match self.current_token() {
            HaplTokenType::CloseVarDec { var_type: t, name: n } => t == var_type && n == name,
            _ => false,
        }
    }

    fn check_close_var_ref(&self, name: &str) -> bool {
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
        }
    }
}
