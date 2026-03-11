use crate::{HtmlTag};
use crate::ast::{LiteralValue, StaticType};
use crate::error::{ErrorCode, HaplError, lexer_err};
use crate::config::{HaplConfig};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LexerTagType {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    And,
    Or,
    Not,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, Clone)]
pub enum HaplTokenType {
    OpenOperator { name: LexerTagType },
    CloseOperator { name: LexerTagType },
    OpenHtmlTag { name: String },
    CloseHtmlTag { name: String },
    Literal(LiteralValue),
    OpenVarDec { var_type: StaticType, name: String },
    CloseVarDec { var_type: StaticType, name: String },
    OpenVarRef { name: String },
    CloseVarRef { name: String },
    OpenVarAssign { name: String },
    CloseVarAssign { name: String },
    OpenPrint,
    ClosePrint,

    // Conditional logic
    OpenConditional,
    CloseConditional,
    OpenIf,
    CloseIf,
    OpenElif,
    CloseElif,
    OpenElse,
    CloseElse,

    // Loops
    OpenLoop { loop_type: LoopType },
    CloseLoop { loop_type: LoopType },
    OpenLoopCondition,
    CloseLoopCondition,
    OpenLoopBody,
    CloseLoopBody,
    OpenLoopIterator,
    CloseLoopIterator,
    OpenLoopIncrement,
    CloseLoopIncrement,

    // Functions
    OpenFunction { name: String, return_type: StaticType },
    CloseFunction { name: String },
    OpenParams,
    CloseParams,
    OpenParam { name: String, param_type: StaticType },
    CloseParam { name: String },
    OpenFunctionBody,
    CloseFunctionBody,
    OpenFunctionCall { name: String },
    CloseFunctionCall { name: String },
    OpenReturn,
    CloseReturn,
    OpenArgs,
    CloseArgs,

    // Lists
    OpenListDec { elem_type: StaticType, name: String },
    CloseListDec { name: String },
    OpenListAccess,
    CloseListAccess,
    OpenListAssign { name: String },
    CloseListAssign { name: String },
    OpenListPush { name: String },
    CloseListPush { name: String },
    OpenListPop { name: String },
    CloseListPop { name: String },

    // Maps
    OpenMapDec { name: String },
    CloseMapDec { name: String },
    OpenMapGet,
    CloseMapGet,
    OpenMapSet { name: String },
    CloseMapSet { name: String },
    OpenMapRemove { name: String },
    CloseMapRemove { name: String },
    OpenMapContains,
    CloseMapContains,
    OpenMapKey,
    CloseMapKey,
    OpenMapValue,
    CloseMapValue,
    OpenMapEntry { key: String },
    CloseMapEntry { key: String },

    OpenLength,
    CloseLength,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopType {
    While,
    For,
}

#[derive(Debug, Clone)]
pub struct HaplToken {
    pub token_type: HaplTokenType,
    pub value: Option<String>,
}

impl HaplToken {
    fn new(token_type: HaplTokenType, value: Option<String>) -> Self {
        Self { token_type, value }
    }

    fn open_operator(name: LexerTagType) -> Self {
        Self::new(
            HaplTokenType::OpenOperator { name },
            Some(operator_to_str(name).to_string()),
        )
    }

    fn close_operator(name: LexerTagType) -> Self {
        Self::new(
            HaplTokenType::CloseOperator { name },
            Some(operator_to_str(name).to_string()),
        )
    }

    fn open_html_tag(name: String) -> Self {
        Self::new(HaplTokenType::OpenHtmlTag { name: name.clone() }, Some(name))
    }

    fn close_html_tag(name: String) -> Self {
        Self::new(HaplTokenType::CloseHtmlTag { name: name.clone() }, Some(name))
    }

    fn string(content: String) -> Self {
        Self::new(
            HaplTokenType::Literal(LiteralValue::String(content.clone())),
            Some(content),
        )
    }

    fn double(double_value: f64) -> Self {
        Self::new(
            HaplTokenType::Literal(LiteralValue::Double(double_value)),
            Some(double_value.to_string()),
        )
    }

    fn integer(integer_value: i64) -> Self {
        Self::new(
            HaplTokenType::Literal(LiteralValue::Integer(integer_value)),
            Some(integer_value.to_string()),
        )
    }

    fn boolean(boolean_value: bool) -> Self {
        Self::new(
            HaplTokenType::Literal(LiteralValue::Boolean(boolean_value)),
            Some(boolean_value.to_string()),
        )
    }

    fn open_print() -> Self {
        Self::new(HaplTokenType::OpenPrint, Some("print".to_string()))
    }

    fn close_print() -> Self {
        Self::new(HaplTokenType::ClosePrint, Some("print".to_string()))
    }

    pub fn open_conditional() -> Self {
        Self::new(HaplTokenType::OpenConditional, Some("conditional".to_string()))
    }

    pub fn close_conditional() -> Self {
        Self::new(HaplTokenType::CloseConditional, Some("conditional".to_string()))
    }

    pub fn open_if() -> Self {
        Self::new(HaplTokenType::OpenIf, Some("if".to_string()))
    }

    pub fn close_if() -> Self {
        Self::new(HaplTokenType::CloseIf, Some("if".to_string()))
    }

    pub fn open_elif() -> Self {
        Self::new(HaplTokenType::OpenElif, Some("elif".to_string()))
    }

