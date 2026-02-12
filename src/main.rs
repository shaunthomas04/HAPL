#![allow(dead_code)]
mod util;
mod html_extractor;
mod lexer;
use std::env;
use crate::lexer::HaplLexer;
use crate::util::load_html_file;
use crate::html_extractor::{parse_html_to_tags, HtmlTag};

fn main() {
    let input_args: Vec<String> = env::args().collect();
    let html_file_path: &String = &input_args[1];


    let html_file_content: String = load_html_file(html_file_path).expect("Unable to load HTML file content");
    let tags: Vec<HtmlTag> = parse_html_to_tags(&html_file_content);


    //debug print of html file structure
    // for tag in &tags{
    //     tag.print_html_tag_structure(0);
    // }

    //debug hapl tokens
    let mut lexer: HaplLexer = HaplLexer::new();
    for tag in &tags{
        lexer.lex(tag);
    }
    lexer.print();


  
}



