use super::*;
use crate::error::{ErrorCode, HaplError, lexer_err};

impl HaplLexer {
    /// `<div class="conditional">` → if / elif / else chain
    pub(super) fn walk_conditional(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        self.tokens.push(HaplToken::open_conditional());

        for child in &tag.child_tags {
            if child.tag_type != "div" { continue; }
            match child.class.as_deref() {
                Some("if") => {
                    self.tokens.push(HaplToken::open_if());
                    for grandchild in &child.child_tags { self.walk(grandchild)?; }
                    self.tokens.push(HaplToken::close_if());
                }
                Some("elif") => {
                    self.tokens.push(HaplToken::open_elif());
                    for grandchild in &child.child_tags { self.walk(grandchild)?; }
                    self.tokens.push(HaplToken::close_elif());
                }
                Some("else") => {
                    self.tokens.push(HaplToken::open_else());
                    for grandchild in &child.child_tags { self.walk(grandchild)?; }
                    self.tokens.push(HaplToken::close_else());
                }
                other => {
                    return Err(lexer_err(
                        ErrorCode::BadConditionalChild,
                        format!(
                            "unexpected child '{}' inside <div class=\"conditional\">",
                            other.unwrap_or("(none)")
                        ),
                    )
                    .with_tag(
                        tag_snippet(child),
                        format!("'{}' is not valid here", other.unwrap_or("(none)")),
                    )
                    .with_hint("valid children of conditional: if, elif, else"));
                }
            }
        }

        self.tokens.push(HaplToken::close_conditional());
        Ok(())
    }

    /// `<div class="while">` → while loop
    pub(super) fn walk_while(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        self.tokens.push(HaplToken::open_loop(LoopType::While));

        for child in &tag.child_tags {
            if child.tag_type != "div" { continue; }
            match child.class.as_deref() {
                Some("condition") => {
                    self.tokens.push(HaplToken::open_loop_condition());
                    for grandchild in &child.child_tags { self.walk(grandchild)?; }
                    self.tokens.push(HaplToken::close_loop_condition());
                }
                Some("body") => {
                    self.tokens.push(HaplToken::open_loop_body());
                    for grandchild in &child.child_tags { self.walk(grandchild)?; }
                    self.tokens.push(HaplToken::close_loop_body());
                }
                other => {
                    return Err(lexer_err(
                        ErrorCode::BadLoopChild,
                        format!(
                            "unexpected child '{}' inside <div class=\"while\">",
                            other.unwrap_or("(none)")
                        ),
                    )
                    .with_tag(
                        tag_snippet(child),
                        format!("'{}' is not valid here", other.unwrap_or("(none)")),
                    )
                    .with_hint("valid children of while: condition, body"));
                }
            }
        }

        self.tokens.push(HaplToken::close_loop(LoopType::While));
        Ok(())
    }

    /// `<div class="for">` → for loop
    pub(super) fn walk_for(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        self.tokens.push(HaplToken::open_loop(LoopType::For));

        for child in &tag.child_tags {
            if child.tag_type != "div" { continue; }
            match child.class.as_deref() {
                Some("iterator") => {
                    self.tokens.push(HaplToken::open_loop_iterator());
                    for grandchild in &child.child_tags { self.walk(grandchild)?; }
                    self.tokens.push(HaplToken::close_loop_iterator());
                }
                Some("condition") => {
                    self.tokens.push(HaplToken::open_loop_condition());
                    for grandchild in &child.child_tags { self.walk(grandchild)?; }
                    self.tokens.push(HaplToken::close_loop_condition());
                }
                Some("increment") => {
                    self.tokens.push(HaplToken::open_loop_increment());
                    for grandchild in &child.child_tags { self.walk(grandchild)?; }
                    self.tokens.push(HaplToken::close_loop_increment());
                }
                Some("body") => {
                    self.tokens.push(HaplToken::open_loop_body());
                    for grandchild in &child.child_tags { self.walk(grandchild)?; }
                    self.tokens.push(HaplToken::close_loop_body());
                }
                other => {
                    return Err(lexer_err(
                        ErrorCode::BadLoopChild,
                        format!(
                            "unexpected child '{}' inside <div class=\"for\">",
                            other.unwrap_or("(none)")
                        ),
                    )
                    .with_tag(
                        tag_snippet(child),
                        format!("'{}' is not valid here", other.unwrap_or("(none)")),
                    )
                    .with_hint("valid children of for: iterator, condition, increment, body"));
                }
            }
        }

        self.tokens.push(HaplToken::close_loop(LoopType::For));
        Ok(())
    }
}
