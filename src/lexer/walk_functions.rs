use super::*;
use crate::ast::StaticType;
use crate::error::{ErrorCode, HaplError, lexer_err};

impl HaplLexer {
    /// Parse the return type from a `<type>-function` class string.
    pub(super) fn parse_function_class(&self, class: &str) -> Option<StaticType> {
        let type_part = class.strip_suffix("-function")?;
        match type_part {
            "integer" => Some(StaticType::Integer),
            "double"  => Some(StaticType::Double),
            "string"  => Some(StaticType::String),
            "boolean" => Some(StaticType::Boolean),
            "void"    => Some(StaticType::Void),
            "map"     => Some(StaticType::Map),
            _         => None,
        }
    }

    /// Parse the param type from a `<type>-param` class string.
    pub(super) fn parse_param_class(&self, class: Option<&str>) -> Option<StaticType> {
        let type_part = class?.strip_suffix("-param")?;
        match type_part {
            "integer" => Some(StaticType::Integer),
            "double"  => Some(StaticType::Double),
            "string"  => Some(StaticType::String),
            "boolean" => Some(StaticType::Boolean),
            "void"    => Some(StaticType::Void),
            "map"     => Some(StaticType::Map),
            _         => None,
        }
    }

    /// Function declaration: `<div class="integer-function" id="myFunc">`
    pub(super) fn walk_function_decl(
        &mut self,
        tag: &HtmlTag,
        name: String,
        return_type: StaticType,
    ) -> Result<(), HaplError> {
        self.tokens.push(HaplToken::new(
            HaplTokenType::OpenFunction { name: name.clone(), return_type },
            Some(name.clone()),
        ));

        for child in &tag.child_tags {
            match child.class.as_deref() {
                Some("params") => {
                    self.tokens.push(HaplToken::open_params());

                    for grandchild in &child.child_tags {
                        let param_type = self
                            .parse_param_class(grandchild.class.as_deref())
                            .ok_or_else(|| {
                                lexer_err(
                                    ErrorCode::UnknownParamType,
                                    format!(
                                        "invalid or missing param type on parameter: class={:?}",
                                        grandchild.class
                                    ),
                                )
                                .with_tag(
                                    tag_snippet(grandchild),
                                    format!(
                                        "'{}' is not a valid param type",
                                        grandchild.class.as_deref().unwrap_or("(none)")
                                    ),
                                )
                                .with_hint(
                                    "valid param types: integer-param, double-param, \
                                     string-param, boolean-param",
                                )
                            })?;

                        let param_name = grandchild.id.clone().ok_or_else(|| {
                            lexer_err(ErrorCode::MissingId, "parameter tag is missing an id attribute")
                                .with_tag(tag_snippet(grandchild), "id is required to name the parameter")
                                .with_hint("example: <div class=\"integer-param\" id=\"age\"></div>")
                        })?;

                        self.tokens.push(HaplToken::new(
                            HaplTokenType::OpenParam {
                                name: param_name.clone(),
                                param_type,
                            },
                            Some(param_name.clone()),
                        ));
                        for param_child in &grandchild.child_tags {
                            self.walk(param_child)?;
                        }
                        self.tokens.push(HaplToken::new(
                            HaplTokenType::CloseParam { name: param_name.clone() },
                            Some(param_name),
                        ));
                    }

                    self.tokens.push(HaplToken::close_params());
                }

                Some("body") => {
                    self.tokens.push(HaplToken::open_function_body());
                    for grandchild in &child.child_tags {
                        self.walk(grandchild)?;
                    }
                    self.tokens.push(HaplToken::close_function_body());
                }

                other => {
                    return Err(lexer_err(
                        ErrorCode::BadFunctionChild,
                        format!(
                            "unexpected child '{}' inside function declaration '{}'",
                            other.unwrap_or("(none)"),
                            name
                        ),
                    )
                    .with_tag(
                        tag_snippet(child),
                        format!("'{}' is not valid here", other.unwrap_or("(none)")),
                    )
                    .with_hint("valid children of a function declaration: params, body"));
                }
            }
        }

        self.tokens.push(HaplToken::new(
            HaplTokenType::CloseFunction { name: name.clone() },
            Some(name),
        ));
        Ok(())
    }

    /// Function call: `<div class="funcName">`
    pub(super) fn walk_function_call(&mut self, tag: &HtmlTag, name: String) -> Result<(), HaplError> {
        self.tokens.push(HaplToken::new(
            HaplTokenType::OpenFunctionCall { name: name.clone() },
            Some(name.clone()),
        ));

        for child in &tag.child_tags {
            if child.class.as_deref() == Some("args") {
                self.tokens.push(HaplToken::open_args());
                for grandchild in &child.child_tags {
                    self.walk(grandchild)?;
                }
                self.tokens.push(HaplToken::close_args());
            } else {
                self.walk(child)?;
            }
        }

        self.tokens.push(HaplToken::new(
            HaplTokenType::CloseFunctionCall { name: name.clone() },
            Some(name),
        ));
        Ok(())
    }
}
