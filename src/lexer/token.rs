use crate::ast::{LiteralValue, StaticType};
use super::tag_type::{LexerTagType, LoopType};
use super::helpers::operator_to_str;

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

    // Length
    OpenLength,
    CloseLength,

    // HTTP requests
    OpenHttpGet { name: String },
    CloseHttpGet { name: String },
    OpenHttpPost { name: String },
    CloseHttpPost { name: String },
    OpenHttpUrl,
    CloseHttpUrl,
    OpenHttpBody,
    CloseHttpBody,

    // User input
    OpenInput,
    CloseInput,

    // Server
    OpenServer { port: u16 },
    CloseServer,
    OpenEndpointGet { path: String },
    CloseEndpointGet,
    OpenEndpointPost { path: String },
    CloseEndpointPost,
    OpenHandler,
    CloseHandler,
    OpenRespond,
    CloseRespond,
}

#[derive(Debug, Clone)]
pub struct HaplToken {
    pub token_type: HaplTokenType,
    pub value: Option<String>,
}

impl HaplToken {
    pub fn new(token_type: HaplTokenType, value: Option<String>) -> Self {
        Self { token_type, value }
    }

    pub fn open_operator(name: LexerTagType) -> Self {
        Self::new(
            HaplTokenType::OpenOperator { name },
            Some(operator_to_str(name).to_string()),
        )
    }

    pub fn close_operator(name: LexerTagType) -> Self {
        Self::new(
            HaplTokenType::CloseOperator { name },
            Some(operator_to_str(name).to_string()),
        )
    }

    pub fn open_html_tag(name: String) -> Self {
        Self::new(HaplTokenType::OpenHtmlTag { name: name.clone() }, Some(name))
    }

    pub fn close_html_tag(name: String) -> Self {
        Self::new(HaplTokenType::CloseHtmlTag { name: name.clone() }, Some(name))
    }

    pub fn string(content: String) -> Self {
        Self::new(
            HaplTokenType::Literal(LiteralValue::String(content.clone())),
            Some(content),
        )
    }

    pub fn double(double_value: f64) -> Self {
        Self::new(
            HaplTokenType::Literal(LiteralValue::Double(double_value)),
            Some(double_value.to_string()),
        )
    }

    pub fn integer(integer_value: i64) -> Self {
        Self::new(
            HaplTokenType::Literal(LiteralValue::Integer(integer_value)),
            Some(integer_value.to_string()),
        )
    }

    pub fn boolean(boolean_value: bool) -> Self {
        Self::new(
            HaplTokenType::Literal(LiteralValue::Boolean(boolean_value)),
            Some(boolean_value.to_string()),
        )
    }

    pub fn open_print() -> Self {
        Self::new(HaplTokenType::OpenPrint, Some("print".to_string()))
    }

    pub fn close_print() -> Self {
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

    pub fn open_http_get(name: String) -> Self {
        Self::new(HaplTokenType::OpenHttpGet { name: name.clone() }, Some(name))
    }

    pub fn close_http_get(name: String) -> Self {
        Self::new(HaplTokenType::CloseHttpGet { name: name.clone() }, Some(name))
    }

    pub fn open_http_post(name: String) -> Self {
        Self::new(HaplTokenType::OpenHttpPost { name: name.clone() }, Some(name))
    }

    pub fn close_http_post(name: String) -> Self {
        Self::new(HaplTokenType::CloseHttpPost { name: name.clone() }, Some(name))
    }

    pub fn open_http_url() -> Self {
        Self::new(HaplTokenType::OpenHttpUrl, Some("url".to_string()))
    }

    pub fn close_http_url() -> Self {
        Self::new(HaplTokenType::CloseHttpUrl, Some("url".to_string()))
    }

    pub fn open_http_body() -> Self {
        Self::new(HaplTokenType::OpenHttpBody, Some("body".to_string()))
    }

    pub fn close_http_body() -> Self {
        Self::new(HaplTokenType::CloseHttpBody, Some("body".to_string()))
    }

    pub fn open_input() -> Self {
        Self::new(HaplTokenType::OpenInput, Some("input".to_string()))
    }

    pub fn close_input() -> Self {
        Self::new(HaplTokenType::CloseInput, Some("input".to_string()))
    }

    pub fn open_server(port: u16) -> Self {
        Self::new(HaplTokenType::OpenServer { port }, Some(port.to_string()))
    }

    pub fn close_server() -> Self {
        Self::new(HaplTokenType::CloseServer, Some("server".to_string()))
    }

    pub fn open_endpoint_get(path: String) -> Self {
        Self::new(HaplTokenType::OpenEndpointGet { path: path.clone() }, Some(path))
    }

    pub fn close_endpoint_get() -> Self {
        Self::new(HaplTokenType::CloseEndpointGet, Some("endpoint-get".to_string()))
    }

    pub fn open_endpoint_post(path: String) -> Self {
        Self::new(HaplTokenType::OpenEndpointPost { path: path.clone() }, Some(path))
    }

    pub fn close_endpoint_post() -> Self {
        Self::new(HaplTokenType::CloseEndpointPost, Some("endpoint-post".to_string()))
    }

    pub fn open_handler() -> Self {
        Self::new(HaplTokenType::OpenHandler, Some("handler".to_string()))
    }

    pub fn close_handler() -> Self {
        Self::new(HaplTokenType::CloseHandler, Some("handler".to_string()))
    }

    pub fn open_respond() -> Self {
        Self::new(HaplTokenType::OpenRespond, Some("respond".to_string()))
    }

    pub fn close_respond() -> Self {
        Self::new(HaplTokenType::CloseRespond, Some("respond".to_string()))
    }
}
