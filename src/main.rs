#![allow(dead_code)]
use std::env;
use std::fs;

fn main() {
    // let input_args: Vec<String> = env::args().collect();
    // let html_file_path: &String = &input_args[1];
    // let html_file_contents: String = fs::read_to_string(html_file_path)
    // .expect("Should have been able to read the file");
    // println!("FilePath: {}", html_file_path);
    // println!("With text:\n{html_file_contents}");




    let html: &str = r##"<a href="#">Home</a>"##;

    match parse_tag(html) {
        None => println!("Parsing for {} failed", html),
        Some(quotient) => {
            println!("HTML TOKEN- content:{}, tag:{}", quotient.content, quotient.tag_type)
        },
    }


}




struct HtmlTag {
    tag_type: String,
    // id: String,
    // class: String,
    // child_tags: Vec<HtmlTag>,
    content: String
}


fn parse_tag(input: &str) -> Option<HtmlTag> {
    // Look for the opening tag in the input
    if let Some(start) = input.find('<') {
        if let Some(end) = input[start..].find('>') {

            let tag_name = &input[start + 1..start + end];
            let closing_tag = format!("</{}>", tag_name);

            if let Some(close_pos) = input.find(&closing_tag) {
                let content_start = start + end + 1;
                let content_end = close_pos;
                let content = &input[content_start..content_end];

                return Some(HtmlTag {
                    tag_type: tag_name.to_string(),
                    content: content.to_string(),
                });
            }
        }
    }
    None
}