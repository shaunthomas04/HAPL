use crate::HtmlTag;
use crate::ast::{LiteralValue, StaticType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LexerTagType {
    Add,
    Subtract,
    Multiply,
    Divide,
    And,
    Or,
    Not
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
    OpenPrint,
    ClosePrint,
    OpenConditional,
    CloseConditional,
    OpenIf,
    CloseIf,
    OpenElif,
    CloseElif,
    OpenElse,
    CloseElse,
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
    
    fn boolean(boolean_value: bool) -> Self {
        Self::new(
            HaplTokenType::Literal(LiteralValue::Boolean(boolean_value)),
            Some(boolean_value.to_string()),
        )
    }

    fn open_print() -> Self {
        Self::new(HaplTokenType::OpenPrint, Some("print".to_string()))
    }

    fn close_print() -> Self {
        Self::new(HaplTokenType::ClosePrint, Some("print".to_string()))
    }

    pub fn open_conditional() -> Self {
        Self::new(HaplTokenType::OpenConditional, Some("conditional".to_string()))
    }
    pub fn close_conditional() -> Self {
        Self::new(HaplTokenType::CloseConditional, Some("conditional".to_string()))
    }
    pub fn open_if() -> Self {
        Self::new(HaplTokenType::OpenIf, Some("if".to_string()))
    }
    pub fn close_if() -> Self {
        Self::new(HaplTokenType::CloseIf, Some("if".to_string()))
    }
    pub fn open_elif() -> Self {
        Self::new(HaplTokenType::OpenElif, Some("elif".to_string()))
    }
    pub fn close_elif() -> Self {
        Self::new(HaplTokenType::CloseElif, Some("elif".to_string()))
    }
    pub fn open_else() -> Self {
        Self::new(HaplTokenType::OpenElse, Some("else".to_string()))
    }
    pub fn close_else() -> Self {
        Self::new(HaplTokenType::CloseElse, Some("else".to_string()))
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
                if ["&&", "||", "!"].contains(&class.as_str()) {
                    let (open_token, close_token) = self.get_boolean_tags(class);

                    self.tokens.push(open_token);

                    for child in &tag.child_tags {
                        self.walk(child);
                    }

                    self.tokens.push(close_token);
                    return;
                }


                // -----------------------------------------
                // Conditionals <div class="conditional">
                // -----------------------------------------
                if class == "conditional" {
                    self.tokens.push(HaplToken::open_conditional());
                    for child in &tag.child_tags {
                        match child.tag_type.as_str() {
                            "div" => {
                                if let Some(child_class) = &child.class {
                                    match child_class.as_str() {
                                        // -----------------------------------------
                                        // If block <div class="if">
                                        // -----------------------------------------
                                        "if" => {
                                            self.tokens.push(HaplToken::open_if());
                                            for grandchild in &child.child_tags {
                                                self.walk(grandchild);
                                            }
                                            self.tokens.push(HaplToken::close_if());
                                        }
                                        // -----------------------------------------
                                        // Elif <div class="elif">
                                        // -----------------------------------------
                                        "elif" => {
                                            self.tokens.push(HaplToken::open_elif());
                                            for grandchild in &child.child_tags {
                                                self.walk(grandchild);
                                            }
                                            self.tokens.push(HaplToken::close_elif());
                                        }
                                        // -----------------------------------------
                                        // Else <div class="else">
                                        // -----------------------------------------
                                        "else" => {
                                            self.tokens.push(HaplToken::open_else());
                                            for grandchild in &child.child_tags {
                                                self.walk(grandchild);
                                            }
                                            self.tokens.push(HaplToken::close_else());
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    self.tokens.push(HaplToken::close_conditional());
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
                        "boolean" => StaticType::Boolean,
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

        // -----------------------------------------
        // Print <p> tag
        // -----------------------------------------
        if tag.tag_type == "p" {
            self.tokens.push(HaplToken::open_print());

            for child in &tag.child_tags {
                self.walk(child);
            }

            self.tokens.push(HaplToken::close_print());
            return;
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
                HaplTokenType::OpenPrint => {
                    println!("OpenPrint -> {:?}", token.value);
                }
                HaplTokenType::ClosePrint => {
                    println!("ClosePrint -> {:?}", token.value);
                }
                HaplTokenType::OpenConditional => {
                    println!("OpenConditional -> {:?}", token.value);
                }
                HaplTokenType::CloseConditional => {
                    println!("CloseConditional -> {:?}", token.value);
                }
                HaplTokenType::OpenIf => {
                    println!("OpenIf -> {:?}", token.value);
                }
                HaplTokenType::CloseIf => {
                    println!("CloseIf -> {:?}", token.value);
                }
                HaplTokenType::OpenElif => {
                    println!("OpenElif -> {:?}", token.value);
                }
                HaplTokenType::CloseElif => {
                    println!("CloseElif -> {:?}", token.value);
                }
                HaplTokenType::OpenElse => {
                    println!("OpenElse -> {:?}", token.value);
                }
                HaplTokenType::CloseElse => {
                    println!("CloseElse -> {:?}", token.value);
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

    fn get_boolean_tags(&self, logic_operator: &str) -> (HaplToken, HaplToken) {
        let tag_type = match logic_operator {
            "&&" => LexerTagType::And,
            "||" => LexerTagType::Or,
            "!" => LexerTagType::Not,
            _ => panic!("Unknown operator: {}", logic_operator),
        };

        (HaplToken::open_operator(tag_type), HaplToken::close_operator(tag_type))
    }

    fn parse_literal(&self, text: &str, class_type: Option<&str>) -> HaplToken {
        let trimmed = text.trim();

        let class = class_type.expect(
            "Static type required on <span>: expected 'integer', 'double', 'boolean' or 'string'",
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
            "string" => {
                if !trimmed.starts_with('"') || !trimmed.ends_with('"') {
                    panic!(
                        "String literals must be enclosed in double quotes (\") but got '{}'",
                        trimmed
                    );
                }
                // Strip the quotes
                let inner = &trimmed[1..trimmed.len() - 1];
                HaplToken::string(inner.to_string()) 
            }

            "boolean" => {
                match trimmed {
                    "true" => HaplToken::boolean(true),
                    "false" => HaplToken::boolean(false),
                    _ => panic!(
                        "Type error: value '{}' is not a valid boolean (expected true or false)",
                        trimmed
                    ),
                }
            }

            other => panic!("Unknown static type '{}'. Expected 'integer', 'double', 'boolean', or 'string'", other),
        }
    }
}

fn operator_to_str(op: LexerTagType) -> &'static str {
    match op {
        LexerTagType::Add => "+",
        LexerTagType::Subtract => "-",
        LexerTagType::Multiply => "*",
        LexerTagType::Divide => "/",
        LexerTagType::And => "&&",
        LexerTagType::Or => "||",
        LexerTagType::Not => "!",

    }
}
