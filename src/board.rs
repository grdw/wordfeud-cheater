use crate::dictionary_generator::Dictionary;
use std::collections::HashSet;

pub struct Board<'a> {
    pub dictionary: &'a Dictionary,
    pub letters: &'a String,
    pub letterpoints_path: &'a String,
    pub layout_path: &'a String,
    pub current_board_path: &'a String
}

impl Board<'_>  {
    pub fn new<'a>(
        letters: &'a String,
        dictionary: &'a Dictionary,
        letterpoints_path: &'a String,
        layout_path: &'a String,
        current_board_path: &'a String) -> Board<'a> {

        Board {
            letters: letters,
            dictionary: dictionary,
            letterpoints_path: letterpoints_path,
            layout_path: layout_path,
            current_board_path: current_board_path
        }
    }
    pub fn plays(&self) -> Vec<String> {
        let combos = self.combinations();
        let mut anagrams = self.dictionary.get_anagrams_for(&combos);
        anagrams.sort_by(|a, b| a.len().cmp(&b.len()));
        anagrams
    }

    fn combinations(&self) -> HashSet<String> {
        let mut combinations = HashSet::new();
        let mut output = String::new();

        for i in 2..=self.letters.len() {
            self.find_unique_combinations(0, i, &mut combinations, &mut output);
        }

        combinations
    }

    // Took this algorithm from:
    // https://www.techiedelight.com/find-distinct-combinations-of-given-length/
    fn find_unique_combinations(&self,
                                offset: usize,
                                length: usize,
                                hash: &mut HashSet<String>,
                                list: &mut String) {

        if self.letters.len() == 0 || length > self.letters.len() {
            return;
        }

        if length == 0 {
            hash.insert(list.clone());
            return;
        }

        for i in offset..self.letters.len() {
            list.push(self.letters.chars().nth(i).unwrap());
            self.find_unique_combinations(i + 1, length - 1, hash, list);
            list.remove(list.len() - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use serial_test::serial;
    use crate::dictionary_generator::generate;

    #[test]
    #[serial]
    fn test_plays_teers() {
        let db_file = String::from("data/test/dictionary.sqlite");
        if Path::new(&db_file).is_file() {
            fs::remove_file(db_file).unwrap();
        }

        let base_path = String::from("data/test");
        let letters = String::from("TEERS");
        let dictionary = generate(base_path);
        let lp_path = String::from("data/test/letterpoints.txt");
        let layout_path = String::from("layout.default.board");
        let current_board_path = String::from("current.board");

        let board = Board::new(
            &letters,
            &dictionary,
            &lp_path,
            &layout_path,
            &current_board_path
        );

        assert_eq!(board.plays(), vec![
            String::from("ER"),
            String::from("EET"),
            String::from("EERST"),
            String::from("ESTER"),
            String::from("RESET")
        ]);
    }

    #[test]
    #[serial]
    fn test_plays_joker() {
        let db_file = String::from("data/test/dictionary.sqlite");
        if Path::new(&db_file).is_file() {
            fs::remove_file(db_file).unwrap();
        }

        let base_path = String::from("data/test");
        let letters = String::from("T??RS");
        let dictionary = generate(base_path);
        let lp_path = String::from("data/test/letterpoints.txt");
        let layout_path = String::from("layout.default.board");
        let current_board_path = String::from("current.board");

        let board = Board::new(
            &letters,
            &dictionary,
            &lp_path,
            &layout_path,
            &current_board_path
        );

        assert_eq!(board.plays(), vec![
            String::from("ER"),
            String::from("MN"),
            String::from("ZE"),
            String::from("EET"),
            String::from("EERST"),
            String::from("ESTER"),
            String::from("RESET"),
            String::from("STAAR"),
            String::from("STEUR")
        ]);
    }
}
