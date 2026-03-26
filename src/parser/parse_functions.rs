use super::*;
use crate::ast::{Expr, StaticType};
use crate::error::{ErrorCode, HaplError, parser_err};
use crate::lexer::HaplTokenType;

impl HaplParser {
    // --------------------------------------------------
    // Pass-1 pre-scan: register all function signatures
    // so forward calls can be type-checked in pass 2.
    // --------------------------------------------------
    pub(super) fn prescan_function_signatures(&mut self) {
        let mut i = 0usize;
        while i < self.tokens.len() {
            if let HaplTokenType::OpenFunction { ref name, return_type } =
                self.tokens[i].token_type.clone()
            {
                let fn_name = name.clone();
                let mut params: Vec<(String, StaticType)> = Vec::new();
                let mut j = i + 1;

                while j < self.tokens.len()
                    && !matches!(self.tokens[j].token_type, HaplTokenType::OpenParams)
                {
                    j += 1;
                }
                j += 1; // skip OpenParams

                while j < self.tokens.len()
                    && !matches!(self.tokens[j].token_type, HaplTokenType::CloseParams)
                {
                    if let HaplTokenType::OpenParam { ref name, param_type } =
                        self.tokens[j].token_type.clone()
                    {
                        params.push((name.clone(), param_type));
                    }
                    j += 1;
                }

                self.function_signatures
                    .entry(fn_name)
                    .or_insert(FunctionSignature { params, return_type });
            }
            i += 1;
        }
    }

