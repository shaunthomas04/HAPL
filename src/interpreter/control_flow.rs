use crate::ast::LiteralValue;
use crate::error::HaplError;

#[derive(Debug, Clone)]
pub(super) enum ControlFlow {
    Value(LiteralValue),
    Return(LiteralValue),
    /// Carries a runtime error up the call stack so it can be reported at the
    /// top level without unwinding via panic.
    Error(HaplError),
}

// --------------------------------------------------
// Macro: bubble a Result<LiteralValue, ControlFlow> up.
// On Ok returns the value; on Err immediately returns the ControlFlow
// (which is either a Return or an Error).
// --------------------------------------------------
macro_rules! bubble {
    ($e:expr) => {
        match $e {
            Ok(v)   => v,
            Err(cf) => return cf,
        }
    };
}
pub(super) use bubble;
