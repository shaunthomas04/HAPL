use crate::util::abbreviate_content;

#[derive(Debug)]
pub struct HtmlTag {
    pub tag_type: String,
    pub id: Option<String>,
    pub class: Option<String>,
    pub content: String,
    pub child_tags: Vec<HtmlTag>,
}

impl HtmlTag {
    pub fn print_html_tag_structure(&self, level: usize) {
        let indent = "  ".repeat(level);

        println!("{}Type: {}", indent, self.tag_type);
        println!("{}Id: {}", indent, self.id.as_deref().unwrap_or("-"));
        println!("{}Class: {}", indent, self.class.as_deref().unwrap_or("-"));
        println!(
            "{}Content: {}",
            indent,
            abbreviate_content(&self.content, 100)
        );
        println!("{}Children:\n", indent);

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
                break;
            }
        }

        // Find end of opening tag
        if let Some(end) = remaining[start..].find('>') {
            let end = end + start;
            let raw_tag = remaining[start + 1..end].trim();

            if raw_tag.is_empty() {
                remaining = &remaining[end + 1..];
                continue;
            }

            let tag_name = match raw_tag.split_whitespace().next() {
                Some(name) => name,
                None => {
                    remaining = &remaining[end + 1..];
                    continue;
                }
            };

            let id_attribute = extract_id_attribute(raw_tag);
            let class_attribute = extract_class_attribute(raw_tag);

            let is_self_closing = raw_tag.ends_with('/');
            let is_void_tag = matches!(
                tag_name,
                "meta" | "link" | "img" | "input" | "br" | "hr"
            );

            let content_start = end + 1;

            if is_self_closing || is_void_tag {
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

            // Proper nested closing tag matching
            if let Some(close_pos) =
                find_matching_closing_tag(&remaining[content_start..], tag_name)
            {
                let content_end = content_start + close_pos;
                let content = remaining[content_start..content_end].trim();

                let child_tags = parse_html_to_tags(content);

                tags.push(HtmlTag {
                    tag_type: tag_name.to_string(),
                    id: id_attribute,
                    class: class_attribute,
                    content: content.to_string(),
                    child_tags,
                });

                let closing_tag = format!("</{}>", tag_name);
                remaining =
                    &remaining[content_end + closing_tag.len()..];
                remaining = remaining.trim();
            } else {
                remaining = &remaining[end + 1..];
                remaining = remaining.trim();
            }
        } else {
            break;
        }
    }

    tags
}

// ✅ NEW: Proper nested tag matcher
fn find_matching_closing_tag(input: &str, tag_name: &str) -> Option<usize> {
    let open_pattern = format!("<{}", tag_name);
    let close_pattern = format!("</{}>", tag_name);

    let mut depth = 1;
    let mut index = 0;

    while index < input.len() {
        let next_open = input[index..].find(&open_pattern);
        let next_close = input[index..].find(&close_pattern);

        match (next_open, next_close) {
            (Some(o), Some(c)) => {
                let o = index + o;
                let c = index + c;

                if o < c {
                    depth += 1;
                    index = o + open_pattern.len();
                } else {
                    depth -= 1;
                    if depth == 0 {
                        return Some(c);
                    }
                    index = c + close_pattern.len();
                }
            }
            (None, Some(c)) => {
                let c = index + c;
                depth -= 1;
                if depth == 0 {
                    return Some(c);
                }
                index = c + close_pattern.len();
            }
            _ => break,
        }
    }

    None
}

fn extract_id_attribute(raw_tag: &str) -> Option<String> {
    for part in raw_tag.split_whitespace() {
        if let Some(value) = part.strip_prefix("id=") {
            if value.starts_with('"') && value.ends_with('"') {
                let inner = &value[1..value.len() - 1];
                if !inner.is_empty() && !inner.contains('"') {
                    return Some(inner.to_string());
                }
            }
        }
    }
    None
}

fn extract_class_attribute(raw_tag: &str) -> Option<String> {
    for part in raw_tag.split_whitespace() {
        if let Some(value) = part.strip_prefix("class=") {
            if value.starts_with('"') && value.ends_with('"') {
                let inner = &value[1..value.len() - 1];
                if !inner.is_empty() && !inner.contains('"') {
                    return Some(inner.to_string());
                }
            }
        }
    }
    None
}
