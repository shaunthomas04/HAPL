mod tag_type;
mod token;
mod helpers;
mod walk_operators;
mod walk_vars;
mod walk_control;
mod walk_functions;
mod walk_collections;
mod walk_io;

// Re-export everything sub-modules need, and everything callers need.
pub use tag_type::{LexerTagType, LoopType};
pub use token::{HaplToken, HaplTokenType};
pub use helpers::{tag_snippet, parse_static_type, operator_to_str};

use crate::{HtmlTag};
use crate::config::HaplConfig;
use crate::error::{ErrorCode, HaplError, lexer_err};

pub struct HaplLexer {
    pub(super) tokens: Vec<HaplToken>,
    pub(super) config: Option<HaplConfig>,
}

impl HaplLexer {
    pub fn new() -> Self {
        Self { tokens: Vec::new(), config: None }
    }

    pub fn with_config(config: HaplConfig) -> Self {
        Self { tokens: Vec::new(), config: Some(config) }
    }

    /// Lex the tag tree, returning `Err(HaplError)` on the first error.
    pub fn lex(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        self.walk(tag)
    }

    pub fn tokens(&self) -> &[HaplToken] {
        &self.tokens
    }

    /// Resolve a class name through the optional config alias map.
    pub(super) fn resolve<'a>(&'a self, class: &'a str) -> &'a str {
        self.config.as_ref()
            .map(|c| c.resolve(class))
            .unwrap_or(class)
    }

    // --------------------------------------------------
    // Top-level tag dispatcher
    // --------------------------------------------------
    pub(super) fn walk(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        match tag.tag_type.as_str() {
            "div"  => self.walk_div(tag),
            "p"    => self.walk_print(tag),
            "var"  => self.walk_var(tag),
            "span" => self.walk_span(tag),
            "html" | "head" | "body" | "title" | "meta" | "link" => {
                self.tokens.push(HaplToken::open_html_tag(tag.tag_type.clone()));
                for child in &tag.child_tags {
                    self.walk(child)?;
                }
                self.tokens.push(HaplToken::close_html_tag(tag.tag_type.clone()));
                Ok(())
            }
            other => Err(
                lexer_err(
                    ErrorCode::UnknownTag,
                    format!("unrecognized tag <{}>", other),
                )
                .with_tag(
                    tag_snippet(tag),
                    format!("'{}' is not a valid HAPL tag", other),
                )
                .with_hint("supported tags: div, p, var, span"),
            ),
        }
    }

