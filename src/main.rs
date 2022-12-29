mod board;
mod dictionary_generator;

use std::env;
use std::path::Path;
use board::Board;

fn main() {
    let args: Vec<String> = env::args().collect();
    let language = &args[1];
    let letters = &args[2];
    let default = String::from("default");
    let layout = args.get(3).unwrap_or(&default);
    let base_path = format!("data/{}", language);

    if !Path::new(&base_path).is_dir() {
        panic!("Folder doesn't exist for language '{}'", language);
    }

    let letterpoints_path = format!("{}/letterpoints.txt", base_path);
    ensure_file_exists(&letterpoints_path);

    let layout_path = format!("layout.{}.board", &layout);
    ensure_file_exists(&layout_path);

    let current_board_path = String::from("current.board");
    ensure_file_exists(&current_board_path);

    let dictionary = dictionary_generator::generate(base_path);
    let board = Board::new(
        letters,
        &dictionary,
        &letterpoints_path,
        &layout_path,
        &current_board_path
    );

    println!("{:?}", board.anagrams());
}

fn ensure_file_exists(file_path: &String) {
    let split = file_path.split("/").collect::<Vec<&str>>();
    if !Path::new(&file_path).is_file() {
        panic!(
            "The '{}' file doesn't exist at '{}'",
            &split[split.len() - 1],
            file_path
        );
    }
}
