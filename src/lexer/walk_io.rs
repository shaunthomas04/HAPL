use super::*;
use crate::error::{ErrorCode, HaplError, lexer_err};

impl HaplLexer {
    /// `<div class="length">`
    pub(super) fn walk_length(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        self.tokens.push(HaplToken::new(
            HaplTokenType::OpenLength,
            Some("length".to_string()),
        ));
        for child in &tag.child_tags {
            self.walk(child)?;
        }
        self.tokens.push(HaplToken::new(
            HaplTokenType::CloseLength,
            Some("length".to_string()),
        ));
        Ok(())
    }

    /// `<div class="http-get" id="response">`
    pub(super) fn walk_http_get(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        let name = tag.id.clone().ok_or_else(|| {
            lexer_err(ErrorCode::MissingId, "http-get is missing an id attribute")
                .with_tag(tag_snippet(tag), "id is required to store the response as a map")
                .with_hint("example: <div class=\"http-get\" id=\"response\">")
        })?;

        self.tokens.push(HaplToken::open_http_get(name.clone()));

        for child in &tag.child_tags {
            match child.class.as_deref() {
                Some("url") => {
                    self.tokens.push(HaplToken::open_http_url());
                    for grandchild in &child.child_tags { self.walk(grandchild)?; }
                    self.tokens.push(HaplToken::close_http_url());
                }
                other => {
                    return Err(lexer_err(
                        ErrorCode::UnknownTag,
                        format!("unexpected child '{}' inside http-get", other.unwrap_or("(none)")),
                    )
                    .with_tag(tag_snippet(child), "only <div class=\"url\"> is valid here")
                    .with_hint(
                        "example: <div class=\"url\"><span class=\"string\">\"https://...\"</span></div>",
                    ));
                }
            }
        }

        self.tokens.push(HaplToken::close_http_get(name));
        Ok(())
    }

    /// `<div class="http-post" id="response">`
    pub(super) fn walk_http_post(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        let name = tag.id.clone().ok_or_else(|| {
            lexer_err(ErrorCode::MissingId, "http-post is missing an id attribute")
                .with_tag(tag_snippet(tag), "id is required to store the response as a map")
                .with_hint("example: <div class=\"http-post\" id=\"response\">")
        })?;

        self.tokens.push(HaplToken::open_http_post(name.clone()));

        for child in &tag.child_tags {
            match child.class.as_deref() {
                Some("url") => {
                    self.tokens.push(HaplToken::open_http_url());
                    for grandchild in &child.child_tags { self.walk(grandchild)?; }
                    self.tokens.push(HaplToken::close_http_url());
                }
                Some("body") => {
                    self.tokens.push(HaplToken::open_http_body());
                    for grandchild in &child.child_tags { self.walk(grandchild)?; }
                    self.tokens.push(HaplToken::close_http_body());
                }
                other => {
                    return Err(lexer_err(
                        ErrorCode::UnknownTag,
                        format!("unexpected child '{}' inside http-post", other.unwrap_or("(none)")),
                    )
                    .with_tag(
                        tag_snippet(child),
                        "only <div class=\"url\"> and <div class=\"body\"> are valid here",
                    )
                    .with_hint(
                        "example: <div class=\"body\"><var class=\"myMap\"></var></div>",
                    ));
                }
            }
        }

        self.tokens.push(HaplToken::close_http_post(name));
        Ok(())
    }

