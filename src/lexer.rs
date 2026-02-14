use crate::HtmlTag;
use crate::ast::{LiteralValue, StaticType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LexerTagType {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone)]
pub enum HaplTokenType {
    OpenOperator { name: LexerTagType },
    CloseOperator { name: LexerTagType },
    OpenHtmlTag { name: String },
    CloseHtmlTag { name: String },
    Literal(LiteralValue),
    OpenVarDec { var_type: StaticType, name: String },
    CloseVarDec { var_type: StaticType, name: String },
    OpenVarRef { name: String },
    CloseVarRef { name: String },
}

#[derive(Debug, Clone)]
pub struct HaplToken {
    pub token_type: HaplTokenType,
    pub value: Option<String>, // printable/debug value
}

impl HaplToken {
    fn new(token_type: HaplTokenType, value: Option<String>) -> Self {
        Self { token_type, value }
    }

    fn open_operator(name: LexerTagType) -> Self {
        Self::new(
            HaplTokenType::OpenOperator { name },
            Some(operator_to_str(name).to_string()),
        )
    }

    fn close_operator(name: LexerTagType) -> Self {
        Self::new(
            HaplTokenType::CloseOperator { name },
            Some(operator_to_str(name).to_string()),
        )
    }

    fn open_html_tag(name: String) -> Self {
        Self::new(
            HaplTokenType::OpenHtmlTag { name: name.clone() },
            Some(name),
        )
    }

    fn close_html_tag(name: String) -> Self {
        Self::new(
            HaplTokenType::CloseHtmlTag { name: name.clone() },
            Some(name),
        )
    }

    fn string(content: String) -> Self {
        Self::new(
            HaplTokenType::Literal(LiteralValue::String(content.clone())),
            Some(content),
        )
    }

    fn double(double_value: f64) -> Self {
        Self::new(
            HaplTokenType::Literal(LiteralValue::Double(double_value)),
            Some(double_value.to_string()),
        )
    }

    fn integer(integer_value: i64) -> Self {
        Self::new(
            HaplTokenType::Literal(LiteralValue::Integer(integer_value)),
            Some(integer_value.to_string()),
        )
    }
}

pub struct HaplLexer {
    tokens: Vec<HaplToken>,
}

impl HaplLexer {
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    pub fn lex(&mut self, tag: &HtmlTag) {
        self.walk(tag);
    }

    fn walk(&mut self, tag: &HtmlTag) {
        // -----------------------------------------
        // Arithmetic <div class="+">
        // -----------------------------------------
        if tag.tag_type == "div" {
            if let Some(class) = &tag.class {
                if ["+", "-", "*", "/"].contains(&class.as_str()) {
                    let (open_token, close_token) = self.get_arithmetic_tags(class);

                    self.tokens.push(open_token);

                    for child in &tag.child_tags {
                        self.walk(child);
                    }

                    self.tokens.push(close_token);
                    return;
                }
            }
        }

        // -----------------------------------------
        // Variable tags <var>
        // -----------------------------------------
        if tag.tag_type == "var" {
            if let Some(class) = &tag.class {
                if let Some(id) = &tag.id {
                    // Variable declaration: <var class="integer" id="x">10</var>
                    let var_type = match class.as_str() {
                        "integer" => StaticType::Integer,
                        "double" => StaticType::Double,
                        "string" => StaticType::String,
                        other => panic!("Unknown variable type '{}'", other),
                    };

                    self.tokens.push(HaplToken::new(
                        HaplTokenType::OpenVarDec {
                            var_type,
                            name: id.clone(),
                        },
                        Some(format!("{} {}", class, id)),
                    ));

                    for child in &tag.child_tags {
                        self.walk(child);
                    }

                    self.tokens.push(HaplToken::new(
                        HaplTokenType::CloseVarDec {
                            var_type,
                            name: id.clone(),
                        },
                        Some(format!("{} {}", class, id)),
                    ));
                    return;
                } else {
                    // Variable reference: <var class="x"></var>
                    self.tokens.push(HaplToken::new(
                        HaplTokenType::OpenVarRef { name: class.clone() },
                        Some(class.clone()),
                    ));

                    for child in &tag.child_tags {
                        self.walk(child);
                    }

                    self.tokens.push(HaplToken::new(
                        HaplTokenType::CloseVarRef { name: class.clone() },
                        Some(class.clone()),
                    ));
                    return;
                }
            } else {
                panic!("<var> tag requires either an id for declaration or class for reference");
            }
        }

        // -----------------------------------------
        // <span> → literals
        // -----------------------------------------
        if tag.tag_type == "span" {
            if tag.child_tags.is_empty() && !tag.content.trim().is_empty() {
                let token = self.parse_literal(&tag.content, tag.class.as_deref());
                self.tokens.push(token);
            } else {
                for child in &tag.child_tags {
                    self.walk(child);
                }
            }
            return;
        }

        // -----------------------------------------
        // Structural HTML tags
        // -----------------------------------------
        let structural = ["html", "head", "body", "title"];

        if structural.contains(&tag.tag_type.as_str()) {
            self.tokens.push(HaplToken::open_html_tag(tag.tag_type.clone()));

            for child in &tag.child_tags {
                self.walk(child);
            }

            self.tokens.push(HaplToken::close_html_tag(tag.tag_type.clone()));
            return;
        }

        // -----------------------------------------
        // Generic leaf literal
        // -----------------------------------------
        if tag.child_tags.is_empty() && !tag.content.trim().is_empty() {
            let token = self.parse_literal(&tag.content, tag.class.as_deref());
            self.tokens.push(token);
        } else {
            for child in &tag.child_tags {
                self.walk(child);
            }
        }
    }