    pub fn close_elif() -> Self {
        Self::new(HaplTokenType::CloseElif, Some("elif".to_string()))
    }

    pub fn open_else() -> Self {
        Self::new(HaplTokenType::OpenElse, Some("else".to_string()))
    }

    pub fn close_else() -> Self {
        Self::new(HaplTokenType::CloseElse, Some("else".to_string()))
    }

    pub fn open_loop(loop_type: LoopType) -> Self {
        Self::new(
            HaplTokenType::OpenLoop { loop_type },
            Some(format!("open_{:?}", loop_type)),
        )
    }

    pub fn close_loop(loop_type: LoopType) -> Self {
        Self::new(
            HaplTokenType::CloseLoop { loop_type },
            Some(format!("close_{:?}", loop_type)),
        )
    }

    pub fn open_loop_condition() -> Self {
        Self::new(HaplTokenType::OpenLoopCondition, Some("open_loop_condition".to_string()))
    }

    pub fn close_loop_condition() -> Self {
        Self::new(HaplTokenType::CloseLoopCondition, Some("close_loop_condition".to_string()))
    }

    pub fn open_loop_body() -> Self {
        Self::new(HaplTokenType::OpenLoopBody, Some("open_loop_body".to_string()))
    }

    pub fn close_loop_body() -> Self {
        Self::new(HaplTokenType::CloseLoopBody, Some("close_loop_body".to_string()))
    }

    pub fn open_loop_iterator() -> Self {
        Self::new(HaplTokenType::OpenLoopIterator, Some("open_loop_iterator".to_string()))
    }

    pub fn close_loop_iterator() -> Self {
        Self::new(HaplTokenType::CloseLoopIterator, Some("close_loop_iterator".to_string()))
    }

    pub fn open_loop_increment() -> Self {
        Self::new(HaplTokenType::OpenLoopIncrement, Some("open_loop_increment".to_string()))
    }

    pub fn close_loop_increment() -> Self {
        Self::new(HaplTokenType::CloseLoopIncrement, Some("close_loop_increment".to_string()))
    }

    pub fn open_function(name: String, return_type: StaticType) -> Self {
        Self::new(
            HaplTokenType::OpenFunction { name: name.clone(), return_type },
            Some(name),
        )
    }

    pub fn close_function(name: String) -> Self {
        Self::new(HaplTokenType::CloseFunction { name: name.clone() }, Some(name))
    }

    pub fn open_params() -> Self {
        Self::new(HaplTokenType::OpenParams, Some("params".to_string()))
    }

    pub fn close_params() -> Self {
        Self::new(HaplTokenType::CloseParams, Some("params".to_string()))
    }

    pub fn open_param(name: String, param_type: StaticType) -> Self {
        Self::new(
            HaplTokenType::OpenParam { name: name.clone(), param_type: param_type.clone() },
            Some(format!("{:?} {}", param_type, name)),
        )
    }

    pub fn close_param(name: String) -> Self {
        Self::new(HaplTokenType::CloseParam { name: name.clone() }, Some(name))
    }

    pub fn open_function_body() -> Self {
        Self::new(HaplTokenType::OpenFunctionBody, Some("function_body".to_string()))
    }

    pub fn close_function_body() -> Self {
        Self::new(HaplTokenType::CloseFunctionBody, Some("function_body".to_string()))
    }

    pub fn open_function_call(name: String) -> Self {
        Self::new(HaplTokenType::OpenFunctionCall { name: name.clone() }, Some(name))
    }

    pub fn close_function_call(name: String) -> Self {
        Self::new(HaplTokenType::CloseFunctionCall { name: name.clone() }, Some(name))
    }

    pub fn open_return() -> Self {
        Self::new(HaplTokenType::OpenReturn, Some("return".to_string()))
    }

    pub fn close_return() -> Self {
        Self::new(HaplTokenType::CloseReturn, Some("return".to_string()))
    }

    pub fn open_args() -> Self {
        Self::new(HaplTokenType::OpenArgs, Some("args".to_string()))
    }

    pub fn close_args() -> Self {
        Self::new(HaplTokenType::CloseArgs, Some("args".to_string()))
    }

    pub fn open_list_dec(elem_type: StaticType, name: String) -> Self {
        Self::new(HaplTokenType::OpenListDec { elem_type, name: name.clone() }, Some(name))
    }
    pub fn close_list_dec(name: String) -> Self {
        Self::new(HaplTokenType::CloseListDec { name: name.clone() }, Some(name))
    }
    pub fn open_list_access() -> Self {
        Self::new(HaplTokenType::OpenListAccess, Some("index".to_string()))
    }
    pub fn close_list_access() -> Self {
        Self::new(HaplTokenType::CloseListAccess, Some("index".to_string()))
    }
    pub fn open_list_assign(name: String) -> Self {
        Self::new(HaplTokenType::OpenListAssign { name: name.clone() }, Some(name))
    }
    pub fn close_list_assign(name: String) -> Self {
        Self::new(HaplTokenType::CloseListAssign { name: name.clone() }, Some(name))
    }
    pub fn open_list_push(name: String) -> Self {
        Self::new(HaplTokenType::OpenListPush { name: name.clone() }, Some(name))
    }
    pub fn close_list_push(name: String) -> Self {
        Self::new(HaplTokenType::CloseListPush { name: name.clone() }, Some(name))
    }
    pub fn open_list_pop(name: String) -> Self {
        Self::new(HaplTokenType::OpenListPop { name: name.clone() }, Some(name))
    }
    pub fn close_list_pop(name: String) -> Self {
        Self::new(HaplTokenType::CloseListPop { name: name.clone() }, Some(name))
    }

