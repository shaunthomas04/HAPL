use crate::error::{ErrorCode, HaplError, config_err};
use std::collections::HashMap;

pub struct HaplConfig {
    keywords: HashMap<String, String>,
}
const VALID_KEYWORDS: &[&str] = &[
    // Types
    "integer", "double", "string", "boolean", "void", "map",
    // Arithmetic
    "+", "-", "*", "/", "%",
    // Boolean
    "&&", "||", "!",
    // Comparison
    "equal", "not_equal", "less", "less_equal", "greater", "greater_equal",
    // Control flow
    "conditional", "if", "elif", "else",
    "while", "for",
    "return",
    // Loop sections
    "condition", "body", "iterator", "increment",
    // Functions
    "params", "args",
    // List operations
    "index", "index-assign", "push", "pop",
    // Map operations
    "map-get", "map-set", "map-remove", "map-contains",
    // Other
    "length", "key", "value",
    // Function type suffixes
    "integer-function", "double-function", "string-function",
    "boolean-function", "void-function", "map-function",
    // Param type suffixes
    "integer-param", "double-param", "string-param",
    "boolean-param", "void-param", "map-param",
    // List type suffixes
    "integer-list", "double-list", "string-list", "boolean-list",
    //http post and get
    "http-get", "http-post", "url",
];

impl HaplConfig {
    pub fn load(path: &str) -> Result<Self, HaplError> {
        let content = std::fs::read_to_string(path)
            .map_err(|_| {
                config_err(ErrorCode::ConfigFileNotFound,
                    format!("config file not found: '{}'", path))
                .with_hint("check that the path to your config file is correct")
            })?;

        let raw: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| {
                config_err(ErrorCode::ConfigInvalidJson,
                    format!("config file contains invalid JSON: {}", e))
                .with_hint("validate your JSON at jsonlint.com")
            })?;

        let root = raw.as_object().ok_or_else(|| {
            config_err(ErrorCode::ConfigInvalidStructure,
                "config root must be a JSON object")
            .with_hint("example: { \"keywords\": { \"map-get\": \"fetch\" } }")
        })?;

        let keywords_val = root.get("keywords").ok_or_else(|| {
            config_err(ErrorCode::ConfigInvalidStructure,
                "missing required field \"keywords\"")
            .with_hint("config must contain a \"keywords\" object")
        })?;

        let keywords_map = keywords_val.as_object().ok_or_else(|| {
            config_err(ErrorCode::ConfigInvalidStructure,
                "\"keywords\" must be a JSON object mapping HAPL keywords to aliases")
            .with_hint("example: { \"keywords\": { \"map-get\": \"fetch\" } }")
        })?;

        let mut keywords: HashMap<String, String> = HashMap::new();
        let mut alias_to_keyword: HashMap<String, String> = HashMap::new();

        for (hapl_keyword, alias_val) in keywords_map {
            if !VALID_KEYWORDS.contains(&hapl_keyword.as_str()) {
                return Err(
                    config_err(ErrorCode::ConfigInvalidKeyword,
                        format!("'{}' is not a valid HAPL keyword", hapl_keyword))
                    .with_hint("only valid HAPL keywords can be remapped")
                );
            }

            let alias = alias_val.as_str().ok_or_else(|| {
                config_err(ErrorCode::ConfigInvalidStructure,
                    format!("alias for '{}' must be a string", hapl_keyword))
                .with_hint("all aliases must be strings")
            })?;

            if alias.trim().is_empty() {
                return Err(
                    config_err(ErrorCode::ConfigEmptyAlias,
                        format!("alias for keyword '{}' must not be empty", hapl_keyword))
                    .with_hint("provide a non-empty string as the alias")
                );
            }

            if let Some(existing) = alias_to_keyword.get(alias) {
                return Err(
                    config_err(ErrorCode::ConfigDuplicateAlias,
                        format!("alias '{}' is already used for keyword '{}'", alias, existing))
                    .with_hint("each alias must map to exactly one keyword")
                );
            }

            alias_to_keyword.insert(alias.to_string(), hapl_keyword.clone());
            keywords.insert(alias.to_string(), hapl_keyword.clone());
        }

        Ok(Self { keywords })
    }

    pub fn resolve<'a>(&'a self, class: &'a str) -> &'a str {
        self.keywords.get(class).map(|s| s.as_str()).unwrap_or(class)
    }
}