    /// `<div class="server" id="8080">`
    pub(super) fn walk_server(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        let port_str = tag.id.clone().ok_or_else(|| {
            lexer_err(ErrorCode::MissingId, "server is missing a port in the id attribute")
                .with_tag(tag_snippet(tag), "id attribute must be the port number")
                .with_hint("example: <div class=\"server\" id=\"8080\">")
        })?;

        let port = port_str.parse::<u16>().map_err(|_| {
            lexer_err(
                ErrorCode::InvalidLiteral,
                format!("server port '{}' is not a valid port number", port_str),
            )
            .with_tag(tag_snippet(tag), "port must be a number between 1 and 65535")
            .with_hint("example: <div class=\"server\" id=\"8080\">")
        })?;

        self.tokens.push(HaplToken::new(
            HaplTokenType::OpenServer { port },
            Some(port_str.clone()),
        ));

        for child in &tag.child_tags {
            match child.class.as_deref() {
                Some("endpoint-get") => {
                    let path = child.id.clone().ok_or_else(|| {
                        lexer_err(
                            ErrorCode::MissingId,
                            "endpoint-get is missing a path in the id attribute",
                        )
                        .with_tag(tag_snippet(child), "id attribute must be the endpoint path")
                        .with_hint("example: <div class=\"endpoint-get\" id=\"/users\">")
                    })?;

                    self.tokens.push(HaplToken::new(
                        HaplTokenType::OpenEndpointGet { path: path.clone() },
                        Some(path.clone()),
                    ));
                    for grandchild in &child.child_tags {
                        match grandchild.class.as_deref() {
                            Some("handler") => {
                                self.tokens.push(HaplToken::open_handler());
                                for gc in &grandchild.child_tags { self.walk(gc)?; }
                                self.tokens.push(HaplToken::close_handler());
                            }
                            other => return Err(lexer_err(
                                ErrorCode::UnknownTag,
                                format!(
                                    "unexpected child '{}' inside endpoint-get",
                                    other.unwrap_or("(none)")
                                ),
                            )
                            .with_tag(
                                tag_snippet(grandchild),
                                "only <div class=\"handler\"> is valid here",
                            )),
                        }
                    }
                    self.tokens.push(HaplToken::new(
                        HaplTokenType::CloseEndpointGet,
                        Some(path),
                    ));
                }

                Some("endpoint-post") => {
                    let path = child.id.clone().ok_or_else(|| {
                        lexer_err(
                            ErrorCode::MissingId,
                            "endpoint-post is missing a path in the id attribute",
                        )
                        .with_tag(tag_snippet(child), "id attribute must be the endpoint path")
                        .with_hint("example: <div class=\"endpoint-post\" id=\"/users\">")
                    })?;

                    self.tokens.push(HaplToken::new(
                        HaplTokenType::OpenEndpointPost { path: path.clone() },
                        Some(path.clone()),
                    ));
                    for grandchild in &child.child_tags {
                        match grandchild.class.as_deref() {
                            Some("handler") => {
                                self.tokens.push(HaplToken::open_handler());
                                for gc in &grandchild.child_tags { self.walk(gc)?; }
                                self.tokens.push(HaplToken::close_handler());
                            }
                            other => return Err(lexer_err(
                                ErrorCode::UnknownTag,
                                format!(
                                    "unexpected child '{}' inside endpoint-post",
                                    other.unwrap_or("(none)")
                                ),
                            )
                            .with_tag(
                                tag_snippet(grandchild),
                                "only <div class=\"handler\"> is valid here",
                            )),
                        }
                    }
                    self.tokens.push(HaplToken::new(
                        HaplTokenType::CloseEndpointPost,
                        Some(path),
                    ));
                }

                other => return Err(lexer_err(
                    ErrorCode::UnknownTag,
                    format!("unexpected child '{}' inside server", other.unwrap_or("(none)")),
                )
                .with_tag(
                    tag_snippet(child),
                    "only endpoint-get and endpoint-post are valid here",
                )
                .with_hint("example: <div class=\"endpoint-get\" id=\"/users\">")),
            }
        }

        self.tokens.push(HaplToken::new(HaplTokenType::CloseServer, Some(port_str)));
        Ok(())
    }

    /// `<div class="respond">`
    pub(super) fn walk_respond(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        self.tokens.push(HaplToken::open_respond());
        for child in &tag.child_tags { self.walk(child)?; }
        self.tokens.push(HaplToken::close_respond());
        Ok(())
    }
}
