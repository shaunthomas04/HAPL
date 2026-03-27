use crate::ast::{LiteralValue, Operator, StaticType};
use indexmap::IndexMap;

/// Short operator name used in runtime error messages.
pub(super) fn op_name(op: &Operator) -> &'static str {
    match op {
        Operator::Add          => "+",
        Operator::Subtract     => "-",
        Operator::Multiply     => "*",
        Operator::Divide       => "/",
        Operator::Modulo       => "%",
        Operator::And          => "&&",
        Operator::Or           => "||",
        Operator::Not          => "!",
        Operator::Equal        => "equal",
        Operator::NotEqual     => "not_equal",
        Operator::Less         => "less",
        Operator::LessEqual    => "less_equal",
        Operator::Greater      => "greater",
        Operator::GreaterEqual => "greater_equal",
    }
}

/// Extension trait so LiteralValue can describe its own type in error messages.
pub(super) trait TypeName {
    fn type_name(&self) -> &'static str;
}

impl TypeName for LiteralValue {
    fn type_name(&self) -> &'static str {
        match self {
            LiteralValue::Integer(_)  => "Integer",
            LiteralValue::Double(_)   => "Double",
            LiteralValue::String(_)   => "String",
            LiteralValue::Boolean(_)  => "Boolean",
            LiteralValue::List { .. } => "List",
            LiteralValue::Map { .. }  => "Map",
        }
    }
}

pub(super) fn json_to_literal(val: serde_json::Value) -> LiteralValue {
    match val {
        serde_json::Value::String(s) => LiteralValue::String(s),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                LiteralValue::Integer(i)
            } else {
                LiteralValue::Double(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::Bool(b) => LiteralValue::Boolean(b),
        serde_json::Value::Object(m) => {
            let mut entries = IndexMap::new();
            for (k, v) in m {
                entries.insert(k, json_to_literal(v));
            }
            LiteralValue::Map { entries }
        }
        serde_json::Value::Array(arr) => {
            let elements: Vec<LiteralValue> = arr.into_iter().map(json_to_literal).collect();
            // Infer elem_type from the first element; fall back to String for empty/mixed arrays.
            let elem_type = elements.first().map(|v| match v {
                LiteralValue::Integer(_) => StaticType::Integer,
                LiteralValue::Double(_)  => StaticType::Double,
                LiteralValue::Boolean(_) => StaticType::Boolean,
                _                        => StaticType::String,
            }).unwrap_or(StaticType::String);
            LiteralValue::List { elem_type, elements }
        }
        serde_json::Value::Null => LiteralValue::String("null".to_string()),
    }
}

/// Check whether a runtime value is compatible with a declared element type.
/// Used to enforce list element types on declaration, push, and index-assign.
pub(super) fn matches_type(val: &LiteralValue, expected: &StaticType) -> bool {
    match (val, expected) {
        (LiteralValue::Integer(_), StaticType::Integer)  => true,
        (LiteralValue::Double(_),  StaticType::Double)   => true,
        (LiteralValue::String(_),  StaticType::String)   => true,
        (LiteralValue::Boolean(_), StaticType::Boolean)  => true,
        (LiteralValue::Map { .. }, StaticType::Map)      => true,
        _ => false,
    }
}
pub(super) fn literal_to_json(val: &LiteralValue) -> Option<serde_json::Value> {
    match val {
        LiteralValue::String(s) => Some(serde_json::Value::String(s.clone())),
        LiteralValue::Integer(n) => Some(serde_json::json!(n)),
        LiteralValue::Double(f) => Some(serde_json::json!(f)),
        LiteralValue::Boolean(b) => Some(serde_json::Value::Bool(*b)),
        LiteralValue::Map { entries } => {
            let mut map = serde_json::Map::new();
            for (k, v) in entries {
                map.insert(k.clone(), literal_to_json(v)?);
            }
            Some(serde_json::Value::Object(map))
        }
        LiteralValue::List { .. } => None,
    }
}