#![allow(dead_code)]
mod util;
use std::env;
use crate::util::load_html_file;


fn main() {
    let input_args: Vec<String> = env::args().collect();
    let html_file_path: &String = &input_args[1];


    let html_file_content: String = load_html_file(html_file_path).expect("Unable to load HTML file content");



    let tags: Vec<HtmlTag> = parse_tag(&html_file_content);
    // Check if any tags were found
    // if tags.is_empty() {
    //     println!("No tags found in input HTML.");
    // } else {
    //     // Iterate over each parsed tag
    //     for tag in tags {
    //         println!(
    //             "TOKEN - content: '{}', tag: '{}', id: {:?}, class: {:?}",
    //             tag.content, tag.tag_type, tag.id, tag.class
    //         );
    //     }
    // }

    for tag in &tags{
        tag.print_html_tag_structure(0);
    }

    // let tag_1: &HtmlTag = &tags[1];
    // tag_1.print_html_tag_structure(0);


  
}




struct HtmlTag {
    tag_type: String,
    id: Option<String>,
    class: Option<String>,
    content: String,
    child_tags: Vec<HtmlTag>,
}

impl HtmlTag {
    fn print_html_tag_structure(&self, level: usize){
        let indent = "  ".repeat(level);

        println!("{}Type: {}", indent, self.tag_type);
        println!("{}Id: {}", indent, self.id.as_deref().unwrap_or("-"));
        println!("{}Class: {}", indent, self.class.as_deref().unwrap_or("-"));
        // println!("{}Content: {}", indent, self.content);
        println!("{}Children:", indent);
        println!("");
        for child_tag in &self.child_tags {
            child_tag.print_html_tag_structure(level + 1);
        }

    }
}














fn parse_tag(input: &str) -> Vec<HtmlTag> {
    let mut tags: Vec<HtmlTag> = Vec::new();
    let mut remaining = input.trim();

    while let Some(start) = remaining.find('<') {
        // Find the closing '>' for the opening tag
        if let Some(end) = remaining[start..].find('>') {
            let end = end + start;
            let raw_tag = remaining[start + 1..end].trim();

            // Get tag name
            let tag_name = match raw_tag.split_whitespace().next() {
                Some(name) => name,
                None => break,
            };

            // Extract id and class
            let id_attribute = extract_id_attribute(raw_tag).unwrap_or_else(|| "None".to_string());
            let class_attribute = extract_class_attribute(raw_tag).unwrap_or_else(|| "None".to_string());

            let closing_tag = format!("</{}>", tag_name);
            let content_start = end + 1;

            // Find the closing tag
            if let Some(close_pos) = remaining[content_start..].find(&closing_tag) {
                let content_end = content_start + close_pos;
                let content = remaining[content_start..content_end].trim();
                let child_tags:Vec<HtmlTag> =  parse_tag(content);

                // Push this tag
                tags.push(HtmlTag {
                    tag_type: tag_name.to_string(),
                    content: content.to_string(),
                    id: Some(id_attribute),
                    class: Some(class_attribute),
                    child_tags: child_tags, 
                });

                // Move remaining past this tag
                remaining = &remaining[content_end + closing_tag.len()..];
            } else {
                break; // No closing tag found
            }
        } else {
            break; // Malformed tag
        }

        remaining = remaining.trim();
    }

    tags
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