    pub fn open_map_dec(name: String) -> Self {
        Self::new(HaplTokenType::OpenMapDec { name: name.clone() }, Some(name))
    }
    pub fn close_map_dec(name: String) -> Self {
        Self::new(HaplTokenType::CloseMapDec { name: name.clone() }, Some(name))
    }
    pub fn open_map_get() -> Self {
        Self::new(HaplTokenType::OpenMapGet, Some("map-get".to_string()))
    }
    pub fn close_map_get() -> Self {
        Self::new(HaplTokenType::CloseMapGet, Some("map-get".to_string()))
    }
    pub fn open_map_set(name: String) -> Self {
        Self::new(HaplTokenType::OpenMapSet { name: name.clone() }, Some(name))
    }
    pub fn close_map_set(name: String) -> Self {
        Self::new(HaplTokenType::CloseMapSet { name: name.clone() }, Some(name))
    }
    pub fn open_map_remove(name: String) -> Self {
        Self::new(HaplTokenType::OpenMapRemove { name: name.clone() }, Some(name))
    }
    pub fn close_map_remove(name: String) -> Self {
        Self::new(HaplTokenType::CloseMapRemove { name: name.clone() }, Some(name))
    }
    pub fn open_map_contains() -> Self {
        Self::new(HaplTokenType::OpenMapContains, Some("map-contains".to_string()))
    }
    pub fn close_map_contains() -> Self {
        Self::new(HaplTokenType::CloseMapContains, Some("map-contains".to_string()))
    }
    pub fn open_map_key() -> Self {
        Self::new(HaplTokenType::OpenMapKey, Some("key".to_string()))
    }
    pub fn close_map_key() -> Self {
        Self::new(HaplTokenType::CloseMapKey, Some("key".to_string()))
    }
    pub fn open_map_value() -> Self {
        Self::new(HaplTokenType::OpenMapValue, Some("value".to_string()))
    }
    pub fn close_map_value() -> Self {
        Self::new(HaplTokenType::CloseMapValue, Some("value".to_string()))
    }
    pub fn open_map_entry(key: String) -> Self {
        Self::new(HaplTokenType::OpenMapEntry { key: key.clone() }, Some(key))
    }
    pub fn close_map_entry(key: String) -> Self {
        Self::new(HaplTokenType::CloseMapEntry { key: key.clone() }, Some(key))
    }

    pub fn open_length() -> Self {
        Self::new(HaplTokenType::OpenLength, Some("length".to_string()))
    }
    pub fn close_length() -> Self {
        Self::new(HaplTokenType::CloseLength, Some("length".to_string()))
    }
}

// ------------------------------------------------------------------
// Helper: render an HtmlTag as a short HTML snippet for error context
// e.g.  `<var class="foo" id="bar">`
// ------------------------------------------------------------------
fn tag_snippet(tag: &HtmlTag) -> String {
    let mut s = format!("<{}", tag.tag_type);
    if let Some(c) = &tag.class { s.push_str(&format!(" class=\"{}\"", c)); }
    if let Some(id) = &tag.id   { s.push_str(&format!(" id=\"{}\"", id));   }
    s.push('>');
    s
}

pub struct HaplLexer {
    tokens: Vec<HaplToken>,
    config: Option<HaplConfig>,

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