    // --------------------------------------------------
    // <div> dispatcher — routes to the correct walk_* fn
    // --------------------------------------------------
    pub(super) fn walk_div(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        let raw_class = tag.class.as_ref().ok_or_else(|| {
            lexer_err(ErrorCode::MissingClass, "missing class attribute on <div> tag")
                .with_tag(tag_snippet(tag), "every <div> must have a class")
                .with_hint("example: <div class=\"+\"> ... </div>")
        })?.clone();

        let class = self.resolve(&raw_class).to_string();

        // Arithmetic operators
        if let Some((open, close)) = self.try_arithmetic_tokens(&class) {
            self.tokens.push(open);
            for child in &tag.child_tags { self.walk(child)?; }
            self.tokens.push(close);
            return Ok(());
        }

        // Boolean operators
        if let Some((open, close)) = self.try_boolean_tokens(&class) {
            self.tokens.push(open);
            for child in &tag.child_tags { self.walk(child)?; }
            self.tokens.push(close);
            return Ok(());
        }

        // Comparison operators
        if let Some((open, close)) = self.try_comparison_tokens(&class) {
            self.tokens.push(open);
            for child in &tag.child_tags { self.walk(child)?; }
            self.tokens.push(close);
            return Ok(());
        }

        if class == "conditional" { return self.walk_conditional(tag); }
        if class == "while"       { return self.walk_while(tag); }
        if class == "for"         { return self.walk_for(tag); }

        if class == "return" {
            self.tokens.push(HaplToken::open_return());
            for child in &tag.child_tags { self.walk(child)?; }
            self.tokens.push(HaplToken::close_return());
            return Ok(());
        }

        // List declaration
        if let Some(elem_type) = self.parse_list_class(&class) {
            let name = tag.id.clone().ok_or_else(|| {
                lexer_err(
                    ErrorCode::MissingId,
                    format!("list declaration '{}' is missing an id attribute", class),
                )
                .with_tag(tag_snippet(tag), "id attribute required to name the list")
                .with_hint(format!("example: <div class=\"{}\" id=\"myList\">", class))
            })?;
            return self.walk_list_decl(tag, name, elem_type);
        }

        // List operations
        if class == "index"        { return self.walk_list_access(tag); }
        if class == "index-assign" { return self.walk_list_assign(tag); }
        if class == "push"         { return self.walk_list_push(tag); }
        if class == "pop"          { return self.walk_list_pop(tag); }

        // Map declaration
        if class == "map" {
            let name = tag.id.as_deref().unwrap_or("").to_string();
            return self.walk_map_dec(tag, name);
        }

        // Map operations
        if class == "map-get"      { return self.walk_map_get(tag); }
        if class == "map-contains" { return self.walk_map_contains(tag); }
        if class == "map-set"      { return self.walk_map_set(tag); }
        if class == "map-remove"   { return self.walk_map_remove(tag); }

        // Server
        if class == "server"  { return self.walk_server(tag); }
        if class == "respond" { return self.walk_respond(tag); }

        // Function declaration
        if let Some(return_type) = self.parse_function_class(&class) {
            let name = tag.id.clone().ok_or_else(|| {
                lexer_err(
                    ErrorCode::MissingId,
                    format!("function declaration '{}' is missing an id attribute", class),
                )
                .with_tag(tag_snippet(tag), "id attribute required to name the function")
                .with_hint(format!("example: <div class=\"{}\" id=\"myFunc\">", class))
            })?;
            return self.walk_function_decl(tag, name, return_type);
        }

        if class == "length"    { return self.walk_length(tag); }
        if class == "http-get"  { return self.walk_http_get(tag); }
        if class == "http-post" { return self.walk_http_post(tag); }

        if class == "input" {
            self.tokens.push(HaplToken::open_input());
            self.tokens.push(HaplToken::close_input());
            return Ok(());
        }

        // Function call (class = function name, no id)
        if tag.id.is_none() {
            return self.walk_function_call(tag, class);
        }

        Err(
            lexer_err(
                ErrorCode::UnknownTag,
                format!("unrecognized <div> usage: class='{}'", class),
            )
            .with_tag(
                tag_snippet(tag),
                format!("'{}' is not a known keyword or declared function", class),
            )
            .with_hint(
                "valid div classes: +, -, *, /, &&, ||, !, equal, not_equal, less, \
                 less_equal, greater, greater_equal, conditional, while, for, return, \
                 <type>-function",
            ),
        )
    }

