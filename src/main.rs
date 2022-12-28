mod dictionary_generator;

use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    let language = &args[1];
    let base_path = format!("data/{}", language);

    if !Path::new(&base_path).is_dir() {
        panic!("Folder doesn't exist for language '{}'", language);
    }

    let letterpoints_file = format!("{}/letterpoints.json", base_path);
    if !Path::new(&letterpoints_file).is_file() {
        panic!(
            "The 'letterpoints.json' file doesn't exist at '{}'",
            letterpoints_file
        );
    }

    let dictionary = dictionary_generator::generate(base_path);

    let letters = &args[2];
    println!("{}", letters);
}
