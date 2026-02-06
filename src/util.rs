use std::fs;

//return either html content as a String or Nothing
pub fn load_html_file(html_filepath: &str) -> Option<String>{
    let html_content = fs::read_to_string(html_filepath);

    match html_content{
        Ok(content) =>{
            Some(content)
        }
        Err(e) => {
            println!("Unable to load HTML file from path {}: {}", html_filepath, e);
            None
        }
    }
}

//helper function to abbrievate content that is too long
pub fn abbreviate_content(content: &str, max_len: usize) -> String {
    let trimmed = content.trim();

    if trimmed.chars().count() <= max_len {
        return trimmed.to_string();
    }

    let start_len = max_len / 2;
    let end_len = max_len / 2;

    let start: String = trimmed.chars().take(start_len).collect();
    let end: String = trimmed.chars().rev().take(end_len).collect::<Vec<_>>().into_iter().rev().collect();

    format!("{} ... {}", start, end)
}
