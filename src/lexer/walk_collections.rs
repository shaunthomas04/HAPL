use super::*;
use crate::ast::StaticType;
use crate::error::{ErrorCode, HaplError, lexer_err};

impl HaplLexer {
    /// Parse `<type>-list` class into a `StaticType` element type.
    pub(super) fn parse_list_class(&self, class: &str) -> Option<StaticType> {
        match class.strip_suffix("-list")? {
            "integer" => Some(StaticType::Integer),
            "double"  => Some(StaticType::Double),
            "string"  => Some(StaticType::String),
            "boolean" => Some(StaticType::Boolean),
            _         => None,
        }
    }

    /// Extract the list/map variable name from the first `<var>` child of an operation tag.
    pub(super) fn extract_list_name(&self, tag: &HtmlTag, op: &str) -> Result<String, HaplError> {
        let first = tag.child_tags.first().ok_or_else(|| {
            lexer_err(ErrorCode::MissingId, format!("<div class=\"{}\"> has no children", op))
                .with_tag(tag_snippet(tag), "expected a <var> reference as the first child")
                .with_hint(format!("example: <div class=\"{}\"><var class=\"myList\"></var>...</div>", op))
        })?;

        if first.tag_type != "var" {
            return Err(lexer_err(
                ErrorCode::UnexpectedToken,
                format!(
                    "first child of <div class=\"{}\"> must be a <var>, got <{}>",
                    op, first.tag_type
                ),
            )
            .with_tag(tag_snippet(first), "expected a <var> reference here")
            .with_hint("the first child must identify the list by name"));
        }

        first.class.clone().ok_or_else(|| {
            lexer_err(
                ErrorCode::MissingClass,
                format!("<var> inside <div class=\"{}\"> is missing a class attribute", op),
            )
            .with_tag(tag_snippet(first), "class attribute is the list name")
            .with_hint("example: <var class=\"myList\"></var>")
        })
    }

    /// Extract the map variable name from the first `<var>` child of a map operation tag.
    pub(super) fn extract_map_name(&self, tag: &HtmlTag, op: &str) -> Result<String, HaplError> {
        let first = tag.child_tags.first().ok_or_else(|| {
            lexer_err(ErrorCode::MissingId, format!("<div class=\"{}\"> has no children", op))
                .with_tag(tag_snippet(tag), "expected a <var> reference as the first child")
                .with_hint(format!("example: <div class=\"{}\"><var class=\"myMap\"></var>...</div>", op))
        })?;

        if first.tag_type != "var" {
            return Err(lexer_err(
                ErrorCode::UnexpectedToken,
                format!(
                    "first child of <div class=\"{}\"> must be a <var>, got <{}>",
                    op, first.tag_type
                ),
            )
            .with_tag(tag_snippet(first), "expected a <var> reference here")
            .with_hint("the first child must identify the map by name"));
        }

        first.class.clone().ok_or_else(|| {
            lexer_err(
                ErrorCode::MissingClass,
                format!("<var> inside <div class=\"{}\"> is missing a class attribute", op),
            )
            .with_tag(tag_snippet(first), "class attribute is the map name")
            .with_hint("example: <var class=\"myMap\"></var>")
        })
    }

    // ------------------------------------------------------------------
    // List operations
    // ------------------------------------------------------------------

    pub(super) fn walk_list_decl(
        &mut self,
        tag: &HtmlTag,
        name: String,
        elem_type: StaticType,
    ) -> Result<(), HaplError> {
        self.tokens.push(HaplToken::open_list_dec(elem_type, name.clone()));
        for child in &tag.child_tags {
            if child.tag_type != "span" {
                return Err(lexer_err(
                    ErrorCode::BadListChild,
                    format!(
                        "unexpected <{}> inside list declaration '{}'",
                        child.tag_type, name
                    ),
                )
                .with_tag(
                    tag_snippet(child),
                    "only literal <span> values are valid inside a list declaration",
                )
                .with_hint("example: <span class=\"integer\">42</span>"));
            }
            self.walk(child)?;
        }
        self.tokens.push(HaplToken::close_list_dec(name));
        Ok(())
    }

    pub(super) fn walk_list_access(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        self.tokens.push(HaplToken::open_list_access());
        for child in &tag.child_tags { self.walk(child)?; }
        self.tokens.push(HaplToken::close_list_access());
        Ok(())
    }

