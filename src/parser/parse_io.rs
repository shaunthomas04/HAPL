use super::*;
use crate::ast::{Endpoint, Expr, HttpMethod, StaticType};
use crate::error::{ErrorCode, HaplError, parser_err};
use crate::lexer::HaplTokenType;

impl HaplParser {
    /// Entry point called from `parse_expression` for HTTP tokens.
    pub(super) fn parse_io_expression(&mut self) -> Result<Expr, HaplError> {
        match self.current_token() {
            HaplTokenType::OpenHttpGet { .. }  => self.parse_http_get(),
            HaplTokenType::OpenHttpPost { .. } => self.parse_http_post(),
            other => Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!("unexpected token '{:?}' in IO expression", other),
            )),
        }
    }

    // --------------------------------------------------
    // HTTP GET
    // --------------------------------------------------
    fn parse_http_get(&mut self) -> Result<Expr, HaplError> {
        let req_name = match self.current_token() {
            HaplTokenType::OpenHttpGet { name } => name.clone(),
            _ => unreachable!(),
        };
        self.advance();

        if !matches!(self.current_token(), HaplTokenType::OpenHttpUrl) {
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                "expected <div class=\"url\"> inside http-get",
            )
            .with_hint(
                "example: <div class=\"url\"><span class=\"string\">\"https://...\"</span></div>",
            ));
        }
        self.advance(); // consume OpenHttpUrl

        let url_expr = Box::new(self.parse_expression()?);

        if !matches!(self.current_token(), HaplTokenType::CloseHttpUrl) {
            return Err(
                parser_err(ErrorCode::TagNotClosed, "http-get url block was never closed")
                    .with_hint(
                        "add a matching closing tag for the <div class=\"url\"> block",
                    ),
            );
        }
        self.advance();

        if !matches!(self.current_token(),
            HaplTokenType::CloseHttpGet { name: ref n } if n == &req_name)
        {
            return Err(parser_err(
                ErrorCode::TagNotClosed,
                format!("http-get '{}' was never closed", req_name),
            )
            .with_hint("add a matching closing tag for the http-get block"));
        }
        self.advance();

        self.declare_var(req_name.clone(), StaticType::Map)?;

        Ok(Expr::HttpGet { name: req_name, url: url_expr })
    }

    // --------------------------------------------------
    // HTTP POST
    // --------------------------------------------------
    fn parse_http_post(&mut self) -> Result<Expr, HaplError> {
        let req_name = match self.current_token() {
            HaplTokenType::OpenHttpPost { name } => name.clone(),
            _ => unreachable!(),
        };
        self.advance();

        if !matches!(self.current_token(), HaplTokenType::OpenHttpUrl) {
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                "expected <div class=\"url\"> inside http-post",
            )
            .with_hint(
                "example: <div class=\"url\"><span class=\"string\">\"https://...\"</span></div>",
            ));
        }
        self.advance();

        let url_expr = Box::new(self.parse_expression()?);

        if !matches!(self.current_token(), HaplTokenType::CloseHttpUrl) {
            return Err(
                parser_err(ErrorCode::TagNotClosed, "http-post url block was never closed")
                    .with_hint(
                        "add a matching closing tag for the <div class=\"url\"> block",
                    ),
            );
        }
        self.advance();

        if !matches!(self.current_token(), HaplTokenType::OpenHttpBody) {
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                "expected <div class=\"body\"> inside http-post",
            )
            .with_hint(
                "example: <div class=\"body\"><var class=\"myMap\"></var></div>",
            ));
        }
        self.advance();

        let body_expr = Box::new(self.parse_expression()?);

        if !matches!(self.current_token(), HaplTokenType::CloseHttpBody) {
            return Err(
                parser_err(ErrorCode::TagNotClosed, "http-post body block was never closed")
                    .with_hint(
                        "add a matching closing tag for the <div class=\"body\"> block",
                    ),
            );
        }
        self.advance();

        if !matches!(self.current_token(),
            HaplTokenType::CloseHttpPost { name: ref n } if n == &req_name)
        {
            return Err(parser_err(
                ErrorCode::TagNotClosed,
                format!("http-post '{}' was never closed", req_name),
            )
            .with_hint("add a matching closing tag for the http-post block"));
        }
        self.advance();

        self.declare_var(req_name.clone(), StaticType::Map)?;

        Ok(Expr::HttpPost { name: req_name, url: url_expr, body: body_expr })
    }

    // --------------------------------------------------
    // Server declaration
    // --------------------------------------------------
    pub(super) fn parse_server(&mut self) -> Result<Expr, HaplError> {
        let port = match self.current_token() {
            HaplTokenType::OpenServer { port } => port,
            _ => unreachable!(),
        };
        self.advance();

        let mut endpoints = Vec::new();

        while !self.is_at_end() && !matches!(self.current_token(), HaplTokenType::CloseServer) {
            let (method, path) = match self.current_token() {
                HaplTokenType::OpenEndpointGet { path }  => (HttpMethod::Get,  path.clone()),
                HaplTokenType::OpenEndpointPost { path } => (HttpMethod::Post, path.clone()),
                other => {
                    return Err(parser_err(
                        ErrorCode::UnexpectedToken,
                        format!(
                            "expected endpoint-get or endpoint-post, got '{:?}'",
                            other
                        ),
                    ));
                }
            };
            self.advance(); // consume OpenEndpointGet/Post

            if !matches!(self.current_token(), HaplTokenType::OpenHandler) {
                return Err(parser_err(
                    ErrorCode::UnexpectedToken,
                    "expected <div class=\"handler\"> inside endpoint",
                )
                .with_hint("each endpoint must have a handler block"));
            }
            self.advance(); // consume OpenHandler

            self.push_scope();
            self.declare_var("params".to_string(), StaticType::Map)?;
            self.declare_var("body".to_string(), StaticType::Map)?;

            let mut handler = Vec::new();
            while !self.is_at_end()
                && !matches!(self.current_token(), HaplTokenType::CloseHandler)
            {
                handler.push(self.parse_statement()?);
            }
            self.pop_scope();

            if self.is_at_end() {
                return Err(parser_err(
                    ErrorCode::TagNotClosed,
                    "handler block was never closed",
                ));
            }
            self.advance(); // consume CloseHandler

            match method {
                HttpMethod::Get => {
                    if !matches!(self.current_token(), HaplTokenType::CloseEndpointGet) {
                        return Err(parser_err(
                            ErrorCode::TagNotClosed,
                            "endpoint-get was never closed",
                        ));
                    }
                }
                HttpMethod::Post => {
                    if !matches!(self.current_token(), HaplTokenType::CloseEndpointPost) {
                        return Err(parser_err(
                            ErrorCode::TagNotClosed,
                            "endpoint-post was never closed",
                        ));
                    }
                }
            }
            self.advance(); // consume CloseEndpointGet/Post

            endpoints.push(Endpoint { method, path, handler });
        }

        if self.is_at_end() {
            return Err(parser_err(ErrorCode::TagNotClosed, "server block was never closed")
                .with_hint(
                    "add a matching closing tag for the <div class=\"server\"> block",
                ));
        }
        self.advance(); // consume CloseServer

        Ok(Expr::ServerDeclaration { port, endpoints })
    }
}
