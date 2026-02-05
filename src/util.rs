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
