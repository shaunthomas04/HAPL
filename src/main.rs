use std::env;
use std::fs;

fn main() {
    let input_args: Vec<String> = env::args().collect();
    let html_file_path: &String = &input_args[1];

    let html_file_contents: String = fs::read_to_string(html_file_path)
    .expect("Should have been able to read the file");

    println!("FilePath: {}", html_file_path);
    println!("With text:\n{html_file_contents}");


}