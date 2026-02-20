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

    // println!("--- TOKENS ---");
    lexer.print();

    // let tokens = lexer
    // .tokens()
    // .iter()
    // .filter(|t| {
    //     !matches!(
    //         t.token_type,
    //         HaplTokenType::OpenHtmlTag { .. }
    //             | HaplTokenType::CloseHtmlTag { .. }
    //     )
    // })
    // .cloned()
    // .collect::<Vec<_>>();


    // if tokens.is_empty() {
    //     println!("No tokens found.");
    //     return;
    // }

    // // 3️⃣ Parse entire program
    // let mut parser = HaplParser::new(tokens);
    // let ast_nodes = parser.parse_program();

    // // println!("\n--- AST ---");
    // // for ast in &ast_nodes {
    // //     println!("{:#?}", ast);
    // // }

    // // 4️⃣ Interpret entire program
    // println!("\n--- RESULTS ---");

    // let mut interpreter = Interpreter::new();

    // for ast in &ast_nodes {
    //     let result = interpreter.eval(ast);
    //     println!("{:?}", result);
    // }
}
