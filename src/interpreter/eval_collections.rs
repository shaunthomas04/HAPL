use super::*;
use crate::ast::{Expr, LiteralValue};
use crate::error::{ErrorCode, runtime_err};
use control_flow::{ControlFlow, bubble};

use indexmap::IndexMap;

impl Interpreter {
    pub(super) fn eval_list(&mut self, expr: &Expr) -> ControlFlow {
        match expr {
            // -------------------------
            // List Declaration
            // -------------------------
            Expr::ListDeclaration { name, elem_type, elements } => {
                let mut evaled = Vec::new();
                for (i, elem) in elements.iter().enumerate() {
                    let val = bubble!(self.eval_value(elem));
                    if !helpers::matches_type(&val, elem_type) {
                        return ControlFlow::Error(
                            runtime_err(
                                ErrorCode::ListElementTypeMismatch,
                                format!(
                                    "list '{}' element {} has wrong type: expected {:?}, got {:?}",
                                    name, i, elem_type, val.type_name()
                                ),
                            )
                            .with_hint(format!(
                                "all elements in '{}' must be of type {:?}",
                                name, elem_type
                            )),
                        );
                    }
                    evaled.push(val);
                }
                let list = LiteralValue::List {
                    elem_type: elem_type.clone(),
                    elements: evaled,
                };
                self.current_scope().insert(name.clone(), list.clone());
                ControlFlow::Value(list)
            }

            // -------------------------
            // List Access
            // -------------------------
            Expr::ListAccess { list, index } => {
                let list_val  = bubble!(self.eval_value(list));
                let index_val = bubble!(self.eval_value(index));

                let idx = match index_val {
                    LiteralValue::Integer(i) => i,
                    other => return ControlFlow::Error(
                        runtime_err(
                            ErrorCode::ListIndexNotInt,
                            format!("list index must be Integer, got {:?}", other.type_name()),
                        )
                        .with_hint("use an integer as the list index"),
                    ),
                };

                match list_val {
                    LiteralValue::List { elements, .. } => {
                        if idx < 0 || idx as usize >= elements.len() {
                            return ControlFlow::Error(
                                runtime_err(
                                    ErrorCode::IndexOutOfBounds,
                                    format!(
                                        "index {} is out of bounds (length {})",
                                        idx,
                                        elements.len()
                                    ),
                                )
                                .with_hint(
                                    "make sure the index is within the list's length",
                                ),
                            );
                        }
                        ControlFlow::Value(elements[idx as usize].clone())
                    }
                    other => ControlFlow::Error(
                        runtime_err(
                            ErrorCode::IndexOutOfBounds,
                            format!("cannot index into {:?}", other.type_name()),
                        )
                        .with_hint("only lists can be indexed"),
                    ),
                }
            }

            // -------------------------
            // List Assign
            // -------------------------
            Expr::ListAssign { name, index, value } => {
                let index_val = bubble!(self.eval_value(index));
                let new_val   = bubble!(self.eval_value(value));

                let idx = match index_val {
                    LiteralValue::Integer(i) => i,
                    other => return ControlFlow::Error(
                        runtime_err(
                            ErrorCode::ListIndexNotInt,
                            format!("list index must be Integer, got {:?}", other.type_name()),
                        )
                        .with_hint("use an integer as the list index"),
                    ),
                };

                let list = match self.lookup(name) {
                    Ok(v)  => v,
                    Err(e) => return ControlFlow::Error(e),
                };

                match list {
                    LiteralValue::List { elem_type, mut elements } => {
                        if !helpers::matches_type(&new_val, &elem_type) {
                            return ControlFlow::Error(
                                runtime_err(
                                    ErrorCode::ListElementTypeMismatch,
                                    format!(
                                        "cannot assign {:?} to list '{}' of type {:?}",
                                        new_val.type_name(), name, elem_type
                                    ),
                                )
                                .with_hint(format!(
                                    "list '{}' only holds {:?} values",
                                    name, elem_type
                                )),
                            );
                        }
                        if idx < 0 || idx as usize >= elements.len() {
                            return ControlFlow::Error(
                                runtime_err(
                                    ErrorCode::IndexOutOfBounds,
                                    format!("index {} is out of bounds (length {})", idx, elements.len()),
                                )
                                .with_hint("make sure the index is within the list's length"),
                            );
                        }
                        elements[idx as usize] = new_val;
                        let updated = LiteralValue::List { elem_type, elements };
                        if let Err(e) = self.assign(name, updated.clone()) {
                            return ControlFlow::Error(e);
                        }
                        ControlFlow::Value(updated)
                    }
                    other => ControlFlow::Error(
                        runtime_err(
                            ErrorCode::IndexOutOfBounds,
                            format!("cannot index into {:?}", other.type_name()),
                        )
                        .with_hint("only lists can be index-assigned"),
                    ),
                }
            }

            // -------------------------
            // List Push
            // -------------------------
            Expr::ListPush { name, value } => {
                let new_val = bubble!(self.eval_value(value));

                let list = match self.lookup(name) {
                    Ok(v)  => v,
                    Err(e) => return ControlFlow::Error(e),
                };

                match list {
                    LiteralValue::List { elem_type, mut elements } => {
                        if !helpers::matches_type(&new_val, &elem_type) {
                            return ControlFlow::Error(
                                runtime_err(
                                    ErrorCode::ListElementTypeMismatch,
                                    format!(
                                        "cannot push {:?} to list '{}' of type {:?}",
                                        new_val.type_name(), name, elem_type
                                    ),
                                )
                                .with_hint(format!(
                                    "list '{}' only holds {:?} values",
                                    name, elem_type
                                )),
                            );
                        }
                        elements.push(new_val);
                        let updated = LiteralValue::List { elem_type, elements };
                        if let Err(e) = self.assign(name, updated.clone()) {
                            return ControlFlow::Error(e);
                        }
                        ControlFlow::Value(updated)
                    }
                    other => ControlFlow::Error(
                        runtime_err(
                            ErrorCode::IndexOutOfBounds,
                            format!("cannot push to {:?}", other.type_name()),
                        )
                        .with_hint("only lists support push"),
                    ),
                }
            }

            // -------------------------
            // List Pop
            // -------------------------
            Expr::ListPop { name } => {
                let list = match self.lookup(name) {
                    Ok(v)  => v,
                    Err(e) => return ControlFlow::Error(e),
                };

                match list {
                    LiteralValue::List { elem_type, mut elements } => {
                        if elements.is_empty() {
                            return ControlFlow::Error(
                                runtime_err(
                                    ErrorCode::EmptyList,
                                    format!(
                                        "cannot pop from empty list '{}'",
                                        name
                                    ),
                                )
                                .with_hint(
                                    "check that the list has elements before popping",
                                ),
                            );
                        }
                        let popped = elements.pop().unwrap();
                        let updated = LiteralValue::List { elem_type, elements };
                        if let Err(e) = self.assign(name, updated) {
                            return ControlFlow::Error(e);
                        }
                        ControlFlow::Value(popped)
                    }
                    other => ControlFlow::Error(
                        runtime_err(
                            ErrorCode::EmptyList,
                            format!("cannot pop from {:?}", other.type_name()),
                        )
                        .with_hint("only lists support pop"),
                    ),
                }
            }

            _ => unreachable!("eval_list called on non-list expr"),
        }
    }