    // --------------------------------------------------
    // Top-level dispatcher
    // --------------------------------------------------
    fn walk(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
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
    // <p> → Print statement
    // --------------------------------------------------
    fn walk_print(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        self.tokens.push(HaplToken::open_print());
        for child in &tag.child_tags {
            self.walk(child)?;
        }
        self.tokens.push(HaplToken::close_print());
        Ok(())
    }

    // --------------------------------------------------
    // <var> → Variable declaration, reference, or assignment
    // --------------------------------------------------
    fn walk_var(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        let raw_class = tag.class.as_ref().ok_or_else(|| {
            lexer_err(
                ErrorCode::MissingClass,
                "missing class attribute on <var> tag",
            )
            .with_tag(
                tag_snippet(tag),
                "expected class=\"<type>\" for declaration or class=\"<name>\" for reference",
            )
            .with_hint("example: <var class=\"integer\" id=\"x\">10</var>")
        })?;

        let class = self.resolve(raw_class).to_string();
        
        
        // let class = tag.class.as_ref().ok_or_else(|| {
        //     lexer_err(
        //         ErrorCode::MissingClass,
        //         "missing class attribute on <var> tag",
        //     )
        //     .with_tag(
        //         tag_snippet(tag),
        //         "expected class=\"<type>\" for declaration or class=\"<name>\" for reference",
        //     )
        //     .with_hint("example: <var class=\"integer\" id=\"x\">10</var>")
        // })?;

        if let Some(id) = &tag.id {
            // Variable declaration: <var class="integer" id="x">10</var>
            let var_type = parse_static_type(&class).ok_or_else(|| {
                lexer_err(
                    ErrorCode::UnknownType,
                    format!("unknown type '{}' in variable declaration", class),
                )
                .with_tag(
                    tag_snippet(tag),
                    format!("'{}' is not a valid type", class),
                )
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

    // --------------------------------------------------
    // <span> → Literal value
    // --------------------------------------------------
   fn walk_span(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
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

    // --------------------------------------------------
    // <div> → All language constructs
    // --------------------------------------------------
    fn walk_div(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        let raw_class = tag.class.as_ref().ok_or_else(|| {
            lexer_err(
                ErrorCode::MissingClass,
                "missing class attribute on <div> tag",
            )
            .with_tag(tag_snippet(tag), "every <div> must have a class")
            .with_hint("example: <div class=\"+\"> ... </div>")
        })?.clone();

        let class = self.resolve(&raw_class).to_string();

        // let class = tag.class.as_ref().ok_or_else(|| {
        //     lexer_err(
        //         ErrorCode::MissingClass,
        //         "missing class attribute on <div> tag",
        //     )
        //     .with_tag(tag_snippet(tag), "every <div> must have a class")
        //     .with_hint("example: <div class=\"+\"> ... </div>")
        // })?.clone();

        // ---- Arithmetic operators ----
        if let Some((open, close)) = self.try_arithmetic_tokens(&class) {
            self.tokens.push(open);
            for child in &tag.child_tags { self.walk(child)?; }
            self.tokens.push(close);
            return Ok(());
        }

        // ---- Boolean operators ----
        if let Some((open, close)) = self.try_boolean_tokens(&class) {
            self.tokens.push(open);
            for child in &tag.child_tags { self.walk(child)?; }
            self.tokens.push(close);
            return Ok(());
        }

        // ---- Comparison operators ----
        if let Some((open, close)) = self.try_comparison_tokens(&class) {
            self.tokens.push(open);
            for child in &tag.child_tags { self.walk(child)?; }
            self.tokens.push(close);
            return Ok(());
        }

        // ---- Conditional ----
        if class == "conditional" {
            return self.walk_conditional(tag);
        }

        // ---- While loop ----
        if class == "while" {
            return self.walk_while(tag);
        }

        // ---- For loop ----
        if class == "for" {
            return self.walk_for(tag);
        }

        // ---- Return ----
        if class == "return" {
            self.tokens.push(HaplToken::open_return());
            for child in &tag.child_tags {
                self.walk(child)?;
            }
            self.tokens.push(HaplToken::close_return());
            return Ok(());
        }


        // ---- List declaration ----
        if let Some(elem_type) = self.parse_list_class(&class) {
            let name = tag.id.clone().ok_or_else(|| {
                lexer_err(ErrorCode::MissingId,
                    format!("list declaration '{}' is missing an id attribute", class))
                .with_tag(tag_snippet(tag), "id attribute required to name the list")
                .with_hint(format!("example: <div class=\"{}\" id=\"myList\">", class))
            })?;
            return self.walk_list_decl(tag, name, elem_type);
        }

        // ---- List operations ----
        if class == "index"        { return self.walk_list_access(tag); }
        if class == "index-assign" { return self.walk_list_assign(tag); }
        if class == "push"         { return self.walk_list_push(tag); }
        if class == "pop"          { return self.walk_list_pop(tag); }


        // ---- Map declaration ----
        if class == "map" {
            match tag.id.as_deref() {
                Some("") | None => {
                    // anonymous inline map (used as a value inside another map entry)
                    return self.walk_map_dec(tag, String::new());
                }
                Some(name) => {
                    return self.walk_map_dec(tag, name.to_string());
                }
            }
        }

        // ---- Map operations ----
        if class == "map-get"      { return self.walk_map_get(tag); }
        if class == "map-contains" { return self.walk_map_contains(tag); }
        if class == "map-set"      { return self.walk_map_set(tag); }
        if class == "map-remove"   { return self.walk_map_remove(tag); }


        // ---- Function declaration ----
        if let Some(return_type) = self.parse_function_class(&class) {
            let name = tag.id.clone().ok_or_else(|| {
                lexer_err(
                    ErrorCode::MissingId,
                    format!("function declaration '{}' is missing an id attribute", class),
                )
                .with_tag(
                    tag_snippet(tag),
                    "id attribute required to name the function",
                )
                .with_hint(format!(
                    "example: <div class=\"{}\" id=\"myFunc\">",
                    class
                ))
            })?;
            return self.walk_function_decl(tag, name, return_type);
        }

        if class == "length" {
            return self.walk_length(tag);
        }

        // ---- Function call ----
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
    // Conditional <div class="conditional">
    // --------------------------------------------------
    fn walk_conditional(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
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
                    return Err(
                        lexer_err(
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
                        .with_hint("valid children of conditional: if, elif, else"),
                    );
                }
            }
        }

        self.tokens.push(HaplToken::close_conditional());
        Ok(())
    }

    // --------------------------------------------------
    // While loop <div class="while">
    // --------------------------------------------------
    fn walk_while(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
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
                    return Err(
                        lexer_err(
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
                        .with_hint("valid children of while: condition, body"),
                    );
                }
            }
        }

        self.tokens.push(HaplToken::close_loop(LoopType::While));
        Ok(())
    }

    // --------------------------------------------------
    // For loop <div class="for">
    // --------------------------------------------------
    fn walk_for(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
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
                    return Err(
                        lexer_err(
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
                        .with_hint("valid children of for: iterator, condition, increment, body"),
                    );
                }
            }
        }

        self.tokens.push(HaplToken::close_loop(LoopType::For));
        Ok(())
    }

    // --------------------------------------------------
    // Function declaration
    // --------------------------------------------------
    fn walk_function_decl(
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
                            lexer_err(
                                ErrorCode::MissingId,
                                "parameter tag is missing an id attribute",
                            )
                            .with_tag(
                                tag_snippet(grandchild),
                                "id is required to name the parameter",
                            )
                            .with_hint(
                                "example: <div class=\"integer-param\" id=\"age\"></div>",
                            )
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
                    return Err(
                        lexer_err(
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
                        .with_hint("valid children of a function declaration: params, body"),
                    );
                }
            }
        }

