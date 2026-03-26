use super::*;
use crate::error::{ErrorCode, HaplError, lexer_err};
use crate::ast::StaticType;

impl HaplLexer {
    /// `<p>` → print statement
    pub(super) fn walk_print(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        self.tokens.push(HaplToken::open_print());
        for child in &tag.child_tags {
            self.walk(child)?;
        }
        self.tokens.push(HaplToken::close_print());
        Ok(())
    }

    /// `<var>` → variable declaration, reference, or assignment
    pub(super) fn walk_var(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        let raw_class = tag.class.as_ref().ok_or_else(|| {
            lexer_err(ErrorCode::MissingClass, "missing class attribute on <var> tag")
                .with_tag(
                    tag_snippet(tag),
                    "expected class=\"<type>\" for declaration or class=\"<name>\" for reference",
                )
                .with_hint("example: <var class=\"integer\" id=\"x\">10</var>")
        })?;

        let class = self.resolve(raw_class).to_string();

        if let Some(id) = &tag.id {
            // Variable declaration: <var class="integer" id="x">10</var>
            let var_type = parse_static_type(&class).ok_or_else(|| {
                lexer_err(
                    ErrorCode::UnknownType,
                    format!("unknown type '{}' in variable declaration", class),
                )
                .with_tag(tag_snippet(tag), format!("'{}' is not a valid type", class))
                .with_hint("valid types: integer, double, string, boolean, map")
            })?;

            self.tokens.push(HaplToken::new(
                HaplTokenType::OpenVarDec { var_type: var_type.clone(), name: id.clone() },
                Some(format!("{} {}", class, id)),
            ));
            for child in &tag.child_tags {
                self.walk(child)?;
            }
            self.tokens.push(HaplToken::new(
                HaplTokenType::CloseVarDec { var_type, name: id.clone() },
                Some(format!("{} {}", class, id)),
            ));
        } else if tag.child_tags.is_empty() {
            // Variable reference: <var class="x"></var>
            self.tokens.push(HaplToken::new(
                HaplTokenType::OpenVarRef { name: class.clone() },
                Some(class.clone()),
            ));
            self.tokens.push(HaplToken::new(
                HaplTokenType::CloseVarRef { name: class.clone() },
                Some(class.clone()),
            ));
        } else {
            // Variable assignment: <var class="x"><span ...>value</span></var>
            self.tokens.push(HaplToken::new(
                HaplTokenType::OpenVarAssign { name: class.clone() },
                Some(class.clone()),
            ));
            for child in &tag.child_tags {
                self.walk(child)?;
            }
            self.tokens.push(HaplToken::new(
                HaplTokenType::CloseVarAssign { name: class.clone() },
                Some(class.clone()),
            ));
        }
        Ok(())
    }

    /// `<span>` → literal value
    pub(super) fn walk_span(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        if tag.child_tags.is_empty() && !tag.content.trim().is_empty() {
            let token = self.parse_literal(tag)?;
            self.tokens.push(token);
        } else {
            for child in &tag.child_tags {
                self.walk(child)?;
            }
        }
        Ok(())
    }

    /// Parse a literal value from a `<span>` tag.
    pub(super) fn parse_literal(&self, tag: &HtmlTag) -> Result<HaplToken, HaplError> {
        let trimmed = tag.content.trim();

        let raw_class = tag.class.as_deref().ok_or_else(|| {
            lexer_err(
                ErrorCode::MissingClass,
                format!("literal tag containing '{}' has no class attribute", trimmed),
            )
            .with_tag(tag_snippet(tag), "class attribute required to identify the type")
            .with_hint("example: <span class=\"integer\">42</span>")
        })?;
        let class = self.resolve(raw_class);

        match class {
            "integer" => trimmed.parse::<i64>().map(HaplToken::integer).map_err(|_| {
                lexer_err(
                    ErrorCode::InvalidLiteral,
                    format!("'{}' is not a valid integer", trimmed),
                )
                .with_tag(tag_snippet(tag), format!("cannot parse '{}' as integer", trimmed))
                .with_hint("integer literals must be whole numbers, e.g. 42 or -7")
            }),

            "double" => trimmed.parse::<f64>().map(HaplToken::double).map_err(|_| {
                lexer_err(
                    ErrorCode::InvalidLiteral,
                    format!("'{}' is not a valid double", trimmed),
                )
                .with_tag(tag_snippet(tag), format!("cannot parse '{}' as double", trimmed))
                .with_hint("double literals must be floating-point numbers, e.g. 3.14")
            }),

            "string" => {
                if !trimmed.starts_with('"') || !trimmed.ends_with('"') || trimmed.len() < 2 {
                    return Err(lexer_err(
                        ErrorCode::StringNotQuoted,
                        format!("string literal '{}' is not enclosed in double quotes", trimmed),
                    )
                    .with_tag(tag_snippet(tag), "string content must be wrapped in double quotes")
                    .with_hint("example: <span class=\"string\">\"hello world\"</span>"));
                }
                let inner = &trimmed[1..trimmed.len() - 1];
                Ok(HaplToken::string(inner.to_string()))
            }

            "boolean" => match trimmed {
                "true"  => Ok(HaplToken::boolean(true)),
                "false" => Ok(HaplToken::boolean(false)),
                _ => Err(lexer_err(
                    ErrorCode::InvalidLiteral,
                    format!("'{}' is not a valid boolean", trimmed),
                )
                .with_tag(tag_snippet(tag), format!("expected 'true' or 'false', got '{}'", trimmed))
                .with_hint("boolean literals must be exactly true or false")),
            },

            other => Err(lexer_err(
                ErrorCode::UnknownType,
                format!("unknown literal type '{}'", other),
            )
            .with_tag(tag_snippet(tag), format!("'{}' is not a recognized type", other))
            .with_hint("valid literal types: integer, double, boolean, string")),
        }
    }
}
