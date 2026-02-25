use crate::HtmlTag;
use crate::ast::{LiteralValue, StaticType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LexerTagType {
    Add,
    Subtract,
    Multiply,
    Divide,
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
    // FIX: OpenPrint and ClosePrint are unit variants (no fields).
    // The parser must match them as `HaplTokenType::OpenPrint` not `HaplTokenType::OpenPrint { .. }`.
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
            HaplTokenType::OpenParam { name: name.clone(), param_type },
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
}

pub struct HaplLexer {
    tokens: Vec<HaplToken>,
}

impl HaplLexer {
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    pub fn lex(&mut self, tag: &HtmlTag) {
        self.walk(tag);
    }

    pub fn tokens(&self) -> &[HaplToken] {
        &self.tokens
    }

    // --------------------------------------------------
    // Top-level dispatcher — splits walk by tag type
    // FIX: Previously one giant method; now dispatches to focused helpers.
    // This makes it much easier to add new tag types without risk of
    // accidentally falling through to the wrong handler.
    // --------------------------------------------------
    fn walk(&mut self, tag: &HtmlTag) {
        match tag.tag_type.as_str() {
            "div"  => self.walk_div(tag),
            "p"    => self.walk_print(tag),
            "var"  => self.walk_var(tag),
            "span" => self.walk_span(tag),
            // Structural HTML tags are walked transparently — their open/close tokens
            // are emitted so the token stream reflects the document structure, but the
            // parser simply ignores unknown structural wrappers and recurses into children.
            // This allows HAPL programs to live inside real, valid HTML files.
            "html" | "head" | "body" | "title" | "meta" | "link" => {
                self.tokens.push(HaplToken::open_html_tag(tag.tag_type.clone()));
                for child in &tag.child_tags {
                    self.walk(child);
                }
                self.tokens.push(HaplToken::close_html_tag(tag.tag_type.clone()));
            }

            other => {
                // FIX: Unknown tags now panic with a clear message instead of
                // silently falling through to the literal parser.
                panic!(
                    "Unrecognized tag type '{}'. \
                     Supported tags: div, p, var, span",
                    other
                );
            }
        }
    }

    // --------------------------------------------------
    // <p> → Print statement
    // --------------------------------------------------
    fn walk_print(&mut self, tag: &HtmlTag) {
        self.tokens.push(HaplToken::open_print());
        for child in &tag.child_tags {
            self.walk(child);
        }
        self.tokens.push(HaplToken::close_print());
    }

    // --------------------------------------------------
    // <var> → Variable declaration, reference, or assignment
    // --------------------------------------------------
    fn walk_var(&mut self, tag: &HtmlTag) {
        let class = tag.class.as_ref().expect(
            "<var> tag requires a class attribute (type name for declaration, variable name for reference/assignment)"
        );

        if let Some(id) = &tag.id {
            // Variable declaration: <var class="integer" id="x">10</var>
            let var_type = parse_static_type(class);

            self.tokens.push(HaplToken::new(
                HaplTokenType::OpenVarDec { var_type, name: id.clone() },
                Some(format!("{} {}", class, id)),
            ));

            for child in &tag.child_tags {
                self.walk(child);
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
                self.walk(child);
            }
            self.tokens.push(HaplToken::new(
                HaplTokenType::CloseVarAssign { name: class.clone() },
                Some(class.clone()),
            ));
        }
    }

    // --------------------------------------------------
    // <span> → Literal value
    // --------------------------------------------------
    fn walk_span(&mut self, tag: &HtmlTag) {
        if tag.child_tags.is_empty() && !tag.content.trim().is_empty() {
            let token = self.parse_literal(&tag.content, tag.class.as_deref());
            self.tokens.push(token);
        } else {
            for child in &tag.child_tags {
                self.walk(child);
            }
        }
    }

    // --------------------------------------------------
    // <div> → All language constructs
    // FIX: Broken into clearly ordered specific checks. The function-call
    // fallback now only fires after ALL known classes have been checked,
    // removing the need for a manually maintained reserved-words list that
    // could drift out of sync.
    // --------------------------------------------------
    fn walk_div(&mut self, tag: &HtmlTag) {
        let class = match &tag.class {
            Some(c) => c.clone(),
            None => {
                // A <div> with no class is not meaningful in this language
                panic!("A <div> tag must have a class attribute");
            }
        };

        // ---- Arithmetic operators ----
        if let Some((open, close)) = self.try_arithmetic_tokens(&class) {
            self.tokens.push(open);
            for child in &tag.child_tags { self.walk(child); }
            self.tokens.push(close);
            return;
        }

        // ---- Boolean operators ----
        if let Some((open, close)) = self.try_boolean_tokens(&class) {
            self.tokens.push(open);
            for child in &tag.child_tags { self.walk(child); }
            self.tokens.push(close);
            return;
        }

        // ---- Comparison operators ----
        if let Some((open, close)) = self.try_comparison_tokens(&class) {
            self.tokens.push(open);
            for child in &tag.child_tags { self.walk(child); }
            self.tokens.push(close);
            return;
        }

        // ---- Conditional ----
        if class == "conditional" {
            self.walk_conditional(tag);
            return;
        }

        // ---- While loop ----
        if class == "while" {
            self.walk_while(tag);
            return;
        }

        // ---- For loop ----
        if class == "for" {
            self.walk_for(tag);
            return;
        }

        // ---- Return ----
        if class == "return" {
            self.tokens.push(HaplToken::open_return());
            for child in &tag.child_tags {
                self.walk(child);
            }
            self.tokens.push(HaplToken::close_return());
            return;
        }

        // ---- Function declaration: <div class="integer-function" id="myFunc"> ----
        if let Some(return_type) = self.parse_function_class(&class) {
            let name = tag.id.clone()
                .expect("Function declaration <div> must have an id attribute");
            self.walk_function_decl(tag, name, return_type);
            return;
        }

        // ---- Function call: <div class="myFunc"> (no id, not a known keyword) ----
        // FIX: No manually maintained reserved list. By this point every known
        // class has already been handled above and returned early. Anything
        // remaining that has no id is treated as a function call.
        if tag.id.is_none() {
            self.walk_function_call(tag, class);
            return;
        }

        panic!(
            "Unrecognized <div> usage: class='{}', id={:?}",
            class,
            tag.id
        );
    }

    // --------------------------------------------------
    // Conditional <div class="conditional">
    // --------------------------------------------------
    fn walk_conditional(&mut self, tag: &HtmlTag) {
        self.tokens.push(HaplToken::open_conditional());

        for child in &tag.child_tags {
            if child.tag_type != "div" { continue; }
            match child.class.as_deref() {
                Some("if") => {
                    self.tokens.push(HaplToken::open_if());
                    for grandchild in &child.child_tags { self.walk(grandchild); }
                    self.tokens.push(HaplToken::close_if());
                }
                Some("elif") => {
                    self.tokens.push(HaplToken::open_elif());
                    for grandchild in &child.child_tags { self.walk(grandchild); }
                    self.tokens.push(HaplToken::close_elif());
                }
                Some("else") => {
                    self.tokens.push(HaplToken::open_else());
                    for grandchild in &child.child_tags { self.walk(grandchild); }
                    self.tokens.push(HaplToken::close_else());
                }
                other => panic!(
                    "Unexpected child class '{:?}' inside <div class=\"conditional\">. \
                     Expected 'if', 'elif', or 'else'.",
                    other
                ),
            }
        }

        self.tokens.push(HaplToken::close_conditional());
    }

    // --------------------------------------------------
    // While loop <div class="while">
    // --------------------------------------------------
    fn walk_while(&mut self, tag: &HtmlTag) {
        self.tokens.push(HaplToken::open_loop(LoopType::While));

        for child in &tag.child_tags {
            if child.tag_type != "div" { continue; }
            match child.class.as_deref() {
                Some("condition") => {
                    self.tokens.push(HaplToken::open_loop_condition());
                    for grandchild in &child.child_tags { self.walk(grandchild); }
                    self.tokens.push(HaplToken::close_loop_condition());
                }
                Some("body") => {
                    self.tokens.push(HaplToken::open_loop_body());
                    for grandchild in &child.child_tags { self.walk(grandchild); }
                    self.tokens.push(HaplToken::close_loop_body());
                }
                other => panic!(
                    "Unexpected child class '{:?}' inside <div class=\"while\">. \
                     Expected 'condition' or 'body'.",
                    other
                ),
            }
        }

        self.tokens.push(HaplToken::close_loop(LoopType::While));
    }

    // --------------------------------------------------
    // For loop <div class="for">
    // --------------------------------------------------
    fn walk_for(&mut self, tag: &HtmlTag) {
        self.tokens.push(HaplToken::open_loop(LoopType::For));

        for child in &tag.child_tags {
            if child.tag_type != "div" { continue; }
            match child.class.as_deref() {
                Some("iterator") => {
                    self.tokens.push(HaplToken::open_loop_iterator());
                    for grandchild in &child.child_tags { self.walk(grandchild); }
                    self.tokens.push(HaplToken::close_loop_iterator());
                }
                Some("condition") => {
                    self.tokens.push(HaplToken::open_loop_condition());
                    for grandchild in &child.child_tags { self.walk(grandchild); }
                    self.tokens.push(HaplToken::close_loop_condition());
                }
                Some("increment") => {
                    self.tokens.push(HaplToken::open_loop_increment());
                    for grandchild in &child.child_tags { self.walk(grandchild); }
                    self.tokens.push(HaplToken::close_loop_increment());
                }
                Some("body") => {
                    self.tokens.push(HaplToken::open_loop_body());
                    for grandchild in &child.child_tags { self.walk(grandchild); }
                    self.tokens.push(HaplToken::close_loop_body());
                }
                other => panic!(
                    "Unexpected child class '{:?}' inside <div class=\"for\">. \
                     Expected 'iterator', 'condition', 'increment', or 'body'.",
                    other
                ),
            }
        }

        self.tokens.push(HaplToken::close_loop(LoopType::For));
    }

    // --------------------------------------------------
    // Function declaration
    // --------------------------------------------------
    fn walk_function_decl(&mut self, tag: &HtmlTag, name: String, return_type: StaticType) {
        self.tokens.push(HaplToken::new(
            HaplTokenType::OpenFunction { name: name.clone(), return_type },
            Some(name.clone()),
        ));

        for child in &tag.child_tags {
            match child.class.as_deref() {
                Some("params") => {
                    self.tokens.push(HaplToken::open_params());

                    for grandchild in &child.child_tags {
                        // FIX: parse_param_class now uses "double" instead of "float"
                        let param_type = self
                            .parse_param_class(grandchild.class.as_deref())
                            .unwrap_or_else(|| {
                                panic!(
                                    "Invalid or missing param type on grandchild: class={:?}",
                                    grandchild.class
                                )
                            });

                        let param_name = grandchild.id.clone()
                            .expect("Parameter <div> must have an id attribute");

                        self.tokens.push(HaplToken::new(
                            HaplTokenType::OpenParam { name: param_name.clone(), param_type },
                            Some(param_name.clone()),
                        ));

                        for param_child in &grandchild.child_tags {
                            self.walk(param_child);
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
                        self.walk(grandchild);
                    }
                    self.tokens.push(HaplToken::close_function_body());
                }

                other => panic!(
                    "Unexpected child class '{:?}' inside function declaration '{}'. \
                     Expected 'params' or 'body'.",
                    other, name
                ),
            }
        }

        self.tokens.push(HaplToken::new(
            HaplTokenType::CloseFunction { name: name.clone() },
            Some(name),
        ));
    }

    // --------------------------------------------------
    // Function call <div class="funcName">
    // --------------------------------------------------
    fn walk_function_call(&mut self, tag: &HtmlTag, name: String) {
        self.tokens.push(HaplToken::new(
            HaplTokenType::OpenFunctionCall { name: name.clone() },
            Some(name.clone()),
        ));

        for child in &tag.child_tags {
            if child.class.as_deref() == Some("args") {
                self.tokens.push(HaplToken::open_args());
                for grandchild in &child.child_tags {
                    self.walk(grandchild);
                }
                self.tokens.push(HaplToken::close_args());
            } else {
                self.walk(child);
            }
        }

        self.tokens.push(HaplToken::new(
            HaplTokenType::CloseFunctionCall { name: name.clone() },
            Some(name),
        ));
    }

    // --------------------------------------------------
    // Literal parsing
    // --------------------------------------------------
    fn parse_literal(&self, text: &str, class_type: Option<&str>) -> HaplToken {
        let trimmed = text.trim();

        // FIX: Error message now includes the offending content for easier debugging
        let class = class_type.unwrap_or_else(|| {
            panic!(
                "Static type class required on literal tag containing '{}'. \
                 Expected class='integer', 'double', 'boolean', or 'string'.",
                trimmed
            )
        });

        match class {
            "integer" => trimmed
                .parse::<i64>()
                .map(HaplToken::integer)
                .unwrap_or_else(|_| {
                    panic!("Type error: '{}' is not a valid integer", trimmed)
                }),

            "double" => trimmed
                .parse::<f64>()
                .map(HaplToken::double)
                .unwrap_or_else(|_| {
                    panic!("Type error: '{}' is not a valid double", trimmed)
                }),

            "string" => {
                if !trimmed.starts_with('"') || !trimmed.ends_with('"') || trimmed.len() < 2 {
                    panic!(
                        "String literals must be enclosed in double quotes but got '{}'",
                        trimmed
                    );
                }
                let inner = &trimmed[1..trimmed.len() - 1];
                HaplToken::string(inner.to_string())
            }

            "boolean" => match trimmed {
                "true"  => HaplToken::boolean(true),
                "false" => HaplToken::boolean(false),
                _ => panic!(
                    "Type error: '{}' is not a valid boolean (expected 'true' or 'false')",
                    trimmed
                ),
            },

            other => panic!(
                "Unknown static type '{}' on literal containing '{}'. \
                 Expected 'integer', 'double', 'boolean', or 'string'.",
                other, trimmed
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
        Some(match type_part {
            "integer" => StaticType::Integer,
            "double"  => StaticType::Double,
            "string"  => StaticType::String,
            "boolean" => StaticType::Boolean,
            "void"    => StaticType::Void,
            other     => panic!("Unknown function return type '{}'", other),
        })
    }

    fn parse_param_class(&self, class: Option<&str>) -> Option<StaticType> {
        let class = class?;
        let type_part = class.strip_suffix("-param")?;

        Some(match type_part {
            "integer" => StaticType::Integer,
            // FIX: Was "float" which didn't match any actual HTML class used elsewhere.
            // Changed to "double" to match the rest of the type system.
            "double"  => StaticType::Double,
            "string"  => StaticType::String,
            "boolean" => StaticType::Boolean,
            "void"    => StaticType::Void,
            _         => return None,
        })
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
                HaplTokenType::OpenPrint => {
                    println!("OpenPrint -> {:?}", token.value);
                }
                HaplTokenType::ClosePrint => {
                    println!("ClosePrint -> {:?}", token.value);
                }
                HaplTokenType::OpenConditional => {
                    println!("OpenConditional -> {:?}", token.value);
                }
                HaplTokenType::CloseConditional => {
                    println!("CloseConditional -> {:?}", token.value);
                }
                HaplTokenType::OpenIf => {
                    println!("OpenIf -> {:?}", token.value);
                }
                HaplTokenType::CloseIf => {
                    println!("CloseIf -> {:?}", token.value);
                }
                HaplTokenType::OpenElif => {
                    println!("OpenElif -> {:?}", token.value);
                }
                HaplTokenType::CloseElif => {
                    println!("CloseElif -> {:?}", token.value);
                }
                HaplTokenType::OpenElse => {
                    println!("OpenElse -> {:?}", token.value);
                }
                HaplTokenType::CloseElse => {
                    println!("CloseElse -> {:?}", token.value);
                }
                HaplTokenType::OpenLoop { loop_type } => {
                    println!("OpenLoop({:?}) -> {:?}", loop_type, token.value);
                }
                HaplTokenType::CloseLoop { loop_type } => {
                    println!("CloseLoop({:?}) -> {:?}", loop_type, token.value);
                }
                HaplTokenType::OpenLoopCondition => {
                    println!("OpenLoopCondition -> {:?}", token.value);
                }
                HaplTokenType::CloseLoopCondition => {
                    println!("CloseLoopCondition -> {:?}", token.value);
                }
                HaplTokenType::OpenLoopBody => {
                    println!("OpenLoopBody -> {:?}", token.value);
                }
                HaplTokenType::CloseLoopBody => {
                    println!("CloseLoopBody -> {:?}", token.value);
                }
                HaplTokenType::OpenLoopIterator => {
                    println!("OpenLoopIterator -> {:?}", token.value);
                }
                HaplTokenType::CloseLoopIterator => {
                    println!("CloseLoopIterator -> {:?}", token.value);
                }
                HaplTokenType::OpenLoopIncrement => {
                    println!("OpenLoopIncrement -> {:?}", token.value);
                }
                HaplTokenType::CloseLoopIncrement => {
                    println!("CloseLoopIncrement -> {:?}", token.value);
                }
                HaplTokenType::OpenFunction { name, return_type } => {
                    println!("OpenFunction({}, {:?}) -> {:?}", name, return_type, token.value);
                }
                HaplTokenType::CloseFunction { name } => {
                    println!("CloseFunction({}) -> {:?}", name, token.value);
                }
                HaplTokenType::OpenParams => {
                    println!("OpenParams -> {:?}", token.value);
                }
                HaplTokenType::CloseParams => {
                    println!("CloseParams -> {:?}", token.value);
                }
                HaplTokenType::OpenParam { name, param_type } => {
                    println!("OpenParam({}, {:?}) -> {:?}", name, param_type, token.value);
                }
                HaplTokenType::CloseParam { name } => {
                    println!("CloseParam({}) -> {:?}", name, token.value);
                }
                HaplTokenType::OpenFunctionBody => {
                    println!("OpenFunctionBody -> {:?}", token.value);
                }
                HaplTokenType::CloseFunctionBody => {
                    println!("CloseFunctionBody -> {:?}", token.value);
                }
                HaplTokenType::OpenFunctionCall { name } => {
                    println!("OpenFunctionCall({}) -> {:?}", name, token.value);
                }
                HaplTokenType::CloseFunctionCall { name } => {
                    println!("CloseFunctionCall({}) -> {:?}", name, token.value);
                }
                HaplTokenType::OpenReturn => {
                    println!("OpenReturn -> {:?}", token.value);
                }
                HaplTokenType::CloseReturn => {
                    println!("CloseReturn -> {:?}", token.value);
                }
                HaplTokenType::OpenArgs => {
                    println!("OpenArgs -> {:?}", token.value);
                }
                HaplTokenType::CloseArgs => {
                    println!("CloseArgs -> {:?}", token.value);
                }
            }
        }
    }
}

// --------------------------------------------------
// Free helpers
// --------------------------------------------------

/// Parse a type keyword into a StaticType. Panics with a clear message on failure.
fn parse_static_type(class: &str) -> StaticType {
    match class {
        "integer" => StaticType::Integer,
        "double"  => StaticType::Double,
        "string"  => StaticType::String,
        "boolean" => StaticType::Boolean,
        "void"    => StaticType::Void,
        other     => panic!(
            "Unknown variable type '{}'. Expected 'integer', 'double', 'string', 'boolean', or 'void'.",
            other
        ),
    }
}

fn operator_to_str(op: LexerTagType) -> &'static str {
    match op {
        LexerTagType::Add          => "+",
        LexerTagType::Subtract     => "-",
        LexerTagType::Multiply     => "*",
        LexerTagType::Divide       => "/",
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