        self.tokens.push(HaplToken::new(
            HaplTokenType::CloseFunction { name: name.clone() },
            Some(name),
        ));
        Ok(())
    }

    // --------------------------------------------------
    // Function call <div class="funcName">
    // --------------------------------------------------
    fn walk_function_call(&mut self, tag: &HtmlTag, name: String) -> Result<(), HaplError> {
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





    fn walk_list_decl(&mut self, tag: &HtmlTag, name: String, elem_type: StaticType) -> Result<(), HaplError> {
        self.tokens.push(HaplToken::open_list_dec(elem_type, name.clone()));
        for child in &tag.child_tags {
            if child.tag_type != "span" {
                return Err(
                    lexer_err(ErrorCode::BadListChild,
                        format!("unexpected <{}> inside list declaration '{}'", child.tag_type, name))
                    .with_tag(tag_snippet(child), "only literal <span> values are valid inside a list declaration")
                    .with_hint("example: <span class=\"integer\">42</span>"),
                );
            }
            self.walk(child)?;
        }
        self.tokens.push(HaplToken::close_list_dec(name));
        Ok(())
    }

    fn walk_list_access(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        self.tokens.push(HaplToken::open_list_access());
        for child in &tag.child_tags { self.walk(child)?; }
        self.tokens.push(HaplToken::close_list_access());
        Ok(())
    }

    fn walk_list_assign(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        let name = self.extract_list_name(tag, "index-assign")?;
        self.tokens.push(HaplToken::open_list_assign(name.clone()));
        for child in &tag.child_tags { self.walk(child)?; }
        self.tokens.push(HaplToken::close_list_assign(name));
        Ok(())
    }

    fn walk_list_push(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        let name = self.extract_list_name(tag, "push")?;
        self.tokens.push(HaplToken::open_list_push(name.clone()));
        for child in &tag.child_tags { self.walk(child)?; }
        self.tokens.push(HaplToken::close_list_push(name));
        Ok(())
    }

    fn walk_list_pop(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        let name = self.extract_list_name(tag, "pop")?;
        self.tokens.push(HaplToken::open_list_pop(name.clone()));
        self.tokens.push(HaplToken::close_list_pop(name));
        Ok(())
    }

    fn extract_list_name(&self, tag: &HtmlTag, op: &str) -> Result<String, HaplError> {
        let first = tag.child_tags.first().ok_or_else(|| {
            lexer_err(ErrorCode::MissingId, format!("<div class=\"{}\"> has no children", op))
            .with_tag(tag_snippet(tag), "expected a <var> reference as the first child")
            .with_hint(format!("example: <div class=\"{}\"><var class=\"myList\"></var>...</div>", op))
        })?;
        if first.tag_type != "var" {
            return Err(
                lexer_err(ErrorCode::UnexpectedToken,
                    format!("first child of <div class=\"{}\"> must be a <var>, got <{}>", op, first.tag_type))
                .with_tag(tag_snippet(first), "expected a <var> reference here")
                .with_hint("the first child must identify the list by name"),
            );
        }
        first.class.clone().ok_or_else(|| {
            lexer_err(ErrorCode::MissingClass,
                format!("<var> inside <div class=\"{}\"> is missing a class attribute", op))
            .with_tag(tag_snippet(first), "class attribute is the list name")
            .with_hint("example: <var class=\"myList\"></var>")
        })
    }

    fn parse_list_class(&self, class: &str) -> Option<StaticType> {
        match class.strip_suffix("-list")? {
            "integer" => Some(StaticType::Integer),
            "double"  => Some(StaticType::Double),
            "string"  => Some(StaticType::String),
            "boolean" => Some(StaticType::Boolean),
            _         => None,
        }
    }

    // --------------------------------------------------
    // Literal parsing — now takes the whole tag for error context
    // --------------------------------------------------
    fn parse_literal(&self, tag: &HtmlTag) -> Result<HaplToken, HaplError> {
        let trimmed = tag.content.trim();

        // let class = tag.class.as_deref().ok_or_else(|| {
        //     lexer_err(
        //         ErrorCode::MissingClass,
        //         format!(
        //             "literal tag containing '{}' has no class attribute",
        //             trimmed
        //         ),
        //     )
        //     .with_tag(
        //         tag_snippet(tag),
        //         "class attribute required to identify the type",
        //     )
        //     .with_hint("example: <span class=\"integer\">42</span>")
        // })?;
        let raw_class = tag.class.as_deref().ok_or_else(|| {
        lexer_err(
            ErrorCode::MissingClass,
            format!(
                "literal tag containing '{}' has no class attribute",
                trimmed
            ),
        )
        .with_tag(
            tag_snippet(tag),
            "class attribute required to identify the type",
        )
        .with_hint("example: <span class=\"integer\">42</span>")
    })?;
    let class = self.resolve(raw_class);


        match class {
            "integer" => trimmed.parse::<i64>().map(HaplToken::integer).map_err(|_| {
                lexer_err(
                    ErrorCode::InvalidLiteral,
                    format!("'{}' is not a valid integer", trimmed),
                )
                .with_tag(
                    tag_snippet(tag),
                    format!("cannot parse '{}' as integer", trimmed),
                )
                .with_hint("integer literals must be whole numbers, e.g. 42 or -7")
            }),

            "double" => trimmed.parse::<f64>().map(HaplToken::double).map_err(|_| {
                lexer_err(
                    ErrorCode::InvalidLiteral,
                    format!("'{}' is not a valid double", trimmed),
                )
                .with_tag(
                    tag_snippet(tag),
                    format!("cannot parse '{}' as double", trimmed),
                )
                .with_hint("double literals must be floating-point numbers, e.g. 3.14")
            }),

            "string" => {
                if !trimmed.starts_with('"') || !trimmed.ends_with('"') || trimmed.len() < 2 {
                    return Err(
                        lexer_err(
                            ErrorCode::StringNotQuoted,
                            format!("string literal '{}' is not enclosed in double quotes", trimmed),
                        )
                        .with_tag(
                            tag_snippet(tag),
                            "string content must be wrapped in double quotes",
                        )
                        .with_hint("example: <span class=\"string\">\"hello world\"</span>"),
                    );
                }
                let inner = &trimmed[1..trimmed.len() - 1];
                Ok(HaplToken::string(inner.to_string()))
            }

            "boolean" => match trimmed {
                "true"  => Ok(HaplToken::boolean(true)),
                "false" => Ok(HaplToken::boolean(false)),
                _ => Err(
                    lexer_err(
                        ErrorCode::InvalidLiteral,
                        format!("'{}' is not a valid boolean", trimmed),
                    )
                    .with_tag(
                        tag_snippet(tag),
                        format!("expected 'true' or 'false', got '{}'", trimmed),
                    )
                    .with_hint("boolean literals must be exactly true or false"),
                ),
            },

            other => Err(
                lexer_err(
                    ErrorCode::UnknownType,
                    format!("unknown literal type '{}'", other),
                )
                .with_tag(
                    tag_snippet(tag),
                    format!("'{}' is not a recognized type", other),
                )
                .with_hint("valid literal types: integer, double, boolean, string"),
            ),
        }
    }

    // --------------------------------------------------
    // Operator helpers
    // --------------------------------------------------

    fn try_arithmetic_tokens(&self, class: &str) -> Option<(HaplToken, HaplToken)> {
        let tag_type = match class {
            "+" => LexerTagType::Add,
            "-" => LexerTagType::Subtract,
            "*" => LexerTagType::Multiply,
            "/" => LexerTagType::Divide,
            "%" => LexerTagType::Modulo,

            _   => return None,
        };
        Some((HaplToken::open_operator(tag_type), HaplToken::close_operator(tag_type)))
    }

    fn try_boolean_tokens(&self, class: &str) -> Option<(HaplToken, HaplToken)> {
        let tag_type = match class {
            "&&" => LexerTagType::And,
            "||" => LexerTagType::Or,
            "!"  => LexerTagType::Not,
            _    => return None,
        };
        Some((HaplToken::open_operator(tag_type), HaplToken::close_operator(tag_type)))
    }

    fn try_comparison_tokens(&self, class: &str) -> Option<(HaplToken, HaplToken)> {
        let tag_type = match class {
            "equal"         => LexerTagType::Equal,
            "not_equal"     => LexerTagType::NotEqual,
            "less"          => LexerTagType::Less,
            "less_equal"    => LexerTagType::LessEqual,
            "greater"       => LexerTagType::Greater,
            "greater_equal" => LexerTagType::GreaterEqual,
            _               => return None,
        };
        Some((HaplToken::open_operator(tag_type), HaplToken::close_operator(tag_type)))
    }

    fn parse_function_class(&self, class: &str) -> Option<StaticType> {
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

    fn parse_param_class(&self, class: Option<&str>) -> Option<StaticType> {
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

    fn walk_map_dec(&mut self, tag: &HtmlTag, name: String) -> Result<(), HaplError> {
        self.tokens.push(HaplToken::open_map_dec(name.clone()));

        for child in &tag.child_tags {
            let key = child.class.clone().ok_or_else(|| {
                lexer_err(ErrorCode::MissingClass,
                    format!("map entry inside '{}' is missing a class attribute", name))
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

    fn walk_map_get(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
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

    fn walk_map_contains(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
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

    fn walk_map_set(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
        // first child is the <var> — extract the map name from it
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

    fn walk_map_remove(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
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

    // mirrors extract_list_name — grabs the map variable name from the first <var> child
    fn extract_map_name(&self, tag: &HtmlTag, op: &str) -> Result<String, HaplError> {
        let first = tag.child_tags.first().ok_or_else(|| {
            lexer_err(ErrorCode::MissingId,
                format!("<div class=\"{}\"> has no children", op))
            .with_tag(tag_snippet(tag), "expected a <var> reference as the first child")
            .with_hint(format!("example: <div class=\"{}\"><var class=\"myMap\"></var>...</div>", op))
        })?;

        if first.tag_type != "var" {
            return Err(
                lexer_err(ErrorCode::UnexpectedToken,
                    format!("first child of <div class=\"{}\"> must be a <var>, got <{}>", op, first.tag_type))
                .with_tag(tag_snippet(first), "expected a <var> reference here")
                .with_hint("the first child must identify the map by name")
            );
        }

        first.class.clone().ok_or_else(|| {
            lexer_err(ErrorCode::MissingClass,
                format!("<var> inside <div class=\"{}\"> is missing a class attribute", op))
            .with_tag(tag_snippet(first), "class attribute is the map name")
            .with_hint("example: <var class=\"myMap\"></var>")
        })
    }

    fn walk_length(&mut self, tag: &HtmlTag) -> Result<(), HaplError> {
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

    fn resolve<'a>(&'a self, class: &'a str) -> &'a str {
        self.config.as_ref()
            .map(|c| c.resolve(class))
            .unwrap_or(class)
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
                HaplTokenType::OpenHtmlTag { name } => {
                    println!("OpenHtmlTag({}) -> {:?}", name, token.value);
                }
                HaplTokenType::CloseHtmlTag { name } => {
                    println!("CloseHtmlTag({}) -> {:?}", name, token.value);
                }
                HaplTokenType::Literal(lit) => {
                    println!("Literal({:?}) -> {:?}", lit, token.value);
                }
                HaplTokenType::OpenVarDec { var_type, name } => {
                    println!("OpenVarDec({:?}, {}) -> {:?}", var_type, name, token.value);
                }
                HaplTokenType::CloseVarDec { var_type, name } => {
                    println!("CloseVarDec({:?}, {}) -> {:?}", var_type, name, token.value);
                }
                HaplTokenType::OpenVarRef { name } => {
                    println!("OpenVarRef({}) -> {:?}", name, token.value);
                }
                HaplTokenType::CloseVarRef { name } => {
                    println!("CloseVarRef({}) -> {:?}", name, token.value);
                }
                HaplTokenType::OpenVarAssign { name } => {
                    println!("OpenVarAssign({}) -> {:?}", name, token.value);
                }
                HaplTokenType::CloseVarAssign { name } => {
                    println!("CloseVarAssign({}) -> {:?}", name, token.value);
                }
                HaplTokenType::OpenPrint => println!("OpenPrint -> {:?}", token.value),
                HaplTokenType::ClosePrint => println!("ClosePrint -> {:?}", token.value),
                HaplTokenType::OpenConditional => println!("OpenConditional -> {:?}", token.value),
                HaplTokenType::CloseConditional => println!("CloseConditional -> {:?}", token.value),
                HaplTokenType::OpenIf => println!("OpenIf -> {:?}", token.value),
                HaplTokenType::CloseIf => println!("CloseIf -> {:?}", token.value),
                HaplTokenType::OpenElif => println!("OpenElif -> {:?}", token.value),
                HaplTokenType::CloseElif => println!("CloseElif -> {:?}", token.value),
                HaplTokenType::OpenElse => println!("OpenElse -> {:?}", token.value),
                HaplTokenType::CloseElse => println!("CloseElse -> {:?}", token.value),
                HaplTokenType::OpenLoop { loop_type } => {
                    println!("OpenLoop({:?}) -> {:?}", loop_type, token.value);
                }
                HaplTokenType::CloseLoop { loop_type } => {
                    println!("CloseLoop({:?}) -> {:?}", loop_type, token.value);
                }
                HaplTokenType::OpenLoopCondition => println!("OpenLoopCondition -> {:?}", token.value),
                HaplTokenType::CloseLoopCondition => println!("CloseLoopCondition -> {:?}", token.value),
                HaplTokenType::OpenLoopBody => println!("OpenLoopBody -> {:?}", token.value),
                HaplTokenType::CloseLoopBody => println!("CloseLoopBody -> {:?}", token.value),
                HaplTokenType::OpenLoopIterator => println!("OpenLoopIterator -> {:?}", token.value),
                HaplTokenType::CloseLoopIterator => println!("CloseLoopIterator -> {:?}", token.value),
                HaplTokenType::OpenLoopIncrement => println!("OpenLoopIncrement -> {:?}", token.value),
                HaplTokenType::CloseLoopIncrement => println!("CloseLoopIncrement -> {:?}", token.value),
                HaplTokenType::OpenFunction { name, return_type } => {
                    println!("OpenFunction({}, {:?}) -> {:?}", name, return_type, token.value);
                }
                HaplTokenType::CloseFunction { name } => {
                    println!("CloseFunction({}) -> {:?}", name, token.value);
                }
                HaplTokenType::OpenParams => println!("OpenParams -> {:?}", token.value),
                HaplTokenType::CloseParams => println!("CloseParams -> {:?}", token.value),
                HaplTokenType::OpenParam { name, param_type } => {
                    println!("OpenParam({}, {:?}) -> {:?}", name, param_type, token.value);
                }
                HaplTokenType::CloseParam { name } => {
                    println!("CloseParam({}) -> {:?}", name, token.value);
                }
                HaplTokenType::OpenFunctionBody => println!("OpenFunctionBody -> {:?}", token.value),
                HaplTokenType::CloseFunctionBody => println!("CloseFunctionBody -> {:?}", token.value),
                HaplTokenType::OpenFunctionCall { name } => {
                    println!("OpenFunctionCall({}) -> {:?}", name, token.value);
                }
                HaplTokenType::CloseFunctionCall { name } => {
                    println!("CloseFunctionCall({}) -> {:?}", name, token.value);
                }
                HaplTokenType::OpenReturn => println!("OpenReturn -> {:?}", token.value),
                HaplTokenType::CloseReturn => println!("CloseReturn -> {:?}", token.value),
                HaplTokenType::OpenArgs => println!("OpenArgs -> {:?}", token.value),
                HaplTokenType::CloseArgs => println!("CloseArgs -> {:?}", token.value),

                HaplTokenType::OpenListDec { elem_type, name } => {
                    println!("OpenListDec({:?}, {}) -> {:?}", elem_type, name, token.value);
                }
                HaplTokenType::CloseListDec { name } => {
                    println!("CloseListDec({}) -> {:?}", name, token.value);
                }
                HaplTokenType::OpenListAccess => println!("OpenListAccess -> {:?}", token.value),
                HaplTokenType::CloseListAccess => println!("CloseListAccess -> {:?}", token.value),
                HaplTokenType::OpenListAssign { name } => {
                    println!("OpenListAssign({}) -> {:?}", name, token.value);
                }
                HaplTokenType::CloseListAssign { name } => {
                    println!("CloseListAssign({}) -> {:?}", name, token.value);
                }
                HaplTokenType::OpenListPush { name } => {
                    println!("OpenListPush({}) -> {:?}", name, token.value);
                }
                HaplTokenType::CloseListPush { name } => {
                    println!("CloseListPush({}) -> {:?}", name, token.value);
                }
                HaplTokenType::OpenListPop { name } => {
                    println!("OpenListPop({}) -> {:?}", name, token.value);
                }
                HaplTokenType::CloseListPop { name } => {
                    println!("CloseListPop({}) -> {:?}", name, token.value);
                }
                HaplTokenType::OpenMapDec { name } => println!("OpenMapDec({}) -> {:?}", name, token.value),
                HaplTokenType::CloseMapDec { name } => println!("CloseMapDec({}) -> {:?}", name, token.value),
                HaplTokenType::OpenMapGet => println!("OpenMapGet -> {:?}", token.value),
                HaplTokenType::CloseMapGet => println!("CloseMapGet -> {:?}", token.value),
                HaplTokenType::OpenMapSet { name } => println!("OpenMapSet({}) -> {:?}", name, token.value),
                HaplTokenType::CloseMapSet { name } => println!("CloseMapSet({}) -> {:?}", name, token.value),
                HaplTokenType::OpenMapRemove { name } => println!("OpenMapRemove({}) -> {:?}", name, token.value),
                HaplTokenType::CloseMapRemove { name } => println!("CloseMapRemove({}) -> {:?}", name, token.value),
                HaplTokenType::OpenMapContains => println!("OpenMapContains -> {:?}", token.value),
                HaplTokenType::CloseMapContains => println!("CloseMapContains -> {:?}", token.value),
                HaplTokenType::OpenMapKey => println!("OpenMapKey -> {:?}", token.value),
                HaplTokenType::CloseMapKey => println!("CloseMapKey -> {:?}", token.value),
                HaplTokenType::OpenMapValue => println!("OpenMapValue -> {:?}", token.value),
                HaplTokenType::CloseMapValue => println!("CloseMapValue -> {:?}", token.value),
                HaplTokenType::OpenMapEntry { key } => println!("OpenMapEntry({}) -> {:?}", key, token.value),
                HaplTokenType::CloseMapEntry { key } => println!("CloseMapEntry({}) -> {:?}", key, token.value),
                HaplTokenType::OpenLength  => println!("OpenLength -> {:?}", token.value),
                HaplTokenType::CloseLength => println!("CloseLength -> {:?}", token.value),

            }
        }
    }
}

// ------------------------------------------------------------------
// Free helpers
// ------------------------------------------------------------------

/// Returns `None` for unknown types instead of panicking — callers handle the error.
fn parse_static_type(class: &str) -> Option<StaticType> {
    match class {
        "integer" => Some(StaticType::Integer),
        "double"  => Some(StaticType::Double),
        "string"  => Some(StaticType::String),
        "boolean" => Some(StaticType::Boolean),
        "void"    => Some(StaticType::Void),
        "map"     => Some(StaticType::Map),
        _         => None,
    }
}

fn operator_to_str(op: LexerTagType) -> &'static str {
    match op {
        LexerTagType::Add          => "+",
        LexerTagType::Subtract     => "-",
        LexerTagType::Multiply     => "*",
        LexerTagType::Divide       => "/",
        LexerTagType::Modulo       => "%",
        LexerTagType::And          => "&&",
        LexerTagType::Or           => "||",
        LexerTagType::Not          => "!",
        LexerTagType::Equal        => "==",
        LexerTagType::NotEqual     => "!=",
        LexerTagType::Less         => "<",
        LexerTagType::LessEqual    => "<=",
        LexerTagType::Greater      => ">",
        LexerTagType::GreaterEqual => ">=",
    }
}