use super::*;
use crate::ast::{Expr, StaticType};
use crate::error::{ErrorCode, HaplError, parser_err};
use crate::lexer::HaplTokenType;

impl HaplParser {
    /// Entry point called from `parse_expression` for all list/map tokens.
    pub(super) fn parse_collection_expression(&mut self) -> Result<Expr, HaplError> {
        match self.current_token() {
            HaplTokenType::OpenListDec { .. }     => self.parse_list_dec(),
            HaplTokenType::OpenListAccess         => self.parse_list_access(),
            HaplTokenType::OpenListAssign { .. }  => self.parse_list_assign(),
            HaplTokenType::OpenListPush { .. }    => self.parse_list_push(),
            HaplTokenType::OpenListPop { .. }     => self.parse_list_pop(),
            HaplTokenType::OpenMapDec { .. }      => self.parse_map_dec(),
            HaplTokenType::OpenMapGet             => self.parse_map_get(),
            HaplTokenType::OpenMapContains        => self.parse_map_contains(),
            HaplTokenType::OpenMapSet { .. }      => self.parse_map_set(),
            HaplTokenType::OpenMapRemove { .. }   => self.parse_map_remove(),
            other => Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!("unexpected token '{:?}' in collection expression", other),
            )),
        }
    }

    // --------------------------------------------------
    // Lists
    // --------------------------------------------------

    fn parse_list_dec(&mut self) -> Result<Expr, HaplError> {
        let (list_name, list_elem_type) = match self.current_token() {
            HaplTokenType::OpenListDec { name, elem_type } => (name.clone(), elem_type.clone()),
            _ => unreachable!(),
        };
        self.advance();

        let mut elements = Vec::new();
        while !self.is_at_end() {
            if matches!(self.current_token(),
                HaplTokenType::CloseListDec { name: ref n } if n == &list_name)
            {
                break;
            }
            let elem = self.parse_expression()?;
            if let Some(elem_type) = self.infer_type(&elem) {
                if elem_type != list_elem_type {
                    return Err(parser_err(
                        ErrorCode::ListElementTypeMismatch,
                        format!(
                            "list '{}' expects {:?} elements, got {:?}",
                            list_name, list_elem_type, elem_type
                        ),
                    )
                    .with_hint(format!(
                        "all elements in '{}' must be of type {:?}",
                        list_name, list_elem_type
                    )));
                }
            }
            elements.push(elem);
        }

        if self.is_at_end() {
            return Err(parser_err(
                ErrorCode::TagNotClosed,
                format!("list declaration '{}' was never closed", list_name),
            )
            .with_hint(format!(
                "add a matching closing tag for list '{}'",
                list_name
            )));
        }
        self.advance(); // consume CloseListDec

        self.declare_var(
            list_name.clone(),
            StaticType::List(Box::new(list_elem_type.clone())),
        )?;

        Ok(Expr::ListDeclaration {
            name: list_name,
            elem_type: list_elem_type,
            elements,
        })
    }

    fn parse_list_access(&mut self) -> Result<Expr, HaplError> {
        self.advance(); // consume OpenListAccess

        let list_expr  = Box::new(self.parse_expression()?);
        let index_expr = Box::new(self.parse_expression()?);

        if let Some(idx_type) = self.infer_type(&index_expr) {
            if idx_type != StaticType::Integer {
                return Err(parser_err(
                    ErrorCode::ListIndexNotInt,
                    format!("list index must be Integer, got {:?}", idx_type),
                )
                .with_hint(
                    "use an integer literal or integer variable as the index",
                ));
            }
        }

        if self.is_at_end() || !matches!(self.current_token(), HaplTokenType::CloseListAccess) {
            return Err(
                parser_err(ErrorCode::TagNotClosed, "list access block was never closed")
                    .with_hint(
                        "add a matching closing tag for the <div class=\"index\"> block",
                    ),
            );
        }
        self.advance();

        Ok(Expr::ListAccess { list: list_expr, index: index_expr })
    }

    fn parse_list_assign(&mut self) -> Result<Expr, HaplError> {
        let list_name = match self.current_token() {
            HaplTokenType::OpenListAssign { name } => name.clone(),
            _ => unreachable!(),
        };
        self.advance();

        let _list_ref  = self.parse_expression()?;
        let index_expr = Box::new(self.parse_expression()?);
        let value_expr = Box::new(self.parse_expression()?);

        if let Some(idx_type) = self.infer_type(&index_expr) {
            if idx_type != StaticType::Integer {
                return Err(parser_err(
                    ErrorCode::ListIndexNotInt,
                    format!("list index must be Integer, got {:?}", idx_type),
                )
                .with_hint(
                    "use an integer literal or integer variable as the index",
                ));
            }
        }

        if self.is_at_end()
            || !matches!(self.current_token(),
                HaplTokenType::CloseListAssign { name: ref n } if n == &list_name)
        {
            return Err(parser_err(
                ErrorCode::TagNotClosed,
                format!("list index-assign '{}' was never closed", list_name),
            )
            .with_hint(
                "add a matching closing tag for the <div class=\"index-assign\"> block",
            ));
        }
        self.advance();

        Ok(Expr::ListAssign {
            name: list_name,
            index: index_expr,
            value: value_expr,
        })
    }

    fn parse_list_push(&mut self) -> Result<Expr, HaplError> {
        let list_name = match self.current_token() {
            HaplTokenType::OpenListPush { name } => name.clone(),
            _ => unreachable!(),
        };
        self.advance();

        let _list_ref  = self.parse_expression()?;
        let value_expr = Box::new(self.parse_expression()?);

        if self.is_at_end()
            || !matches!(self.current_token(),
                HaplTokenType::CloseListPush { name: ref n } if n == &list_name)
        {
            return Err(parser_err(
                ErrorCode::TagNotClosed,
                format!("list push '{}' was never closed", list_name),
            )
            .with_hint(
                "add a matching closing tag for the <div class=\"push\"> block",
            ));
        }
        self.advance();

        Ok(Expr::ListPush { name: list_name, value: value_expr })
    }

    fn parse_list_pop(&mut self) -> Result<Expr, HaplError> {
        let list_name = match self.current_token() {
            HaplTokenType::OpenListPop { name } => name.clone(),
            _ => unreachable!(),
        };
        self.advance();

        if self.is_at_end()
            || !matches!(self.current_token(),
                HaplTokenType::CloseListPop { name: ref n } if n == &list_name)
        {
            return Err(parser_err(
                ErrorCode::TagNotClosed,
                format!("list pop '{}' was never closed", list_name),
            )
            .with_hint(
                "add a matching closing tag for the <div class=\"pop\"> block",
            ));
        }
        self.advance();

        Ok(Expr::ListPop { name: list_name })
    }

    // --------------------------------------------------
    // Maps
    // --------------------------------------------------

    pub(super) fn parse_map_dec(&mut self) -> Result<Expr, HaplError> {
        let map_name = match self.current_token() {
            HaplTokenType::OpenMapDec { name } => name.clone(),
            _ => unreachable!(),
        };
        self.advance();

        let mut entries = Vec::new();

        while !self.is_at_end() {
            if matches!(self.current_token(),
                HaplTokenType::CloseMapDec { name: ref n } if n == &map_name)
            {
                break;
            }

            let key = match self.current_token() {
                HaplTokenType::OpenMapEntry { key } => key.clone(),
                other => {
                    return Err(parser_err(
                        ErrorCode::UnexpectedToken,
                        format!("expected map entry but found '{:?}'", other),
                    )
                    .with_hint(
                        "example: <div class=\"age\"><span class=\"integer\">30</span></div>",
                    ));
                }
            };
            self.advance(); // consume OpenMapEntry

            let value_expr = self.parse_expression()?;

            if !matches!(self.current_token(),
                HaplTokenType::CloseMapEntry { key: ref k } if k == &key)
            {
                return Err(parser_err(
                    ErrorCode::TagNotClosed,
                    format!("map entry '{}' was never closed", key),
                )
                .with_hint(format!(
                    "add a matching closing tag for entry '{}'",
                    key
                )));
            }
            self.advance(); // consume CloseMapEntry
            entries.push((key, value_expr));
        }

        if self.is_at_end() {
            return Err(parser_err(
                ErrorCode::TagNotClosed,
                format!("map declaration '{}' was never closed", map_name),
            )
            .with_hint(format!(
                "add a matching closing tag for map '{}'",
                map_name
            )));
        }
        self.advance(); // consume CloseMapDec

        if !map_name.is_empty() {
            self.declare_var(map_name.clone(), StaticType::Map)?;
        }

        Ok(Expr::MapDeclaration { name: map_name, entries })
    }

    fn parse_map_get(&mut self) -> Result<Expr, HaplError> {
        self.advance(); // consume OpenMapGet

        let map_expr = Box::new(self.parse_expression()?);

        if !matches!(self.current_token(), HaplTokenType::OpenMapKey) {
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!(
                    "expected <div class=\"key\"> in map-get but found '{:?}'",
                    self.current_token()
                ),
            )
            .with_hint(
                "example: <div class=\"key\"><span class=\"string\">\"name\"</span></div>",
            ));
        }
        self.advance(); // consume OpenMapKey

        let key_expr = Box::new(self.parse_expression()?);

        if !matches!(self.current_token(), HaplTokenType::CloseMapKey) {
            return Err(
                parser_err(ErrorCode::TagNotClosed, "map-get key block was never closed")
                    .with_hint(
                        "add a matching closing tag for the <div class=\"key\"> block",
                    ),
            );
        }
        self.advance(); // consume CloseMapKey

        if !matches!(self.current_token(), HaplTokenType::CloseMapGet) {
            return Err(
                parser_err(ErrorCode::TagNotClosed, "map-get block was never closed")
                    .with_hint(
                        "add a matching closing tag for the <div class=\"map-get\"> block",
                    ),
            );
        }
        self.advance();

        Ok(Expr::MapGet { map: map_expr, key: key_expr })
    }

    fn parse_map_contains(&mut self) -> Result<Expr, HaplError> {
        self.advance(); // consume OpenMapContains

        let map_expr = Box::new(self.parse_expression()?);

        if !matches!(self.current_token(), HaplTokenType::OpenMapKey) {
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!(
                    "expected <div class=\"key\"> in map-contains but found '{:?}'",
                    self.current_token()
                ),
            )
            .with_hint(
                "example: <div class=\"key\"><span class=\"string\">\"name\"</span></div>",
            ));
        }
        self.advance();

        let key_expr = Box::new(self.parse_expression()?);

        if !matches!(self.current_token(), HaplTokenType::CloseMapKey) {
            return Err(
                parser_err(ErrorCode::TagNotClosed, "map-contains key block was never closed")
                    .with_hint(
                        "add a matching closing tag for the <div class=\"key\"> block",
                    ),
            );
        }
        self.advance();

        if !matches!(self.current_token(), HaplTokenType::CloseMapContains) {
            return Err(
                parser_err(ErrorCode::TagNotClosed, "map-contains block was never closed")
                    .with_hint(
                        "add a matching closing tag for the <div class=\"map-contains\"> block",
                    ),
            );
        }
        self.advance();

        Ok(Expr::MapContains { map: map_expr, key: key_expr })
    }

    fn parse_map_set(&mut self) -> Result<Expr, HaplError> {
        let map_name = match self.current_token() {
            HaplTokenType::OpenMapSet { name } => name.clone(),
            _ => unreachable!(),
        };
        self.advance();

        let _map_ref = self.parse_expression()?;

        if !matches!(self.current_token(), HaplTokenType::OpenMapKey) {
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!(
                    "expected <div class=\"key\"> in map-set but found '{:?}'",
                    self.current_token()
                ),
            )
            .with_hint(
                "example: <div class=\"key\"><span class=\"string\">\"age\"</span></div>",
            ));
        }
        self.advance();

        let key_expr = Box::new(self.parse_expression()?);

        if !matches!(self.current_token(), HaplTokenType::CloseMapKey) {
            return Err(
                parser_err(ErrorCode::TagNotClosed, "map-set key block was never closed")
                    .with_hint(
                        "add a matching closing tag for the <div class=\"key\"> block",
                    ),
            );
        }
        self.advance();

        if !matches!(self.current_token(), HaplTokenType::OpenMapValue) {
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!(
                    "expected <div class=\"value\"> in map-set but found '{:?}'",
                    self.current_token()
                ),
            )
            .with_hint(
                "example: <div class=\"value\"><span class=\"integer\">31</span></div>",
            ));
        }
        self.advance();

        let value_expr = Box::new(self.parse_expression()?);

        if !matches!(self.current_token(), HaplTokenType::CloseMapValue) {
            return Err(
                parser_err(ErrorCode::TagNotClosed, "map-set value block was never closed")
                    .with_hint(
                        "add a matching closing tag for the <div class=\"value\"> block",
                    ),
            );
        }
        self.advance();

        if !matches!(self.current_token(),
            HaplTokenType::CloseMapSet { name: ref n } if n == &map_name)
        {
            return Err(parser_err(
                ErrorCode::TagNotClosed,
                format!("map-set '{}' was never closed", map_name),
            )
            .with_hint(format!(
                "add a matching closing tag for the map-set block on '{}'",
                map_name
            )));
        }
        self.advance();

        Ok(Expr::MapSet { name: map_name, key: key_expr, value: value_expr })
    }

    fn parse_map_remove(&mut self) -> Result<Expr, HaplError> {
        let map_name = match self.current_token() {
            HaplTokenType::OpenMapRemove { name } => name.clone(),
            _ => unreachable!(),
        };
        self.advance();

        let _map_ref = self.parse_expression()?;

        if !matches!(self.current_token(), HaplTokenType::OpenMapKey) {
            return Err(parser_err(
                ErrorCode::UnexpectedToken,
                format!(
                    "expected <div class=\"key\"> in map-remove but found '{:?}'",
                    self.current_token()
                ),
            )
            .with_hint(
                "example: <div class=\"key\"><span class=\"string\">\"age\"</span></div>",
            ));
        }
        self.advance();

        let key_expr = Box::new(self.parse_expression()?);

        if !matches!(self.current_token(), HaplTokenType::CloseMapKey) {
            return Err(
                parser_err(ErrorCode::TagNotClosed, "map-remove key block was never closed")
                    .with_hint(
                        "add a matching closing tag for the <div class=\"key\"> block",
                    ),
            );
        }
        self.advance();

        if !matches!(self.current_token(),
            HaplTokenType::CloseMapRemove { name: ref n } if n == &map_name)
        {
            return Err(parser_err(
                ErrorCode::TagNotClosed,
                format!("map-remove '{}' was never closed", map_name),
            )
            .with_hint(format!(
                "add a matching closing tag for the map-remove block on '{}'",
                map_name
            )));
        }
        self.advance();

        Ok(Expr::MapRemove { name: map_name, key: key_expr })
    }
}