    // --------------------------------------------------
    // Function declaration
    // --------------------------------------------------
    pub(super) fn parse_function(&mut self) -> Result<Expr, HaplError> {
        let (name, return_type) = match self.current_token() {
            HaplTokenType::OpenFunction { name, return_type } => (name.clone(), return_type.clone()),
            other => {
                return Err(parser_err(
                    ErrorCode::UnexpectedToken,
                    format!("expected function declaration but found '{:?}'", other),
                ));
            }
        };
        self.advance(); // consume OpenFunction

        // Params
        if !matches!(self.current_token(), HaplTokenType::OpenParams) {
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!(
                    "expected <params> in function '{}' but found '{:?}'",
                    name,
                    self.current_token()
                ),
            )
            .with_hint(format!(
                "add a <div class=\"params\"> block inside function '{}'",
                name
            )));
        }
        self.advance(); // consume OpenParams

        let mut params = Vec::new();

        while !matches!(self.current_token(), HaplTokenType::CloseParams) {
            match self.current_token() {
                HaplTokenType::OpenParam { name: param_name, param_type } => {
                    let param_name = param_name.clone();
                    let param_type_copy = param_type;
                    self.advance(); // consume OpenParam

                    if !matches!(
                        self.current_token(),
                        HaplTokenType::CloseParam { name: ref n } if n == &param_name
                    ) {
                        return Err(parser_err(
                            ErrorCode::TagNotClosed,
                            format!("parameter '{}' was never closed", param_name),
                        )
                        .with_hint(format!(
                            "add a matching </div> closing tag for parameter '{}'",
                            param_name
                        )));
                    }
                    self.advance(); // consume CloseParam
                    params.push((param_name, param_type_copy));
                }
                other => {
                    return Err(parser_err(
                        ErrorCode::UnexpectedToken,
                        format!("unexpected token '{:?}' inside <params>", other),
                    )
                    .with_hint(
                        "params may only contain parameter declarations, \
                         e.g. <div class=\"integer-param\" id=\"x\"></div>",
                    ));
                }
            }
        }
        self.advance(); // consume CloseParams

        // Register signature before parsing body so recursive calls type-check correctly
        self.function_signatures.insert(
            name.clone(),
            FunctionSignature {
                params: params.clone(),
                return_type: return_type.clone(),
            },
        );

        // Body
        if !matches!(self.current_token(), HaplTokenType::OpenFunctionBody) {
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!(
                    "expected <body> in function '{}' but found '{:?}'",
                    name,
                    self.current_token()
                ),
            )
            .with_hint(format!(
                "add a <div class=\"body\"> block inside function '{}'",
                name
            )));
        }
        self.advance(); // consume OpenFunctionBody

        self.push_scope();

        let prev_fn_return_type = self.current_fn_return_type.replace(return_type.clone());

        for (param_name, param_type) in &params {
            self.declare_var(param_name.clone(), param_type.clone())?;
        }

        let mut body = Vec::new();
        while !self.is_at_end()
            && !matches!(self.current_token(), HaplTokenType::CloseFunctionBody)
        {
            body.push(self.parse_statement()?);
        }

        self.current_fn_return_type = prev_fn_return_type;

        if self.is_at_end() {
            self.pop_scope();
            return Err(parser_err(
                ErrorCode::TagNotClosed,
                format!("body of function '{}' was never closed", name),
            )
            .with_hint("add </div> to close the <div class=\"body\"> block"));
        }

        self.pop_scope();
        self.advance(); // consume CloseFunctionBody

        if !matches!(
            self.current_token(),
            HaplTokenType::CloseFunction { name: ref n } if n == &name
        ) {
            return Err(parser_err(
                ErrorCode::TagNotClosed,
                format!("function '{}' was never closed", name),
            )
            .with_hint(format!(
                "add a matching closing tag for function '{}'",
                name
            )));
        }
        self.advance(); // consume CloseFunction

        Ok(Expr::FunctionDeclaration {
            name,
            return_type,
            params,
            body,
        })
    }

    // --------------------------------------------------
    // Function call
    // --------------------------------------------------
    pub(super) fn parse_function_call(&mut self) -> Result<Expr, HaplError> {
        let name = match self.current_token() {
            HaplTokenType::OpenFunctionCall { name } => name.clone(),
            other => {
                return Err(parser_err(
                    ErrorCode::UnexpectedToken,
                    format!("expected function call but found '{:?}'", other),
                ));
            }
        };
        self.advance(); // consume OpenFunctionCall

        let mut args = Vec::new();

        if matches!(self.current_token(), HaplTokenType::OpenArgs) {
            self.advance(); // consume OpenArgs

            while !self.is_at_end() && !matches!(self.current_token(), HaplTokenType::CloseArgs) {
                args.push(self.parse_expression()?);
            }

            if self.is_at_end() {
                return Err(parser_err(
                    ErrorCode::TagNotClosed,
                    format!(
                        "argument list for function call '{}' was never closed",
                        name
                    ),
                )
                .with_hint(
                    "add a matching </div> to close the <div class=\"args\"> block",
                ));
            }
            self.advance(); // consume CloseArgs
        }

        if !matches!(
            self.current_token(),
            HaplTokenType::CloseFunctionCall { name: ref n } if n == &name
        ) {
            return Err(parser_err(
                ErrorCode::TagNotClosed,
                format!("function call '{}' was never closed", name),
            )
            .with_hint(format!(
                "add a matching closing tag for the '{}' function call",
                name
            )));
        }
        self.advance(); // consume CloseFunctionCall

        // Static type-checking against the registered signature
        if let Some(sig) = self.function_signatures.get(&name).cloned() {
            if args.len() != sig.params.len() {
                return Err(parser_err(
                    ErrorCode::WrongArgCount,
                    format!(
                        "function '{}' expects {} argument{}, but {} {} provided",
                        name,
                        sig.params.len(),
                        if sig.params.len() == 1 { "" } else { "s" },
                        args.len(),
                        if args.len() == 1 { "was" } else { "were" },
                    ),
                )
                .with_hint(format!(
                    "'{}' signature: ({})",
                    name,
                    sig.params
                        .iter()
                        .map(|(n, t)| format!("{}: {:?}", n, t))
                        .collect::<Vec<_>>()
                        .join(", ")
                )));
            }

            for (i, (arg_expr, (param_name, param_type))) in
                args.iter().zip(sig.params.iter()).enumerate()
            {
                if let Some(arg_type) = self.infer_type(arg_expr) {
                    if arg_type != *param_type {
                        return Err(parser_err(
                            ErrorCode::TypeMismatch,
                            format!(
                                "argument {} ('{}') of call to '{}': expected {:?}, got {:?}",
                                i + 1,
                                param_name,
                                name,
                                param_type,
                                arg_type,
                            ),
                        )
                        .with_hint(format!(
                            "parameter '{}' is declared as {:?} — pass a {:?} value",
                            param_name, param_type, param_type
                        )));
                    }
                }
            }
        }

        Ok(Expr::FunctionCall { name, args })
    }

    // --------------------------------------------------
    // Return statement
    // --------------------------------------------------
    pub(super) fn parse_return(&mut self) -> Result<Expr, HaplError> {
        self.advance(); // consume OpenReturn

        let value = self.parse_expression()?;

        if self.is_at_end() || !matches!(self.current_token(), HaplTokenType::CloseReturn) {
            return Err(
                parser_err(ErrorCode::TagNotClosed, "return statement was never closed")
                    .with_hint(
                        "add a matching closing tag for the <div class=\"return\"> block",
                    ),
            );
        }
        self.advance(); // consume CloseReturn

        if let Some(expected) = &self.current_fn_return_type {
            if *expected == StaticType::Void {
                return Err(parser_err(
                    ErrorCode::TypeMismatch,
                    "void function cannot return a value",
                )
                .with_hint(
                    "declare the function with a non-void return type, or remove the return statement",
                ));
            }

            if let Some(actual) = self.infer_type(&value) {
                if actual != *expected {
                    return Err(parser_err(
                        ErrorCode::TypeMismatch,
                        format!(
                            "return type mismatch: function declared as {:?} but returns {:?}",
                            expected, actual
                        ),
                    )
                    .with_hint(format!(
                        "change the return value to a {:?} expression, or update the function's return type",
                        expected
                    )));
                }
            }
        }

        Ok(Expr::Return {
            value: Some(Box::new(value)),
        })
    }
}
