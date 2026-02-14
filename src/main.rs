#![allow(dead_code)]

mod util;
mod html_extractor;
mod lexer;
mod parser;
mod ast;
mod interpreter;

use std::env;

use crate::lexer::{HaplLexer, HaplTokenType};
use crate::parser::HaplParser;
use crate::interpreter::Interpreter;
use crate::util::load_html_file;
use crate::html_extractor::{parse_html_to_tags, HtmlTag};
use crate::ast::{LiteralValue};

fn main() {
    let input_args: Vec<String> = env::args().collect();

    if input_args.len() < 2 {
        panic!("Usage: cargo run <file.html>");
    }

    let html_file_path = &input_args[1];

    // 1️⃣ Load HTML
    let html_file_content =
        load_html_file(html_file_path).expect("Unable to load HTML file content");

    let tags: Vec<HtmlTag> = parse_html_to_tags(&html_file_content);

    // 2️⃣ Lex
    let mut lexer = HaplLexer::new();
    for tag in &tags {
        lexer.lex(tag);
    }

    println!("--- TOKENS ---");
    lexer.print();

    // 3️⃣ Filter only arithmetic-related tokens
    // let expression_tokens = lexer
    //     .tokens()
    //     .iter()
    //     .filter(|token| {
    //         matches!(
    //             token.token_type,
    //             HaplTokenType::OpenOperator { .. }
    //                 | HaplTokenType::CloseOperator { .. }
    //                 | HaplTokenType::Literal(LiteralValue::Integer(_))
    //         )
    //     })
    //     .cloned()
    //     .collect::<Vec<_>>();

    // if expression_tokens.is_empty() {
    //     println!("No arithmetic expressions found.");
    //     return;
    // }

    // // 4️⃣ Parse
    // let mut parser = HaplParser::new(expression_tokens);
    // let ast = parser.parse();

    // println!("\n--- AST ---");
    // println!("{:#?}", ast);

    // // 5️⃣ Interpret
    // let mut interpreter = Interpreter::new();
    // let result = interpreter.eval(&ast);

    // println!("\n--- RESULT ---");
    // match result {
    //     LiteralValue::Integer(n) => println!("{}", n),
    //     LiteralValue::Double(f) => println!("{}", f),
    //     LiteralValue::String(s) => println!("{}", s),
    // }
}
