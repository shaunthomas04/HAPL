use super::*;
use crate::ast::{Expr, HttpMethod, LiteralValue};
use crate::error::{ErrorCode, runtime_err};
use control_flow::{ControlFlow, bubble};
use helpers::{json_to_literal, literal_to_json, TypeName};
use indexmap::IndexMap;

impl Interpreter {
    pub(super) fn eval_http_get(&mut self, expr: &Expr) -> ControlFlow {
        let (name, url) = match expr {
            Expr::HttpGet { name, url } => (name, url),
            _ => unreachable!(),
        };

        let url_val = bubble!(self.eval_value(url));
        let url_str = match url_val {
            LiteralValue::String(s) => s,
            other => return ControlFlow::Error(
                runtime_err(
                    ErrorCode::HttpRequestFailed,
                    format!(
                        "http-get url must be a string, got {:?}",
                        other.type_name()
                    ),
                )
                .with_hint("wrap the url in a string literal"),
            ),
        };

        let result_map = match reqwest::blocking::get(&url_str) {
            Ok(response) => match response.json::<serde_json::Value>() {
                Ok(json) => json_to_literal(json),
                Err(e) => {
                    let mut entries = IndexMap::new();
                    entries.insert(
                        "error".to_string(),
                        LiteralValue::String(format!("invalid JSON response: {}", e)),
                    );
                    LiteralValue::Map { entries }
                }
            },
            Err(e) => {
                let mut entries = IndexMap::new();
                entries.insert(
                    "error".to_string(),
                    LiteralValue::String(format!("request failed: {}", e)),
                );
                LiteralValue::Map { entries }
            }
        };

        self.current_scope().insert(name.clone(), result_map.clone());
        ControlFlow::Value(result_map)
    }

    pub(super) fn eval_http_post(&mut self, expr: &Expr) -> ControlFlow {
        let (name, url, body) = match expr {
            Expr::HttpPost { name, url, body } => (name, url, body),
            _ => unreachable!(),
        };

        let url_val = bubble!(self.eval_value(url));
        let url_str = match url_val {
            LiteralValue::String(s) => s,
            other => return ControlFlow::Error(
                runtime_err(
                    ErrorCode::HttpRequestFailed,
                    format!(
                        "http-post url must be a string, got {:?}",
                        other.type_name()
                    ),
                )
                .with_hint("wrap the url in a string literal"),
            ),
        };

        let body_val = bubble!(self.eval_value(body));
        let body_json = match literal_to_json(&body_val) {
            Some(json) => json,
            None => return ControlFlow::Error(
                runtime_err(ErrorCode::HttpRequestFailed, "http-post body must be a map")
                    .with_hint("pass a map variable as the body"),
            ),
        };

        let client = reqwest::blocking::Client::new();
        let result_map = match client.post(&url_str).json(&body_json).send() {
            Ok(response) => match response.json::<serde_json::Value>() {
                Ok(json) => json_to_literal(json),
                Err(e) => {
                    let mut entries = IndexMap::new();
                    entries.insert(
                        "error".to_string(),
                        LiteralValue::String(format!("invalid JSON response: {}", e)),
                    );
                    LiteralValue::Map { entries }
                }
            },
            Err(e) => {
                let mut entries = IndexMap::new();
                entries.insert(
                    "error".to_string(),
                    LiteralValue::String(format!("request failed: {}", e)),
                );
                LiteralValue::Map { entries }
            }
        };

        self.current_scope().insert(name.clone(), result_map.clone());
        ControlFlow::Value(result_map)
    }

    pub(super) fn eval_server(&mut self, expr: &Expr) -> ControlFlow {
        let (port, endpoints) = match expr {
            Expr::ServerDeclaration { port, endpoints } => (*port, endpoints),
            _ => unreachable!(),
        };

        let addr = format!("0.0.0.0:{}", port);
        let server = match tiny_http::Server::http(&addr) {
            Ok(s) => s,
            Err(e) => return ControlFlow::Error(
                runtime_err(
                    ErrorCode::HttpRequestFailed,
                    format!("failed to start server on port {}: {}", port, e),
                )
                .with_hint("check that the port is not already in use"),
            ),
        };

        println!("HAPL server running on http://localhost:{}", port);

        'requests: for mut request in server.incoming_requests() {
            let method = request.method().clone();
            let url    = request.url().to_string();

            let (path, query_string) = match url.find('?') {
                Some(i) => (&url[..i], &url[i + 1..]),
                None    => (url.as_str(), ""),
            };

            // Parse query params
            let mut params_entries = IndexMap::new();
            for pair in query_string.split('&') {
                if pair.is_empty() { continue; }
                let mut parts = pair.splitn(2, '=');
                let key = parts.next().unwrap_or("").to_string();
                let val = parts.next().unwrap_or("").to_string();
                params_entries.insert(key, LiteralValue::String(val));
            }
            let params_map = LiteralValue::Map { entries: params_entries };

            // Match endpoint
            let matched = endpoints.iter().find(|e| {
                e.path == path
                    && match (&method, &e.method) {
                        (tiny_http::Method::Get,  HttpMethod::Get)  => true,
                        (tiny_http::Method::Post, HttpMethod::Post) => true,
                        _ => false,
                    }
            });

            let endpoint = match matched {
                Some(e) => e,
                None => {
                    let _ = request.respond(
                        tiny_http::Response::from_string("404 not found")
                            .with_status_code(404),
                    );
                    continue 'requests;
                }
            };

            // Parse POST body
            let mut body_entries = IndexMap::new();
            if matches!(method, tiny_http::Method::Post) {
                let mut body_str = String::new();
                if request.as_reader().read_to_string(&mut body_str).is_ok() {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_str) {
                        if let LiteralValue::Map { entries } = json_to_literal(json) {
                            body_entries = entries;
                        }
                    }
                }
            }
            let body_map = LiteralValue::Map { entries: body_entries };

            // Run handler
            self.push_scope();
            self.current_scope().insert("params".to_string(), params_map);
            self.current_scope().insert("body".to_string(), body_map);

            let mut response_value = LiteralValue::String(String::new());

            for stmt in &endpoint.handler {
                match self.eval(stmt) {
                    ControlFlow::Value(v)  => { response_value = v; }
                    ControlFlow::Return(v) => { response_value = v; break; }
                    ControlFlow::Error(e)  => {
                        self.pop_scope();
                        let _ = request.respond(
                            tiny_http::Response::from_string(format!(
                                "Internal error: {}",
                                e
                            ))
                            .with_status_code(500),
                        );
                        continue 'requests;
                    }
                }
                if matches!(stmt, Expr::Respond { .. }) { break; }
            }

            self.pop_scope();

            // Serialize response
            let (body, content_type) = match &response_value {
                LiteralValue::Map { .. } => {
                    let json = literal_to_json(&response_value)
                        .map(|j| j.to_string())
                        .unwrap_or_else(|| "{}".to_string());
                    (json, "application/json")
                }
                LiteralValue::String(s)  => (s.clone(), "text/plain"),
                LiteralValue::Integer(n) => (n.to_string(), "text/plain"),
                LiteralValue::Double(f)  => (f.to_string(), "text/plain"),
                LiteralValue::Boolean(b) => (b.to_string(), "text/plain"),
                LiteralValue::List { .. } => ("[]".to_string(), "application/json"),
            };

            let _ = request.respond(
                tiny_http::Response::from_string(body).with_header(
                    tiny_http::Header::from_bytes("Content-Type", content_type).unwrap(),
                ),
            );
        }

        ControlFlow::Value(LiteralValue::Boolean(true))
    }
}
