use crate::HtmlTag;

#[derive(Debug, Clone)]

//possible hapl token types
pub enum HaplToken {
    OpenTag { name: String },
    CloseTag { name: String },
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
