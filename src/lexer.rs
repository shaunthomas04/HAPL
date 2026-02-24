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
    OpenPrint,
    ClosePrint,
    
    //conditional logic
    OpenConditional,
    CloseConditional,
    OpenIf,
    CloseIf,
    OpenElif,
    CloseElif,
    OpenElse,
    CloseElse,
    
    //loops
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

    //functions
    OpenFunction { name: String, return_type: StaticType },
    CloseFunction { name: String },
    OpenParams,
    CloseParams,
    OpenParam { name: String, param_type: StaticType, },
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
    pub value: Option<String>, // printable/debug value
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
        Self::new(
            HaplTokenType::OpenHtmlTag { name: name.clone() },
            Some(name),
        )
    }

    fn close_html_tag(name: String) -> Self {
        Self::new(
            HaplTokenType::CloseHtmlTag { name: name.clone() },
            Some(name),
        )
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
            HaplTokenType::OpenFunction { 
                name: name.clone(), 
                return_type 
            },
            Some(name),
        )
    }

    pub fn close_function(name: String) -> Self {
        Self::new(
            HaplTokenType::CloseFunction { name: name.clone() },
            Some(name),
        )
    }

    pub fn open_params() -> Self {
        Self::new(HaplTokenType::OpenParams, Some("params".to_string()))
    }

    pub fn close_params() -> Self {
        Self::new(HaplTokenType::CloseParams, Some("params".to_string()))
    }

    pub fn open_param(name: String, param_type: StaticType) -> Self {
        Self::new(
            HaplTokenType::OpenParam {
                name: name.clone(),
                param_type,
            },
            Some(format!("{:?} {}", param_type, name)),
        )
    }

    pub fn close_param(name: String) -> Self {
        Self::new(
            HaplTokenType::CloseParam {
                name: name.clone(),
            },
            Some(name),
        )
    }

    pub fn open_function_body() -> Self {
        Self::new(HaplTokenType::OpenFunctionBody, Some("function_body".to_string()))
    }

    pub fn close_function_body() -> Self {
        Self::new(HaplTokenType::CloseFunctionBody, Some("function_body".to_string()))
    }

    pub fn open_function_call(name: String) -> Self {
        Self::new(
            HaplTokenType::OpenFunctionCall { name: name.clone() },
            Some(name),
        )
    }

    pub fn close_function_call(name: String) -> Self {
        Self::new(
            HaplTokenType::CloseFunctionCall { name: name.clone() },
            Some(name),
        )
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

    fn walk(&mut self, tag: &HtmlTag) {
        if tag.tag_type == "div" {
            if let Some(class) = &tag.class {
                // -----------------------------------------
                // Arithmetic <div class="+">
                // -----------------------------------------
                if ["+", "-", "*", "/"].contains(&class.as_str()) {
                    let (open_token, close_token) = self.get_arithmetic_tags(class);

                    self.tokens.push(open_token);

                    for child in &tag.child_tags {
                        self.walk(child);
                    }

                    self.tokens.push(close_token);
                    return;
                }

                // -----------------------------------------
                // Logical Operators <div class="&&">
                // -----------------------------------------
                if ["&&", "||", "!"].contains(&class.as_str()) {
                    let (open_token, close_token) = self.get_boolean_tags(class);

                    self.tokens.push(open_token);

                    for child in &tag.child_tags {
                        self.walk(child);
                    }

                    self.tokens.push(close_token);
                    return;
                }

                // -----------------------------------------
                // relational comparison Operators <div class="equal">
                // -----------------------------------------
                if [
                        "equal",
                        "not_equal",
                        "less",
                        "less_equal",
                        "greater",
                        "greater_equal"
                    ].contains(&class.as_str()) {
                    let (open_token, close_token) = self.get_comparison_tags(class);

                    self.tokens.push(open_token);

                    for child in &tag.child_tags {
                        self.walk(child);
                    }

                    self.tokens.push(close_token);
                    return;
                }

                // -----------------------------------------
                // Conditionals <div class="conditional">
                // -----------------------------------------
                if class == "conditional" {
                    self.tokens.push(HaplToken::open_conditional());
                    for child in &tag.child_tags {
                        match child.tag_type.as_str() {
                            "div" => {
                                if let Some(child_class) = &child.class {
                                    match child_class.as_str() {
                                        // -----------------------------------------
                                        // If block <div class="if">
                                        // -----------------------------------------
                                        "if" => {
                                            self.tokens.push(HaplToken::open_if());
                                            for grandchild in &child.child_tags {
                                                self.walk(grandchild);
                                            }
                                            self.tokens.push(HaplToken::close_if());
                                        }
                                        // -----------------------------------------
                                        // Elif <div class="elif">
                                        // -----------------------------------------
                                        "elif" => {
                                            self.tokens.push(HaplToken::open_elif());
                                            for grandchild in &child.child_tags {
                                                self.walk(grandchild);
                                            }
                                            self.tokens.push(HaplToken::close_elif());
                                        }
                                        // -----------------------------------------
                                        // Else <div class="else">
                                        // -----------------------------------------
                                        "else" => {
                                            self.tokens.push(HaplToken::open_else());
                                            for grandchild in &child.child_tags {
                                                self.walk(grandchild);
                                            }
                                            self.tokens.push(HaplToken::close_else());
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    self.tokens.push(HaplToken::close_conditional());
                    return;
                }


                // -----------------------------------------
                // while loops <div class="while">
                // -----------------------------------------
                if class == "while" {
                    self.tokens.push(HaplToken::open_loop(LoopType::While));
                    for child in &tag.child_tags {
                        match child.tag_type.as_str() {
                            "div" => {
                                if let Some(child_class) = &child.class {
                                    match child_class.as_str() {
                                        // -----------------------------------------
                                        // While condition block <div class="condition">
                                        // -----------------------------------------
                                        "condition" => {
                                            self.tokens.push(HaplToken::open_loop_condition());
                                            for grandchild in &child.child_tags {
                                                self.walk(grandchild);
                                            }
                                            self.tokens.push(HaplToken::close_loop_condition());
                                        }
                                        // -----------------------------------------
                                        // while body block <div class="body">
                                        // -----------------------------------------
                                        "body" => {
                                            self.tokens.push(HaplToken::open_loop_body());
                                            for grandchild in &child.child_tags {
                                                self.walk(grandchild);
                                            }
                                            self.tokens.push(HaplToken::close_loop_body());
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    self.tokens.push(HaplToken::close_loop(LoopType::While));
                    return;
                }


                // -----------------------------------------
                // for loops <div class="for">
                // -----------------------------------------
                if class == "for" {
                    self.tokens.push(HaplToken::open_loop(LoopType::For));
                    for child in &tag.child_tags {
                        match child.tag_type.as_str() {
                            "div" => {
                                if let Some(child_class) = &child.class {
                                    match child_class.as_str() {
                                        // -----------------------------------------
                                        // Iterator value condition block <div class="iterator">
                                        // -----------------------------------------
                                        "iterator" => {
                                            self.tokens.push(HaplToken::open_loop_iterator());
                                            for grandchild in &child.child_tags {
                                                self.walk(grandchild);
                                            }
                                            self.tokens.push(HaplToken::close_loop_iterator());
                                        }
                                        // -----------------------------------------
                                        // For loop condition block <div class="condition">
                                        // -----------------------------------------
                                        "condition" => {
                                            self.tokens.push(HaplToken::open_loop_condition());
                                            for grandchild in &child.child_tags {
                                                self.walk(grandchild);
                                            }
                                            self.tokens.push(HaplToken::close_loop_condition());
                                        }
                                        // -----------------------------------------
                                        // Increment value condition block <div class="increment">
                                        // -----------------------------------------
                                        "increment" => {
                                            self.tokens.push(HaplToken::open_loop_increment());
                                            for grandchild in &child.child_tags {
                                                self.walk(grandchild);
                                            }
                                            self.tokens.push(HaplToken::close_loop_increment());
                                        }
                                        // -----------------------------------------
                                        // For loop body block <div class="body">
                                        // -----------------------------------------
                                        "body" => {
                                            self.tokens.push(HaplToken::open_loop_body());
                                            for grandchild in &child.child_tags {
                                                self.walk(grandchild);
                                            }
                                            self.tokens.push(HaplToken::close_loop_body());
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    self.tokens.push(HaplToken::close_loop(LoopType::For));
                    return;
                }

                // -----------------------------------------
                // Function declaration
                // <div class="string-function" id="myFunc">
                // -----------------------------------------
                if let Some(return_type) = self.parse_function_class(class) {
                    let name = tag.id.clone()
                        .expect("Function must have an id attribute");

                    self.tokens.push(
                        HaplToken::new(
                            HaplTokenType::OpenFunction {
                                name: name.clone(),
                                return_type,
                            },
                            Some(name.clone()),
                        )
                    );

                    for child in &tag.child_tags {
                        if let Some(child_class) = &child.class {
                            match child_class.as_str() {

                                "params" => {
                                    self.tokens.push(
                                        HaplToken::new(
                                            HaplTokenType::OpenParams,
                                            Some("params".to_string())
                                        )
                                    );

                                    for grandchild in &child.child_tags {

                                        // Expect each param to be something like:
                                        // <div class="integer-param" id="x">

                                        if let Some(param_type) = self.parse_param_class(grandchild.class.as_deref()) {

                                            let param_name = grandchild.id.clone()
                                                .expect("Parameter must have an id attribute");

                                            // OpenParam
                                            self.tokens.push(
                                                HaplToken::new(
                                                    HaplTokenType::OpenParam {
                                                        name: param_name.clone(),
                                                        param_type,
                                                    },
                                                    Some(param_name.clone())
                                                )
                                            );

                                            // Walk param contents if needed
                                            for param_child in &grandchild.child_tags {
                                                self.walk(param_child);
                                            }

                                            // CloseParam
                                            self.tokens.push(
                                                HaplToken::new(
                                                    HaplTokenType::CloseParam {
                                                        name: param_name.clone(),
                                                    },
                                                    Some(param_name)
                                                )
                                            );
                                        }
                                    }

                                    self.tokens.push(
                                        HaplToken::new(
                                            HaplTokenType::CloseParams,
                                            Some("params".to_string())
                                        )
                                    );
                                }

                                "body" => {
                                    self.tokens.push(
                                        HaplToken::new(
                                            HaplTokenType::OpenFunctionBody,
                                            Some("function_body".to_string())
                                        )
                                    );

                                    for grandchild in &child.child_tags {
                                        self.walk(grandchild);
                                    }

                                    self.tokens.push(
                                        HaplToken::new(
                                            HaplTokenType::CloseFunctionBody,
                                            Some("function_body".to_string())
                                        )
                                    );
                                }

                                _ => {}
                            }
                        }
                    }

                    self.tokens.push(
                        HaplToken::new(
                            HaplTokenType::CloseFunction { name: name.clone() },
                            Some(name),
                        )
                    );

                    return;
                }

                // -----------------------------------------
                // Function call
                // <div class="function-name">
                // -----------------------------------------
                if tag.id.is_none() && tag.class.is_some() {
                    let function_name = class.clone();

                    // Prevent conflicts with reserved classes
                    let reserved = [
                        "+", "-", "*", "/",
                        "&&", "||", "!",
                        "equal", "not_equal",
                        "less", "less_equal",
                        "greater", "greater_equal",
                        "conditional", "if", "elif", "else",
                        "while", "for",
                        "params", "body", "args",
                    ];

                    if !reserved.contains(&function_name.as_str()) {

                        self.tokens.push(
                            HaplToken::new(
                                HaplTokenType::OpenFunctionCall {
                                    name: function_name.clone(),
                                },
                                Some(function_name.clone()),
                            )
                        );

                        for child in &tag.child_tags {
                            if let Some(child_class) = &child.class {
                                if child_class == "args" {
                                    self.tokens.push(
                                        HaplToken::new(
                                            HaplTokenType::OpenArgs,
                                            Some("args".to_string())
                                        )
                                    );

                                    for grandchild in &child.child_tags {
                                        self.walk(grandchild);
                                    }

                                    self.tokens.push(
                                        HaplToken::new(
                                            HaplTokenType::CloseArgs,
                                            Some("args".to_string())
                                        )
                                    );

                                    continue;
                                }
                            }

                            self.walk(child);
                        }

                        self.tokens.push(
                            HaplToken::new(
                                HaplTokenType::CloseFunctionCall {
                                    name: function_name.clone(),
                                },
                                Some(function_name),
                            )
                        );

                        return;
                    }
                }















            }
        }

        // -----------------------------------------
        // Print <p> tag
        // -----------------------------------------
        if tag.tag_type == "p" {
            self.tokens.push(HaplToken::open_print());

            for child in &tag.child_tags {
                self.walk(child);
            }

            self.tokens.push(HaplToken::close_print());
            return;
        }

        // -----------------------------------------
        // Variable tags <var>
        // -----------------------------------------
        if tag.tag_type == "var" {
            if let Some(class) = &tag.class {
                if let Some(id) = &tag.id {
                    // Variable declaration: <var class="integer" id="x">10</var>
                    let var_type = match class.as_str() {
                        "integer" => StaticType::Integer,
                        "double" => StaticType::Double,
                        "string" => StaticType::String,
                        "boolean" => StaticType::Boolean,
                        other => panic!("Unknown variable type '{}'", other),
                    };

                    self.tokens.push(HaplToken::new(
                        HaplTokenType::OpenVarDec {
                            var_type,
                            name: id.clone(),
                        },
                        Some(format!("{} {}", class, id)),
                    ));

                    for child in &tag.child_tags {
                        self.walk(child);
                    }

                    self.tokens.push(HaplToken::new(
                        HaplTokenType::CloseVarDec {
                            var_type,
                            name: id.clone(),
                        },
                        Some(format!("{} {}", class, id)),
                    ));
                    return;
                } else {
                    if tag.child_tags.is_empty(){
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
                        // ----------------------------
                        // Variable Assignment
                        // ----------------------------
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

                    return;
                }
            } else {
                panic!("<var> tag requires either an id for declaration or class for reference");
            }
        }

        // -----------------------------------------
        // <span> → literals
        // -----------------------------------------
        if tag.tag_type == "span" {
            if tag.child_tags.is_empty() && !tag.content.trim().is_empty() {
                let token = self.parse_literal(&tag.content, tag.class.as_deref());
                self.tokens.push(token);
            } else {
                for child in &tag.child_tags {
                    self.walk(child);
                }
            }
            return;
        }

        // -----------------------------------------
        // Structural HTML tags
        // -----------------------------------------
        let structural = ["html", "head", "body", "title"];

        if structural.contains(&tag.tag_type.as_str()) {
            self.tokens.push(HaplToken::open_html_tag(tag.tag_type.clone()));

            for child in &tag.child_tags {
                self.walk(child);
            }

            self.tokens.push(HaplToken::close_html_tag(tag.tag_type.clone()));
            return;
        }

        // -----------------------------------------
        // Generic leaf literal
        // -----------------------------------------
        if tag.child_tags.is_empty() && !tag.content.trim().is_empty() {
            let token = self.parse_literal(&tag.content, tag.class.as_deref());
            self.tokens.push(token);
        } else {
            for child in &tag.child_tags {
                self.walk(child);
            }
        }

    }

    pub fn print(&self) {
        for token in &self.tokens {
            match &token.token_type {

                // ========================
                // Operators
                // ========================
                HaplTokenType::OpenOperator { name } => {
                    println!("OpenOperator({}) -> {:?}", operator_to_str(*name), token.value);
                }
                HaplTokenType::CloseOperator { name } => {
                    println!("CloseOperator({}) -> {:?}", operator_to_str(*name), token.value);
                }

                // ========================
                // HTML
                // ========================
                HaplTokenType::OpenHtmlTag { name } => {
                    println!("OpenHtmlTag({}) -> {:?}", name, token.value);
                }
                HaplTokenType::CloseHtmlTag { name } => {
                    println!("CloseHtmlTag({}) -> {:?}", name, token.value);
                }

                // ========================
                // Literals
                // ========================
                HaplTokenType::Literal(lit) => {
                    println!("Literal({:?}) -> {:?}", lit, token.value);
                }

                // ========================
                // Variables
                // ========================
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

                // ========================
                // Print
                // ========================
                HaplTokenType::OpenPrint => {
                    println!("OpenPrint -> {:?}", token.value);
                }
                HaplTokenType::ClosePrint => {
                    println!("ClosePrint -> {:?}", token.value);
                }

                // ========================
                // Conditionals
                // ========================
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

                // ========================
                // Loops
                // ========================
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

                // ========================
                // Functions
                // ========================
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

                // ========================
                // Function Calls
                // ========================
                HaplTokenType::OpenFunctionCall { name } => {
                    println!("OpenFunctionCall({}) -> {:?}", name, token.value);
                }
                HaplTokenType::CloseFunctionCall { name } => {
                    println!("CloseFunctionCall({}) -> {:?}", name, token.value);
                }

                // ========================
                // Return
                // ========================
                HaplTokenType::OpenReturn => {
                    println!("OpenReturn -> {:?}", token.value);
                }
                HaplTokenType::CloseReturn => {
                    println!("CloseReturn -> {:?}", token.value);
                }

                // ========================
                // Args
                // ========================
                HaplTokenType::OpenArgs => {
                    println!("OpenArgs -> {:?}", token.value);
                }
                HaplTokenType::CloseArgs => {
                    println!("CloseArgs -> {:?}", token.value);
                }
            }
        }
    }

    pub fn tokens(&self) -> &[HaplToken] {
        &self.tokens
    }

    // -----------------------------------------
    // Helpers
    // -----------------------------------------

    fn get_arithmetic_tags(&self, operator: &str) -> (HaplToken, HaplToken) {
        let tag_type = match operator {
            "+" => LexerTagType::Add,
            "-" => LexerTagType::Subtract,
            "*" => LexerTagType::Multiply,
            "/" => LexerTagType::Divide,
            _ => panic!("Unknown operator: {}", operator),
        };

        (HaplToken::open_operator(tag_type), HaplToken::close_operator(tag_type))
    }

    fn get_boolean_tags(&self, logic_operator: &str) -> (HaplToken, HaplToken) {
        let tag_type = match logic_operator {
            "&&" => LexerTagType::And,
            "||" => LexerTagType::Or,
            "!" => LexerTagType::Not,
            _ => panic!("Unknown operator: {}", logic_operator),
        };

        (HaplToken::open_operator(tag_type), HaplToken::close_operator(tag_type))
    }

    fn get_comparison_tags(&self, operator: &str) -> (HaplToken, HaplToken) {
        let tag_type = match operator {
            "equal" => LexerTagType::Equal,
            "not_equal" => LexerTagType::NotEqual,
            "less" => LexerTagType::Less,
            "less_equal" => LexerTagType::LessEqual,
            "greater" => LexerTagType::Greater,
            "greater_equal" => LexerTagType::GreaterEqual,
            _ => panic!("Unknown comparison operator: {}", operator),
        };

        (
            HaplToken::open_operator(tag_type),
            HaplToken::close_operator(tag_type),
        )
    }

    fn parse_literal(&self, text: &str, class_type: Option<&str>) -> HaplToken {
        let trimmed = text.trim();

        let class = class_type.expect(
            "Static type required on <span>: expected 'integer', 'double', 'boolean' or 'string'",
        );

        match class {
            "integer" => trimmed
                .parse::<i64>()
                .map(HaplToken::integer)
                .unwrap_or_else(|_| panic!("Type error: value '{}' is not a valid integer", trimmed)),
            "double" => trimmed
                .parse::<f64>()
                .map(HaplToken::double)
                .unwrap_or_else(|_| panic!("Type error: value '{}' is not a valid double", trimmed)),
            "string" => {
                if !trimmed.starts_with('"') || !trimmed.ends_with('"') {
                    panic!(
                        "String literals must be enclosed in double quotes (\") but got '{}'",
                        trimmed
                    );
                }
                // Strip the quotes
                let inner = &trimmed[1..trimmed.len() - 1];
                HaplToken::string(inner.to_string()) 
            }

            "boolean" => {
                match trimmed {
                    "true" => HaplToken::boolean(true),
                    "false" => HaplToken::boolean(false),
                    _ => panic!(
                        "Type error: value '{}' is not a valid boolean (expected true or false)",
                        trimmed
                    ),
                }
            }

            other => panic!("Unknown static type '{}'. Expected 'integer', 'double', 'boolean', or 'string'", other),
        }
    }

    fn parse_function_class(&self, class: &str) -> Option<StaticType> {
        if let Some(type_part) = class.strip_suffix("-function") {
            return Some(match type_part {
                "integer" => StaticType::Integer,
                "double" => StaticType::Double,
                "string" => StaticType::String,
                "boolean" => StaticType::Boolean,
                "void" => StaticType::Void,
                other => panic!("Unknown function return type '{}'", other),
            });
        }
        None
    }

    fn parse_param_class(&self, class: Option<&str>) -> Option<StaticType> {
        let class = class?;

        // Must end with "-param"
        if !class.ends_with("-param") {
            return None;
        }

        // Strip "-param"
        let type_part = class.strip_suffix("-param")?;

        match type_part {
            "integer" => Some(StaticType::Integer),
            "float"   => Some(StaticType::Double),
            "string"  => Some(StaticType::String),
            "boolean" => Some(StaticType::Boolean),
            "void"    => Some(StaticType::Void),

            _ => None,
        }
    }

}

fn operator_to_str(op: LexerTagType) -> &'static str {
    match op {
        LexerTagType::Add => "+",
        LexerTagType::Subtract => "-",
        LexerTagType::Multiply => "*",
        LexerTagType::Divide => "/",
        LexerTagType::And => "&&",
        LexerTagType::Or => "||",
        LexerTagType::Not => "!",
        LexerTagType::Equal => "==",
        LexerTagType::NotEqual => "!=",
        LexerTagType::Less => "<",
        LexerTagType::LessEqual => "<=",
        LexerTagType::Greater => ">",
        LexerTagType::GreaterEqual => ">=",
    }
}