    // --------------------------------------------------
    // Debug printer
    // --------------------------------------------------
    pub fn print(&self) {
        for token in &self.tokens {
            match &token.token_type {
                HaplTokenType::OpenOperator { name } => {
                    println!("OpenOperator({}) -> {:?}", operator_to_str(*name), token.value);
                }
                HaplTokenType::CloseOperator { name } => {
                    println!("CloseOperator({}) -> {:?}", operator_to_str(*name), token.value);
                }
                HaplTokenType::OpenHtmlTag { name } => println!("OpenHtmlTag({}) -> {:?}", name, token.value),
                HaplTokenType::CloseHtmlTag { name } => println!("CloseHtmlTag({}) -> {:?}", name, token.value),
                HaplTokenType::Literal(lit) => println!("Literal({:?}) -> {:?}", lit, token.value),
                HaplTokenType::OpenVarDec { var_type, name } => {
                    println!("OpenVarDec({:?}, {}) -> {:?}", var_type, name, token.value);
                }
                HaplTokenType::CloseVarDec { var_type, name } => {
                    println!("CloseVarDec({:?}, {}) -> {:?}", var_type, name, token.value);
                }
                HaplTokenType::OpenVarRef { name }    => println!("OpenVarRef({}) -> {:?}", name, token.value),
                HaplTokenType::CloseVarRef { name }   => println!("CloseVarRef({}) -> {:?}", name, token.value),
                HaplTokenType::OpenVarAssign { name } => println!("OpenVarAssign({}) -> {:?}", name, token.value),
                HaplTokenType::CloseVarAssign { name }=> println!("CloseVarAssign({}) -> {:?}", name, token.value),
                HaplTokenType::OpenPrint              => println!("OpenPrint -> {:?}", token.value),
                HaplTokenType::ClosePrint             => println!("ClosePrint -> {:?}", token.value),
                HaplTokenType::OpenConditional        => println!("OpenConditional -> {:?}", token.value),
                HaplTokenType::CloseConditional       => println!("CloseConditional -> {:?}", token.value),
                HaplTokenType::OpenIf                 => println!("OpenIf -> {:?}", token.value),
                HaplTokenType::CloseIf                => println!("CloseIf -> {:?}", token.value),
                HaplTokenType::OpenElif               => println!("OpenElif -> {:?}", token.value),
                HaplTokenType::CloseElif              => println!("CloseElif -> {:?}", token.value),
                HaplTokenType::OpenElse               => println!("OpenElse -> {:?}", token.value),
                HaplTokenType::CloseElse              => println!("CloseElse -> {:?}", token.value),
                HaplTokenType::OpenLoop { loop_type } => {
                    println!("OpenLoop({:?}) -> {:?}", loop_type, token.value);
                }
                HaplTokenType::CloseLoop { loop_type } => {
                    println!("CloseLoop({:?}) -> {:?}", loop_type, token.value);
                }
                HaplTokenType::OpenLoopCondition  => println!("OpenLoopCondition -> {:?}", token.value),
                HaplTokenType::CloseLoopCondition => println!("CloseLoopCondition -> {:?}", token.value),
                HaplTokenType::OpenLoopBody       => println!("OpenLoopBody -> {:?}", token.value),
                HaplTokenType::CloseLoopBody      => println!("CloseLoopBody -> {:?}", token.value),
                HaplTokenType::OpenLoopIterator   => println!("OpenLoopIterator -> {:?}", token.value),
                HaplTokenType::CloseLoopIterator  => println!("CloseLoopIterator -> {:?}", token.value),
                HaplTokenType::OpenLoopIncrement  => println!("OpenLoopIncrement -> {:?}", token.value),
                HaplTokenType::CloseLoopIncrement => println!("CloseLoopIncrement -> {:?}", token.value),
                HaplTokenType::OpenFunction { name, return_type } => {
                    println!("OpenFunction({}, {:?}) -> {:?}", name, return_type, token.value);
                }
                HaplTokenType::CloseFunction { name } => println!("CloseFunction({}) -> {:?}", name, token.value),
                HaplTokenType::OpenParams       => println!("OpenParams -> {:?}", token.value),
                HaplTokenType::CloseParams      => println!("CloseParams -> {:?}", token.value),
                HaplTokenType::OpenParam { name, param_type } => {
                    println!("OpenParam({}, {:?}) -> {:?}", name, param_type, token.value);
                }
                HaplTokenType::CloseParam { name }    => println!("CloseParam({}) -> {:?}", name, token.value),
                HaplTokenType::OpenFunctionBody       => println!("OpenFunctionBody -> {:?}", token.value),
                HaplTokenType::CloseFunctionBody      => println!("CloseFunctionBody -> {:?}", token.value),
                HaplTokenType::OpenFunctionCall { name } => {
                    println!("OpenFunctionCall({}) -> {:?}", name, token.value);
                }
                HaplTokenType::CloseFunctionCall { name } => {
                    println!("CloseFunctionCall({}) -> {:?}", name, token.value);
                }
                HaplTokenType::OpenReturn  => println!("OpenReturn -> {:?}", token.value),
                HaplTokenType::CloseReturn => println!("CloseReturn -> {:?}", token.value),
                HaplTokenType::OpenArgs    => println!("OpenArgs -> {:?}", token.value),
                HaplTokenType::CloseArgs   => println!("CloseArgs -> {:?}", token.value),
                HaplTokenType::OpenListDec { elem_type, name } => {
                    println!("OpenListDec({:?}, {}) -> {:?}", elem_type, name, token.value);
                }
                HaplTokenType::CloseListDec { name }    => println!("CloseListDec({}) -> {:?}", name, token.value),
                HaplTokenType::OpenListAccess           => println!("OpenListAccess -> {:?}", token.value),
                HaplTokenType::CloseListAccess          => println!("CloseListAccess -> {:?}", token.value),
                HaplTokenType::OpenListAssign { name }  => println!("OpenListAssign({}) -> {:?}", name, token.value),
                HaplTokenType::CloseListAssign { name } => println!("CloseListAssign({}) -> {:?}", name, token.value),
                HaplTokenType::OpenListPush { name }    => println!("OpenListPush({}) -> {:?}", name, token.value),
                HaplTokenType::CloseListPush { name }   => println!("CloseListPush({}) -> {:?}", name, token.value),
                HaplTokenType::OpenListPop { name }     => println!("OpenListPop({}) -> {:?}", name, token.value),
                HaplTokenType::CloseListPop { name }    => println!("CloseListPop({}) -> {:?}", name, token.value),
                HaplTokenType::OpenMapDec { name }      => println!("OpenMapDec({}) -> {:?}", name, token.value),
                HaplTokenType::CloseMapDec { name }     => println!("CloseMapDec({}) -> {:?}", name, token.value),
                HaplTokenType::OpenMapGet               => println!("OpenMapGet -> {:?}", token.value),
                HaplTokenType::CloseMapGet              => println!("CloseMapGet -> {:?}", token.value),
                HaplTokenType::OpenMapSet { name }      => println!("OpenMapSet({}) -> {:?}", name, token.value),
                HaplTokenType::CloseMapSet { name }     => println!("CloseMapSet({}) -> {:?}", name, token.value),
                HaplTokenType::OpenMapRemove { name }   => println!("OpenMapRemove({}) -> {:?}", name, token.value),
                HaplTokenType::CloseMapRemove { name }  => println!("CloseMapRemove({}) -> {:?}", name, token.value),
                HaplTokenType::OpenMapContains          => println!("OpenMapContains -> {:?}", token.value),
                HaplTokenType::CloseMapContains         => println!("CloseMapContains -> {:?}", token.value),
                HaplTokenType::OpenMapKey               => println!("OpenMapKey -> {:?}", token.value),
                HaplTokenType::CloseMapKey              => println!("CloseMapKey -> {:?}", token.value),
                HaplTokenType::OpenMapValue             => println!("OpenMapValue -> {:?}", token.value),
                HaplTokenType::CloseMapValue            => println!("CloseMapValue -> {:?}", token.value),
                HaplTokenType::OpenMapEntry { key }     => println!("OpenMapEntry({}) -> {:?}", key, token.value),
                HaplTokenType::CloseMapEntry { key }    => println!("CloseMapEntry({}) -> {:?}", key, token.value),
                HaplTokenType::OpenLength               => println!("OpenLength -> {:?}", token.value),
                HaplTokenType::CloseLength              => println!("CloseLength -> {:?}", token.value),
                HaplTokenType::OpenHttpGet { name }     => println!("OpenHttpGet({}) -> {:?}", name, token.value),
                HaplTokenType::CloseHttpGet { name }    => println!("CloseHttpGet({}) -> {:?}", name, token.value),
                HaplTokenType::OpenHttpPost { name }    => println!("OpenHttpPost({}) -> {:?}", name, token.value),
                HaplTokenType::CloseHttpPost { name }   => println!("CloseHttpPost({}) -> {:?}", name, token.value),
                HaplTokenType::OpenHttpUrl              => println!("OpenHttpUrl -> {:?}", token.value),
                HaplTokenType::CloseHttpUrl             => println!("CloseHttpUrl -> {:?}", token.value),
                HaplTokenType::OpenHttpBody             => println!("OpenHttpBody -> {:?}", token.value),
                HaplTokenType::CloseHttpBody            => println!("CloseHttpBody -> {:?}", token.value),
                HaplTokenType::OpenInput                => println!("OpenInput -> {:?}", token.value),
                HaplTokenType::CloseInput               => println!("CloseInput -> {:?}", token.value),
                HaplTokenType::OpenServer { port }      => println!("OpenServer({}) -> {:?}", port, token.value),
                HaplTokenType::CloseServer              => println!("CloseServer -> {:?}", token.value),
                HaplTokenType::OpenEndpointGet { path } => println!("OpenEndpointGet({}) -> {:?}", path, token.value),
                HaplTokenType::CloseEndpointGet         => println!("CloseEndpointGet -> {:?}", token.value),
                HaplTokenType::OpenEndpointPost { path }=> println!("OpenEndpointPost({}) -> {:?}", path, token.value),
                HaplTokenType::CloseEndpointPost        => println!("CloseEndpointPost -> {:?}", token.value),
                HaplTokenType::OpenHandler              => println!("OpenHandler -> {:?}", token.value),
                HaplTokenType::CloseHandler             => println!("CloseHandler -> {:?}", token.value),
                HaplTokenType::OpenRespond              => println!("OpenRespond -> {:?}", token.value),
                HaplTokenType::CloseRespond             => println!("CloseRespond -> {:?}", token.value),
            }
        }
    }
}