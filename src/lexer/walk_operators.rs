use super::*;

impl HaplLexer {
    pub(super) fn try_arithmetic_tokens(&self, class: &str) -> Option<(HaplToken, HaplToken)> {
        let tag_type = match class {
            "+" => LexerTagType::Add,
            "-" => LexerTagType::Subtract,
            "*" => LexerTagType::Multiply,
            "/" => LexerTagType::Divide,
            "%" => LexerTagType::Modulo,
            _   => return None,
        };
        Some((HaplToken::open_operator(tag_type), HaplToken::close_operator(tag_type)))
    }

    pub(super) fn try_boolean_tokens(&self, class: &str) -> Option<(HaplToken, HaplToken)> {
        let tag_type = match class {
            "&&" => LexerTagType::And,
            "||" => LexerTagType::Or,
            "!"  => LexerTagType::Not,
            _    => return None,
        };
        Some((HaplToken::open_operator(tag_type), HaplToken::close_operator(tag_type)))
    }

    pub(super) fn try_comparison_tokens(&self, class: &str) -> Option<(HaplToken, HaplToken)> {
        let tag_type = match class {
            "equal"         => LexerTagType::Equal,
            "not_equal"     => LexerTagType::NotEqual,
            "less"          => LexerTagType::Less,
            "less_equal"    => LexerTagType::LessEqual,
            "greater"       => LexerTagType::Greater,
            "greater_equal" => LexerTagType::GreaterEqual,
            _               => return None,
        };
        Some((HaplToken::open_operator(tag_type), HaplToken::close_operator(tag_type)))
    }
}