    pub(super) fn walk_list_assign(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        let name = self.extract_list_name(tag, "index-assign")?;
        self.tokens.push(HaplToken::open_list_assign(name.clone()));
        for child in &tag.child_tags { self.walk(child)?; }
        self.tokens.push(HaplToken::close_list_assign(name));
        Ok(())
    }

    pub(super) fn walk_list_push(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        let name = self.extract_list_name(tag, "push")?;
        self.tokens.push(HaplToken::open_list_push(name.clone()));
        for child in &tag.child_tags { self.walk(child)?; }
        self.tokens.push(HaplToken::close_list_push(name));
        Ok(())
    }

    pub(super) fn walk_list_pop(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        let name = self.extract_list_name(tag, "pop")?;
        self.tokens.push(HaplToken::open_list_pop(name.clone()));
        self.tokens.push(HaplToken::close_list_pop(name));
        Ok(())
    }

    // ------------------------------------------------------------------
    // Map operations
    // ------------------------------------------------------------------

    pub(super) fn walk_map_dec(&mut self, tag: &HtmlTag, name: String) -> Result<(), HaplError> {
        self.tokens.push(HaplToken::open_map_dec(name.clone()));

        for child in &tag.child_tags {
            let key = child.class.clone().ok_or_else(|| {
                lexer_err(
                    ErrorCode::MissingClass,
                    format!("map entry inside '{}' is missing a class attribute", name),
                )
                .with_tag(tag_snippet(child), "the class attribute is used as the map key")
                .with_hint("example: <div class=\"age\"><span class=\"integer\">30</span></div>")
            })?;

            self.tokens.push(HaplToken::open_map_entry(key.clone()));
            for grandchild in &child.child_tags {
                self.walk(grandchild)?;
            }
            self.tokens.push(HaplToken::close_map_entry(key));
        }

        self.tokens.push(HaplToken::close_map_dec(name));
        Ok(())
    }

    pub(super) fn walk_map_get(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        self.tokens.push(HaplToken::open_map_get());
        for child in &tag.child_tags {
            if child.class.as_deref() == Some("key") {
                self.tokens.push(HaplToken::open_map_key());
                for grandchild in &child.child_tags { self.walk(grandchild)?; }
                self.tokens.push(HaplToken::close_map_key());
            } else {
                self.walk(child)?;
            }
        }
        self.tokens.push(HaplToken::close_map_get());
        Ok(())
    }

    pub(super) fn walk_map_contains(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        self.tokens.push(HaplToken::open_map_contains());
        for child in &tag.child_tags {
            if child.class.as_deref() == Some("key") {
                self.tokens.push(HaplToken::open_map_key());
                for grandchild in &child.child_tags { self.walk(grandchild)?; }
                self.tokens.push(HaplToken::close_map_key());
            } else {
                self.walk(child)?;
            }
        }
        self.tokens.push(HaplToken::close_map_contains());
        Ok(())
    }

    pub(super) fn walk_map_set(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        let name = self.extract_map_name(tag, "map-set")?;
        self.tokens.push(HaplToken::open_map_set(name.clone()));

        for child in &tag.child_tags {
            match child.class.as_deref() {
                Some("key") => {
                    self.tokens.push(HaplToken::open_map_key());
                    for grandchild in &child.child_tags { self.walk(grandchild)?; }
                    self.tokens.push(HaplToken::close_map_key());
                }
                Some("value") => {
                    self.tokens.push(HaplToken::open_map_value());
                    for grandchild in &child.child_tags { self.walk(grandchild)?; }
                    self.tokens.push(HaplToken::close_map_value());
                }
                _ => { self.walk(child)?; }
            }
        }

        self.tokens.push(HaplToken::close_map_set(name));
        Ok(())
    }

    pub(super) fn walk_map_remove(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        let name = self.extract_map_name(tag, "map-remove")?;
        self.tokens.push(HaplToken::open_map_remove(name.clone()));

        for child in &tag.child_tags {
            if child.class.as_deref() == Some("key") {
                self.tokens.push(HaplToken::open_map_key());
                for grandchild in &child.child_tags { self.walk(grandchild)?; }
                self.tokens.push(HaplToken::close_map_key());
            } else {
                self.walk(child)?;
            }
        }

        self.tokens.push(HaplToken::close_map_remove(name));
        Ok(())
    }
}
