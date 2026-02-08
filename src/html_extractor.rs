use crate::util::abbreviate_content;

//This is not an AST node, it simply represents HTML as a node
pub struct HtmlTag {
    pub tag_type: String,
    pub id: Option<String>,
    pub class: Option<String>,
    pub content: String,
    pub child_tags: Vec<HtmlTag>,
}

impl HtmlTag {
    //print htmlTag node for debugging purposes
    pub fn print_html_tag_structure(&self, level: usize){
        let indent = "  ".repeat(level);

        println!("{}Type: {}", indent, self.tag_type);
        println!("{}Id: {}", indent, self.id.as_deref().unwrap_or("-"));
        println!("{}Class: {}", indent, self.class.as_deref().unwrap_or("-"));
        println!("{}Content: {}", indent, abbreviate_content(&self.content, 100));
        println!("{}Children:", indent);
        println!("");
        for child_tag in &self.child_tags {
            child_tag.print_html_tag_structure(level + 1);
        }

    }
}


pub fn parse_html_to_tags(input: &str) -> Vec<HtmlTag> {
    let mut tags: Vec<HtmlTag> = Vec::new();
    let mut remaining = input.trim();

    while let Some(start) = remaining.find('<') {
        // Skip comments
        if remaining[start..].starts_with("<!--") {
            if let Some(end_comment) = remaining[start..].find("-->") {
                remaining = &remaining[start + end_comment + 3..];
                remaining = remaining.trim();
                continue;
            } else {
                break; // unclosed comment
            }
        }

        // Find closing '>' for the opening tag
        if let Some(end) = remaining[start..].find('>') {
            let end = end + start;
            let raw_tag = remaining[start + 1..end].trim();

            // Skip empty tags
            if raw_tag.is_empty() {
                remaining = &remaining[end + 1..];
                continue;
            }

            // Get tag name
            let tag_name = match raw_tag.split_whitespace().next() {
                Some(name) => name,
                None => {
                    remaining = &remaining[end + 1..];
                    continue;
                }
            };

            // Extract attributes
            let id_attribute = extract_id_attribute(raw_tag);
            let class_attribute = extract_class_attribute(raw_tag);

            // Detect self-closing or void tags
            let is_self_closing = raw_tag.ends_with('/');
            let is_void_tag = matches!(
                tag_name,
                "meta" | "link" | "img" | "input" | "br" | "hr"
            );

            let content_start = end + 1;

            if is_self_closing || is_void_tag {
                // Self-closing / void tag
                tags.push(HtmlTag {
                    tag_type: tag_name.to_string(),
                    id: id_attribute,
                    class: class_attribute,
                    content: String::new(),
                    child_tags: Vec::new(),
                });

                remaining = &remaining[end + 1..];
                remaining = remaining.trim();
                continue;
            }

            // Normal tag: find closing
            let closing_tag = format!("</{}>", tag_name);
            if let Some(close_pos) = remaining[content_start..].find(&closing_tag) {
                let content_end = content_start + close_pos;
                let content = remaining[content_start..content_end].trim();

                // Recursively parse children
                let child_tags = parse_html_to_tags(content);

                tags.push(HtmlTag {
                    tag_type: tag_name.to_string(),
                    id: id_attribute,
                    class: class_attribute,
                    content: content.to_string(),
                    child_tags,
                });

                remaining = &remaining[content_end + closing_tag.len()..];
                remaining = remaining.trim();
            } else {
                // Malformed tag; skip
                remaining = &remaining[end + 1..];
                remaining = remaining.trim();
            }
        } else {
            break; // malformed tag
        }
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