    pub(super) fn eval_map(&mut self, expr: &Expr) -> ControlFlow {
        match expr {
            // -------------------------
            // Map Declaration
            // -------------------------
            Expr::MapDeclaration { name, entries } => {
                let mut evaled = IndexMap::new();
                for (key, value_expr) in entries {
                    let val = bubble!(self.eval_value(value_expr));
                    evaled.insert(key.clone(), val);
                }
                let map = LiteralValue::Map { entries: evaled };
                if !name.is_empty() {
                    self.current_scope().insert(name.clone(), map.clone());
                }
                ControlFlow::Value(map)
            }

            // -------------------------
            // Map Get
            // -------------------------
            Expr::MapGet { map, key } => {
                let map_val = bubble!(self.eval_value(map));
                let key_val = bubble!(self.eval_value(key));

                let key_str = match key_val {
                    LiteralValue::String(s) => s,
                    other => return ControlFlow::Error(
                        runtime_err(
                            ErrorCode::BadOperatorTypes,
                            format!(
                                "map key must be a String, got {:?}",
                                other.type_name()
                            ),
                        )
                        .with_hint("map keys must always be strings"),
                    ),
                };

                match map_val {
                    LiteralValue::Map { entries } => match entries.get(&key_str) {
                        Some(val) => ControlFlow::Value(val.clone()),
                        None => ControlFlow::Error(
                            runtime_err(
                                ErrorCode::VariableNotFound,
                                format!("key '{}' not found in map", key_str),
                            )
                            .with_hint(format!(
                                "use map-contains to check if '{}' exists before accessing it",
                                key_str
                            )),
                        ),
                    },
                    other => ControlFlow::Error(
                        runtime_err(
                            ErrorCode::BadOperatorTypes,
                            format!("cannot use map-get on {:?}", other.type_name()),
                        )
                        .with_hint("map-get can only be used on a map variable"),
                    ),
                }
            }

            // -------------------------
            // Map Set
            // -------------------------
            Expr::MapSet { name, key, value } => {
                let key_val = bubble!(self.eval_value(key));
                let new_val = bubble!(self.eval_value(value));

                let key_str = match key_val {
                    LiteralValue::String(s) => s,
                    other => return ControlFlow::Error(
                        runtime_err(
                            ErrorCode::BadOperatorTypes,
                            format!(
                                "map key must be a String, got {:?}",
                                other.type_name()
                            ),
                        )
                        .with_hint("map keys must always be strings"),
                    ),
                };

                let map = match self.lookup(name) {
                    Ok(v)  => v,
                    Err(e) => return ControlFlow::Error(e),
                };

                match map {
                    LiteralValue::Map { mut entries } => {
                        entries.insert(key_str, new_val);
                        let updated = LiteralValue::Map { entries };
                        if let Err(e) = self.assign(name, updated.clone()) {
                            return ControlFlow::Error(e);
                        }
                        ControlFlow::Value(updated)
                    }
                    other => ControlFlow::Error(
                        runtime_err(
                            ErrorCode::BadOperatorTypes,
                            format!("cannot use map-set on {:?}", other.type_name()),
                        )
                        .with_hint("map-set can only be used on a map variable"),
                    ),
                }
            }

            // -------------------------
            // Map Remove
            // -------------------------
            Expr::MapRemove { name, key } => {
                let key_val = bubble!(self.eval_value(key));

                let key_str = match key_val {
                    LiteralValue::String(s) => s,
                    other => return ControlFlow::Error(
                        runtime_err(
                            ErrorCode::BadOperatorTypes,
                            format!(
                                "map key must be a String, got {:?}",
                                other.type_name()
                            ),
                        )
                        .with_hint("map keys must always be strings"),
                    ),
                };

                let map = match self.lookup(name) {
                    Ok(v)  => v,
                    Err(e) => return ControlFlow::Error(e),
                };

                match map {
                    LiteralValue::Map { mut entries } => {
                        if !entries.contains_key(&key_str) {
                            return ControlFlow::Error(
                                runtime_err(
                                    ErrorCode::VariableNotFound,
                                    format!(
                                        "key '{}' not found in map '{}'",
                                        key_str, name
                                    ),
                                )
                                .with_hint(format!(
                                    "use map-contains to check if '{}' exists before removing it",
                                    key_str
                                )),
                            );
                        }
                        entries.shift_remove(&key_str);
                        let updated = LiteralValue::Map { entries };
                        if let Err(e) = self.assign(name, updated.clone()) {
                            return ControlFlow::Error(e);
                        }
                        ControlFlow::Value(updated)
                    }
                    other => ControlFlow::Error(
                        runtime_err(
                            ErrorCode::BadOperatorTypes,
                            format!("cannot use map-remove on {:?}", other.type_name()),
                        )
                        .with_hint("map-remove can only be used on a map variable"),
                    ),
                }
            }

            // -------------------------
            // Map Contains
            // -------------------------
            Expr::MapContains { map, key } => {
                let map_val = bubble!(self.eval_value(map));
                let key_val = bubble!(self.eval_value(key));

                let key_str = match key_val {
                    LiteralValue::String(s) => s,
                    other => return ControlFlow::Error(
                        runtime_err(
                            ErrorCode::BadOperatorTypes,
                            format!(
                                "map key must be a String, got {:?}",
                                other.type_name()
                            ),
                        )
                        .with_hint("map keys must always be strings"),
                    ),
                };

                match map_val {
                    LiteralValue::Map { entries } => ControlFlow::Value(
                        LiteralValue::Boolean(entries.contains_key(&key_str)),
                    ),
                    other => ControlFlow::Error(
                        runtime_err(
                            ErrorCode::BadOperatorTypes,
                            format!(
                                "cannot use map-contains on {:?}",
                                other.type_name()
                            ),
                        )
                        .with_hint("map-contains can only be used on a map variable"),
                    ),
                }
            }

            _ => unreachable!("eval_map called on non-map expr"),
        }
    }
}
