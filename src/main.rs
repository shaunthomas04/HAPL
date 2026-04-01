#![allow(dead_code)]

mod util;
mod html_extractor;
mod lexer;
mod parser;
mod ast;
mod interpreter;
mod error;
mod config;

use std::env;

use crate::config::HaplConfig;
use crate::error::{lexer_err, ErrorCode, config_err};
use crate::lexer::{HaplLexer, HaplTokenType};
use crate::parser::HaplParser;
use crate::interpreter::Interpreter;
use crate::util::load_html_file;
use crate::html_extractor::{parse_html_to_tags, HtmlTag};

// ==========================================
// DEBUG FLAGS — toggle these on/off
// ==========================================
const DEBUG_HTML_TAGS: bool = false;
const DEBUG_TOKENS: bool    = false;
const DEBUG_AST: bool       = false;
const DEBUG_RESULTS: bool   = false;
// ==========================================

// cargo run html-location.html --keyword-configs keyword-remap-location.json
fn parse_config_flag(args: &[String]) -> Option<HaplConfig> {
    let pos = args.iter().position(|a| a == "--keyword-configs")?;

    let path = args.get(pos + 1).unwrap_or_else(|| {
        config_err(
            ErrorCode::ConfigMissingPath,
            "--keyword-configs flag requires a path argument",
        )
        .with_hint("usage: cargo run <file.html> --keyword-configs remap.json")
        .report_and_exit("cli");
    });

    if !path.ends_with(".json") {
        config_err(
            ErrorCode::ConfigNotJsonFile,
            format!("config file must be a .json file, got '{}'", path),
        )
        .with_hint("usage: cargo run <file.html> --keyword-configs remap.json")
        .report_and_exit("cli");
    }

    Some(HaplConfig::load(path).unwrap_or_else(|e| e.report_and_exit(path)))
}

fn main() {
    let input_args: Vec<String> = env::args().collect();

    if input_args.len() < 2 {
        eprintln!("{}{}error{}: no input file provided",
            "\x1b[1m", "\x1b[31m", "\x1b[0m");
        eprintln!("  \x1b[1m\x1b[36musage:\x1b[0m cargo run <file.html> [--keyword-configs remap.json]");
        std::process::exit(1);
    }

    let html_file_path = &input_args[1];

    // ── Load config if provided ──────────────────────────────────────
    let config = parse_config_flag(&input_args);

    // ── Load HTML ────────────────────────────────────────────────────
    let html_file_content = load_html_file(html_file_path).unwrap_or_else(|| {
        lexer_err(
            ErrorCode::InvalidLiteral,
            format!("could not read '{}'", html_file_path),
        )
        .with_hint("check that the file exists and you have read permissions")
        .report_and_exit(html_file_path);
    });

    // ── Extract HTML tags ────────────────────────────────────────────
    let tags: Vec<HtmlTag> = parse_html_to_tags(&html_file_content);

    if tags.is_empty() {
        eprintln!(
            "\x1b[1m\x1b[33mwarning\x1b[0m: '{}' contains no recognisable Hapl tags — nothing to do",
            html_file_path
        );
        return;
    }

    if DEBUG_HTML_TAGS {
        println!("--- HTML TAGS ---");
        for tag in &tags {
            println!("{:#?}", tag);
        }
    }

    // ── Lex ──────────────────────────────────────────────────────────
    let mut lexer = match config {
        Some(c) => HaplLexer::with_config(c),
        None    => HaplLexer::new(),
    };

    for tag in &tags {
        lexer.lex(tag).unwrap_or_else(|e| {
            e.report_and_exit(html_file_path);
        });
    }

    if DEBUG_TOKENS {
        println!("--- TOKENS ---");
        lexer.print();
    }

    let tokens = lexer
        .tokens()
        .iter()
        .filter(|t| {
            !matches!(
                t.token_type,
                HaplTokenType::OpenHtmlTag { .. } | HaplTokenType::CloseHtmlTag { .. }
            )
        })
        .cloned()
        .collect::<Vec<_>>();

    if tokens.is_empty() {
        eprintln!(
            "\x1b[1m\x1b[33mwarning\x1b[0m: lexer produced no tokens from '{}' — nothing to parse",
            html_file_path
        );
        return;
    }

    // ── Parse ────────────────────────────────────────────────────────
    let mut parser = HaplParser::new(tokens);
    // let ast_nodes = parser.parse_program().unwrap_or_else(|_e| {
    //     parser_err(
    //         ErrorCode::UnexpectedToken,
    //         "parser failed to produce an AST",
    //     )
    //     .report_and_exit(html_file_path);
    // });
    let ast_nodes = parser.parse_program().unwrap_or_else(|e| {
        e.report_and_exit(html_file_path);
    });


    if ast_nodes.is_empty() {
        eprintln!(
            "\x1b[1m\x1b[33mwarning\x1b[0m: parser produced an empty AST from '{}' — nothing to run",
            html_file_path
        );
        return;
    }

    if DEBUG_AST {
        println!("--- AST ---");
        for node in &ast_nodes {
            println!("{:#?}", node);
        }
    }

    // ── Interpret ────────────────────────────────────────────────────
    if DEBUG_RESULTS {
        println!("--- RESULTS ---");
    }

    let mut interpreter = Interpreter::new();
    interpreter.run(&ast_nodes, html_file_path);
}