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




    // let html: &str = r##"<a href="#"><a href="#">Home</a></a>"##;
    let html: &str = r#"<div id="my-id" class="weird-class">Hello <span>world!</span> & some text #123</div>"#;


    match parse_tag(html) {
        None => println!("Parsing for {} failed", html),
        Some(quotient) => {
            println!("HTML TOKEN- content:{}, tag:{}, id:{:?}, class{:?}", quotient.content, quotient.tag_type, quotient.id, quotient.class)
        },
    }

  
}




struct HtmlTag {
    tag_type: String,
    id: Option<String>,
    class: Option<String>,
    // child_tags: Vec<HtmlTag>,
    content: String
}


fn parse_tag(input: &str) -> Option<HtmlTag> {
    let input = input.trim();

    if let Some(start) = input.find('<') {
        if let Some(end) = input[start..].find('>') {
            let raw_tag = input[start + 1..start + end].trim();

            // Extract just the tag name (before attributes)
            let tag_name = raw_tag.split_whitespace().next()?;

            //get the id of the tag if it exists
            let id_attribute: String;
            match extract_id_attribute(raw_tag) {
                None => id_attribute = "None".to_string(),
                Some(id_attribute_value) => {
                    id_attribute = id_attribute_value;
                },
            }

            let class_attribute: String;
            match extract_class_attribute(raw_tag) {
                None => class_attribute = "None".to_string(),
                Some(class_attribute_value) => {
                    class_attribute = class_attribute_value;
                },
            }



            let closing_tag = format!("</{}>", tag_name);

            if let Some(close_pos) = input.rfind(&closing_tag) {
                let content_start = start + end + 1;
                let content_end = close_pos;

                let content = input[content_start..content_end].trim();

                return Some(HtmlTag {
                    tag_type: tag_name.to_string(),
                    content: content.to_string(),
                    id: Some(id_attribute),
                    class: Some(class_attribute)
                });
            }
        }
    }

    None
}


fn extract_id_attribute(raw_tag: &str) -> Option<String> {    
    //design choice (im lazy) - splitting at whitespace means cannot use multiple words in a class tag
    //actually could make some sense to tighten up whats allowed 
    for part in raw_tag.split_whitespace() {
        if let Some(value) = part.strip_prefix("id=") {
            // Must start and end with exactly one quote
            if value.starts_with('"') && value.ends_with('"') {
                let inner: &str = &value[1..value.len() - 1];

                // Reject bad content
                if inner.contains('"') {
                    return None;
                }
                if inner.is_empty() {
                    return None;
                }

                return Some(inner.to_string());
            }
        }
    }
    None
}

fn extract_class_attribute(raw_opening_tag: &str) -> Option<String> {
    for part in raw_opening_tag.split_whitespace(){
        if let Some(value) = part.strip_prefix("class="){
            // Must start and end with exactly one quote
            if value.starts_with('"') && value.ends_with('"') {
                let inner = &value[1..value.len() - 1];

                // Reject bad content
                if inner.contains('"') {
                    return None;
                }
                if inner.is_empty() {
                    return None;
                }

                return Some(inner.to_string());
            }
        }

    }
    None
}

