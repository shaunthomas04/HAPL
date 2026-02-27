#![allow(dead_code)]

mod util;
mod html_extractor;
mod lexer;
mod parser;
mod ast;
mod interpreter;
mod error;

use std::env;

use crate::lexer::{HaplLexer, HaplTokenType};
use crate::parser::HaplParser;
use crate::interpreter::Interpreter;
use crate::util::load_html_file;
use crate::html_extractor::{parse_html_to_tags, HtmlTag};

// ==========================================
// DEBUG FLAGS — toggle these on/off
// ==========================================
const DEBUG_HTML_TAGS: bool   = false; // raw parsed HTML tags
const DEBUG_TOKENS: bool      = false; // lexer token output
const DEBUG_AST: bool         = false   ; // parsed AST nodes
const DEBUG_RESULTS: bool     = true;  // interpreter output
// ==========================================

fn main() {
    let input_args: Vec<String> = env::args().collect();

    if input_args.len() < 2 {
        panic!("Usage: cargo run <file.html>");
    }

    let html_file_path = &input_args[1];

    // Load HTML
    let html_file_content =
        load_html_file(html_file_path).expect("Unable to load HTML file content");

    let tags: Vec<HtmlTag> = parse_html_to_tags(&html_file_content);

    if DEBUG_HTML_TAGS {
        println!("--- HTML TAGS ---");
        for tag in &tags {
            println!("{:#?}", tag);
        }
    }

    //Lex
    let mut lexer = HaplLexer::new();
    for tag in &tags {
        lexer.lex(tag).unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
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
                HaplTokenType::OpenHtmlTag { .. }
                    | HaplTokenType::CloseHtmlTag { .. }
            )
        })
        .cloned()
        .collect::<Vec<_>>();

    if tokens.is_empty() {
        println!("No tokens found.");
        return;
    }

    // Parse entire program
    let mut parser = HaplParser::new(tokens);
    let ast_nodes = parser.parse_program();

    if DEBUG_AST {
        println!("--- AST ---");
        for node in &ast_nodes {
            println!("{:#?}", node);
        }
    }

    // Interpret entire program
    if DEBUG_RESULTS {
        println!("--- RESULTS ---");
    }
    let mut interpreter = Interpreter::new();
    interpreter.run(&ast_nodes);
}