    pub fn print(&self) {
        for token in &self.tokens {
            match &token.token_type {
                HaplTokenType::OpenOperator { name } => {
                    println!("OpenOperator({}) -> {:?}", operator_to_str(*name), token.value);
                }
                HaplTokenType::CloseOperator { name } => {
                    println!("CloseOperator({}) -> {:?}", operator_to_str(*name), token.value);
                }
                HaplTokenType::OpenHtmlTag { name } => {
                    println!("OpenHtmlTag({}) -> {:?}", name, token.value);
                }
                HaplTokenType::CloseHtmlTag { name } => {
                    println!("CloseHtmlTag({}) -> {:?}", name, token.value);
                }
                HaplTokenType::Literal(lit) => {
                    println!("Literal({:?}) -> {:?}", lit, token.value);
                }
                HaplTokenType::OpenVarDec { var_type, name } => {
                    println!("OpenVarDec({:?}) -> {}", var_type, name);
                }
                HaplTokenType::CloseVarDec { var_type, name } => {
                    println!("CloseVarDec({:?}) -> {}", var_type, name);
                }
                HaplTokenType::OpenVarRef { name } => {
                    println!("OpenVarRef({}) -> {:?}", name, token.value);
                }
                HaplTokenType::CloseVarRef { name } => {
                    println!("CloseVarRef({}) -> {:?}", name, token.value);
                }
            }
        }
    }

    pub fn tokens(&self) -> &[HaplToken] {
        &self.tokens
    }

    // -----------------------------------------
    // Helpers
    // -----------------------------------------

    fn get_arithmetic_tags(&self, operator: &str) -> (HaplToken, HaplToken) {
        let tag_type = match operator {
            "+" => LexerTagType::Add,
            "-" => LexerTagType::Subtract,
            "*" => LexerTagType::Multiply,
            "/" => LexerTagType::Divide,
            _ => panic!("Unknown operator: {}", operator),
        };

        (HaplToken::open_operator(tag_type), HaplToken::close_operator(tag_type))
    }

    fn parse_literal(&self, text: &str, class_type: Option<&str>) -> HaplToken {
        let trimmed = text.trim();

        let class = class_type.expect(
            "Static type required on <span>: expected 'integer', 'double', or 'string'",
        );

        match class {
            "integer" => trimmed
                .parse::<i64>()
                .map(HaplToken::integer)
                .unwrap_or_else(|_| panic!("Type error: value '{}' is not a valid integer", trimmed)),
            "double" => trimmed
                .parse::<f64>()
                .map(HaplToken::double)
                .unwrap_or_else(|_| panic!("Type error: value '{}' is not a valid double", trimmed)),
            "string" => HaplToken::string(trimmed.to_string()),
            other => panic!("Unknown static type '{}'. Expected 'integer', 'double', or 'string'", other),
        }
    }
}

fn operator_to_str(op: LexerTagType) -> &'static str {
    match op {
        LexerTagType::Add => "+",
        LexerTagType::Subtract => "-",
        LexerTagType::Multiply => "*",
        LexerTagType::Divide => "/",
    }
}
