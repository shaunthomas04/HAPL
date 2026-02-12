use crate::HtmlTag;

#[derive(Debug, Clone, Copy)]
enum LexerTagType {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug)]
pub enum HaplToken {
    OpenOperator { name: LexerTagType },
    CloseOperator { name: LexerTagType },

    OpenHtmlTag { name: String },
    CloseHtmlTag { name: String },

    Text(String),
    Number(i64),
}

pub struct HaplLexer {
    tokens: Vec<HaplToken>,
}

impl HaplLexer {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
        }
    }

    pub fn lex(&mut self, tag: &HtmlTag) {
        self.walk(tag);
    }

    fn walk(&mut self, tag: &HtmlTag) {
        //Arithmetic <div class="+">
        if tag.tag_type == "div" {
            if let Some(class) = &tag.class {
                if ["+", "-", "*", "/"].contains(&class.as_str()) {
                    let (open_token, close_token) = get_arithmetic_tags(class);

                    self.tokens.push(open_token);

                    for child in &tag.child_tags {
                        self.walk(child);
                    }

                    self.tokens.push(close_token);
                    return; // STOP further processing
                }
            }
        }

        // -------------------------------------------------
        // 2️⃣ <span> → ONLY emit literal, never OpenHtmlTag
        // -------------------------------------------------
        if tag.tag_type == "span" {
            if tag.child_tags.is_empty() && !tag.content.trim().is_empty() {
                let literal = parse_number(&tag.content)
                    .unwrap_or_else(|| parse_text(&tag.content));

                self.tokens.push(literal);
            } else {
                for child in &tag.child_tags {
                    self.walk(child);
                }
            }

            return; // IMPORTANT: prevent generic handling
        }

        // -------------------------------------------------
        // 3️⃣ Structural HTML tags (html, head, body, etc.)
        // -------------------------------------------------
        let structural = ["html", "head", "body", "title"];

        if structural.contains(&tag.tag_type.as_str()) {
            self.tokens.push(HaplToken::OpenHtmlTag {
                name: tag.tag_type.clone(),
            });

            for child in &tag.child_tags {
                self.walk(child);
            }

            self.tokens.push(HaplToken::CloseHtmlTag {
                name: tag.tag_type.clone(),
            });

            return;
        }

        // -------------------------------------------------
        // 4️⃣ Leaf literal (ONLY if true leaf)
        // -------------------------------------------------
        if tag.child_tags.is_empty() && !tag.content.trim().is_empty() {
            let literal = parse_number(&tag.content)
                .unwrap_or_else(|| parse_text(&tag.content));

            self.tokens.push(literal);
        } else {
            for child in &tag.child_tags {
                self.walk(child);
            }
        }
    }

    pub fn print(&self) {
        for token in &self.tokens {
            match token {
                HaplToken::OpenOperator { name } => {
                    println!("OpenOperator({})", operator_to_str(*name));
                }
                HaplToken::CloseOperator { name } => {
                    println!("CloseOperator({})", operator_to_str(*name));
                }
                HaplToken::OpenHtmlTag { name } => {
                    println!("OpenHtmlTag({})", name);
                }
                HaplToken::CloseHtmlTag { name } => {
                    println!("CloseHtmlTag({})", name);
                }
                HaplToken::Text(text) => println!("Text(\"{}\")", text),
                HaplToken::Number(n) => println!("Number({})", n),
            }
        }
    }
}

// -------------------------------------------------
// Helpers
// -------------------------------------------------

fn get_arithmetic_tags(operator: &str) -> (HaplToken, HaplToken) {
    let tag_type = match operator {
        "+" => LexerTagType::Add,
        "-" => LexerTagType::Subtract,
        "*" => LexerTagType::Multiply,
        "/" => LexerTagType::Divide,
        _ => panic!("Unknown operator"),
    };

    (
        HaplToken::OpenOperator { name: tag_type },
        HaplToken::CloseOperator { name: tag_type },
    )
}

fn operator_to_str(op: LexerTagType) -> &'static str {
    match op {
        LexerTagType::Add => "+",
        LexerTagType::Subtract => "-",
        LexerTagType::Multiply => "*",
        LexerTagType::Divide => "/",
    }
}

fn parse_number(text: &str) -> Option<HaplToken> {
    text.trim()
        .parse::<i64>()
        .ok()
        .map(HaplToken::Number)
}

fn parse_text(text: &str) -> HaplToken {
    HaplToken::Text(text.trim().to_string())
}
