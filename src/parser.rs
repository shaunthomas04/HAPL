use crate::lexer::{HaplToken, HaplTokenType, LexerTagType};
use crate::ast::{Expr, Operator};

pub struct HaplParser {
    tokens: Vec<HaplToken>,
    position: usize,
}

impl HaplParser {
    pub fn new(tokens: Vec<HaplToken>) -> Self {
        Self { tokens, position: 0 }
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

            _ => panic!("Unexpected token in parser"),
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

    fn map_operator(&self, op: LexerTagType) -> Operator {
        match op {
            LexerTagType::Add => Operator::Add,
            LexerTagType::Subtract => Operator::Subtract,
            LexerTagType::Multiply => Operator::Multiply,
            LexerTagType::Divide => Operator::Divide,
        }
    }
}
