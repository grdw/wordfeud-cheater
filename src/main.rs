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

    let wordlist_file = format!("{}/wordlist.txt", base_path);
    if !Path::new(&wordlist_file).is_file() {
        panic!(
            "The 'wordlist.txt' file doesn't exist at '{}'",
            wordlist_file
        );
    }

    println!("{:?}", args);
}
