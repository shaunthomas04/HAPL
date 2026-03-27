use super::*;
use crate::ast::LiteralValue;
use crate::error::{ErrorCode, HaplError, runtime_err};

impl Interpreter {
    pub(super) fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub(super) fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub(super) fn current_scope(&mut self) -> &mut HashMap<String, LiteralValue> {
        self.scopes
            .last_mut()
            .expect("scope stack is empty — this is a bug in the interpreter")
    }

    pub(super) fn lookup(&self, name: &str) -> Result<LiteralValue, HaplError> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Ok(val.clone());
            }
        }
        Err(runtime_err(
            ErrorCode::VariableNotFound,
            format!("variable '{}' not found", name),
        )
        .with_hint(format!(
            "make sure '{}' is declared before it is used",
            name
        )))
    }

    pub(super) fn assign(&mut self, name: &str, val: LiteralValue) -> Result<(), HaplError> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), val);
                return Ok(());
            }
        }
        Err(runtime_err(
            ErrorCode::AssignBeforeDeclare,
            format!(
                "cannot assign to '{}' — variable has not been declared",
                name
            ),
        )
        .with_hint(format!(
            "declare '{}' with <var class=\"<type>\" id=\"{}\"> before assigning to it",
            name, name
        )))
    }
}
