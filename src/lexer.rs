use crate::HtmlTag;

#[derive(Debug, Clone)]


enum AllowedHtmlTags {
    span,
    div
}

#[derive(Debug, Clone, Copy)]
enum LexerTagType {
    Add,
    Subtract,
    Multiply,
    Divide,
}

//possible hapl token types
pub enum HaplToken {
    OpenTag { name: LexerTagType },
    CloseTag { name: LexerTagType },
    Text(String),
    Number(i64),
}


pub struct HaplLexer {
    tokens: Vec<HaplToken>,
}

impl HaplLexer {
    // Create a new HAPL lexer
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
        }
    }

    // Consume an HtmlTag tree and add tokens to self vector
    pub fn lex(&mut self, tag: &HtmlTag) {
        self.walk(tag);
    }

    // Recursive walk over the HtmlTag tree
    fn walk(&mut self, tag: &HtmlTag) {
        // opening tag
        self.tokens.push(HaplToken::OpenTag {
            name: tag.tag_type.clone(),
        });

        // leaf node (text or number)
        if tag.child_tags.is_empty() {
            let text = tag.content.trim();

            if !text.is_empty() {
                if let Ok(n) = text.parse::<i64>() {
                    self.tokens.push(HaplToken::Number(n));
                } else {
                    self.tokens.push(HaplToken::Text(text.to_string()));
                }
            }
        }

        // recurse children nodes
        for child in &tag.child_tags {
            self.walk(child);
        }

        // closing tag
        self.tokens.push(HaplToken::CloseTag {
            name: tag.tag_type.clone(),
        });
    }

    pub fn print(&self) {
        if self.tokens.is_empty() {
            println!("No HAPL tokens to print.");
            return;
        }

        for token in &self.tokens {
            match token {
                HaplToken::OpenTag { name } => println!("OpenTag({})", name),
                HaplToken::CloseTag { name } => println!("CloseTag({})", name),
                HaplToken::Text(text) => println!("Text(\"{}\")", text),
                HaplToken::Number(n) => println!("Number({})", n),
            }
        }
    }
}


fn get_arithmetic_tags(operator: &str) -> (HaplToken, HaplToken) {
    // Map operator string to LexerTagType
    let tag_type = match operator {
        "+" => LexerTagType::Add,
        "-" => LexerTagType::Subtract,
        "*" => LexerTagType::Multiply,
        "/" => LexerTagType::Divide,
        _ => panic!("Unknown operator"),
    };

    // Create two tokens
    let token1 = HaplToken::OpenTag { name: tag_type };
    let token2 = HaplToken::CloseTag { name: tag_type };

    (token1, token2)
}

// Small function to parse numbers
fn parse_number(text: &str) -> Option<HaplToken> {
    if let Ok(n) = text.trim().parse::<i64>() {
        Some(HaplToken::Number(n))
    } else {
        None
    }
}

// Small function to parse strings
fn parse_text(text: &str) -> HaplToken {
    HaplToken::Text(text.trim().to_string())
}

// Main function called for HTML tag content
fn get_literal_value(html_tag_content: &str, tag: &str) -> HaplToken {
    match tag {
        "span" => parse_number(html_tag_content).unwrap_or_else(|| {
            // fallback if number parsing fails
            parse_text(html_tag_content)
        }),
        "p" => parse_text(html_tag_content),
        _ => parse_text(html_tag_content), // default fallback
    }
}
