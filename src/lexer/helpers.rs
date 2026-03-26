use crate::{HtmlTag};
use crate::ast::StaticType;
use super::tag_type::LexerTagType;

/// Render an HtmlTag as a short HTML snippet for error context.
/// e.g. `<var class="foo" id="bar">`
pub fn tag_snippet(tag: &HtmlTag) -> String {
    let mut s = format!("<{}", tag.tag_type);
    if let Some(c) = &tag.class { s.push_str(&format!(" class=\"{}\"", c)); }
    if let Some(id) = &tag.id   { s.push_str(&format!(" id=\"{}\"", id));   }
    s.push('>');
    s
}

/// Map a type keyword string to a `StaticType`. Returns `None` for unknown types.
pub fn parse_static_type(class: &str) -> Option<StaticType> {
    match class {
        "integer" => Some(StaticType::Integer),
        "double"  => Some(StaticType::Double),
        "string"  => Some(StaticType::String),
        "boolean" => Some(StaticType::Boolean),
        "void"    => Some(StaticType::Void),
        "map"     => Some(StaticType::Map),
        _         => None,
    }
}

/// Map a `LexerTagType` operator to its source-level string representation.
pub fn operator_to_str(op: LexerTagType) -> &'static str {
    match op {
        LexerTagType::Add          => "+",
        LexerTagType::Subtract     => "-",
        LexerTagType::Multiply     => "*",
        LexerTagType::Divide       => "/",
        LexerTagType::Modulo       => "%",
        LexerTagType::And          => "&&",
        LexerTagType::Or           => "||",
        LexerTagType::Not          => "!",
        LexerTagType::Equal        => "==",
        LexerTagType::NotEqual     => "!=",
        LexerTagType::Less         => "<",
        LexerTagType::LessEqual    => "<=",
        LexerTagType::Greater      => ">",
        LexerTagType::GreaterEqual => ">=",
    }
}
