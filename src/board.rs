use crate::dictionary_generator::Dictionary;
use std::collections::{HashSet, HashMap};
use std::fs;

pub struct Board<'a> {
    pub letters: &'a String,
    pub dictionary: &'a Dictionary,
    scorer: LetterScorer,
    parsed_board: ParsedBoard
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
            scorer: LetterScorer::parse(letterpoints_path),
            parsed_board: ParsedBoard::parse(layout_path, current_board_path)
        }
    }

    pub fn anagrams(&self) -> Vec<String> {
        let combos = self.combinations();
        let mut anagrams = self.dictionary.get_anagrams_for(&combos);

        anagrams.sort_by(|a, b| {
            let a_score = self.scorer.score(b, self.letters);
            let b_score = self.scorer.score(a, self.letters);
            a_score.cmp(&b_score)
        });

        anagrams
    }

    fn optimal_plays(&self) -> Vec<Play> {
        let mut plays = vec![];

        if self.parsed_board.is_opening_turn() {
            let (endx, y) = self.parsed_board.origin();
            let anagrams = self.anagrams();
            let word = &anagrams[0]; // This can _and_ should be optimized
            let sx = endx - word.len() + 1;

            for tx in sx..=endx {
                plays.push(
                    Play {
                        points: self.scorer.score(word, self.letters),
                        position: (tx, y),
                        word: word.to_string()
                    }
                );
            }
        } else {
            // Start by detecting where each letter is in the tiles
            // then start connecting them with your existing letters.
            // See which letter(s) forms the largest amount of points.
        }

        plays
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

struct ParsedBoard {
    tiles: Vec<Vec<Tile>>
}

#[derive(Debug, Eq, PartialEq)]
enum Tile {
    Letter(char),
    Empty,
    Start,
    DoubleLetter,
    TripleLetter,
    DoubleWord,
    TripleWord,
}

impl ParsedBoard {
    fn parse(layout_path: &String, current_board_path: &String) -> ParsedBoard {
        let current_board = fs::read_to_string(current_board_path).unwrap();
        let layout = fs::read_to_string(layout_path).unwrap();

        let mut tiles: Vec<Vec<Tile>> =
            layout
                .split_terminator("\n")
                .map(|line| {
                    line
                        .chars()
                        .map(|tile| {
                            match tile {
                                '.' => Tile::Empty,
                                '1' => Tile::Start,
                                '2' => Tile::DoubleLetter,
                                '3' => Tile::TripleLetter,
                                '4' => Tile::DoubleWord,
                                '5' => Tile::TripleWord,
                                _ => panic!("Invalid tile")
                            }
                        })
                        .collect()
                })
                .collect();

        for (y, l) in current_board.split_terminator("\n").enumerate() {
            for (x, c) in l.chars().enumerate() {
                if c == '.' {
                    continue
                }

                tiles[y][x] = Tile::Letter(c);
            }
        }

        ParsedBoard { tiles: tiles }
    }

    fn origin(&self) -> (usize, usize) {
        let mut y = 0;
        let mut x = 0;

        for (ty, row) in self.tiles.iter().enumerate() {
            for (tx, tile) in row.iter().enumerate() {
                if let Tile::Start = tile {
                    y = ty;
                    x = tx;
                    break;
                }
            }
        }

        (x, y)
    }

    fn is_opening_turn(&self) -> bool {
        for row in self.tiles.iter() {
            for tile in row {
                if let Tile::Letter(_) = tile {
                    return false;
                }
            }
        }

        true
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Play {
    word: String,
    points: u16,
    position: (usize, usize)
}

struct LetterScorer {
    points: HashMap<char, u16>,
}

impl LetterScorer {
    fn parse(path: &String) -> LetterScorer {
        let mut score = HashMap::new();
        let letterpoints = fs::read_to_string(path).unwrap();

        for line in letterpoints.split_terminator("\n") {
            let (c, points) = line.split_once(",").unwrap();

            score.insert(
                c.chars().nth(0).unwrap(),
                points.parse::<u16>().unwrap()
            );
        }

        LetterScorer { points: score }
    }

    fn score_with_board(&self,
                        word: &String,
                        letters: &String,
                        board: &ParsedBoard,
                        direction: char,
                        (x, y): (usize, usize)) -> u16 {

        let mut total_points = 0;
        let mut points_letters = letters.replace("?", "");
        let range = 0..word.len();

        let tiles: Vec<&Tile> = match direction {
            'H' => {
                let start_row = &board.tiles[y];
                range.map(|i| &start_row[x + i]).collect()
            },
            'V' => range.map(|i| &board.tiles[y + i][x]).collect(),
            _   => panic!("Invalid direction")
        };

        for (i, w) in word.chars().enumerate() {
            if points_letters.contains(w) {
                let multiplier = match tiles[i] {
                    Tile::DoubleLetter => 2,
                    Tile::TripleLetter => 3,
                    _ => 1
                };

                total_points += self.points[&w] * multiplier;
                points_letters = points_letters.replacen(&w.to_string(), "", 1);
            }
        }

        if tiles.contains(&&Tile::DoubleWord) {
            total_points *= 2;
        }

        if tiles.contains(&&Tile::TripleWord) {
            total_points *= 3;
        }

        total_points
    }

    fn score(&self, word: &String, letters: &String) -> u16 {
        let mut total_points = 0;
        let mut points_letters = letters.replace("?", "");

        for w in word.chars() {
            if points_letters.contains(w) {
                total_points += self.points[&w];
                points_letters = points_letters.replacen(&w.to_string(), "", 1);
            }
        }
        total_points
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
    fn test_anagrams_teers() {
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

        assert_eq!(board.anagrams(), vec![
            String::from("EERST"),
            String::from("ESTER"),
            String::from("RESET"),
            String::from("EET"),
            String::from("ER")
        ]);
    }

    #[test]
    #[serial]
    fn test_anagrams_joker() {
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

        assert_eq!(board.anagrams(), vec![
            String::from("EERST"),
            String::from("ESTER"),
            String::from("RESET"),
            String::from("STAAR"),
            String::from("STEUR"),
            String::from("EET"),
            String::from("ER"),
            String::from("MN"),
            String::from("ZE")
        ]);

        assert_eq!(board.optimal_plays(), vec![
            Play { word: String::from("EERST"), position: (7, 7), points: 12 },
            Play { word: String::from("EERST"), position: (3, 7), points: 12 },
        ]);
    }

    #[test]
    fn test_parse_board() {
        let layout_path = String::from("layout.default.board");
        let current_board_path = String::from("current.board");
        let board = ParsedBoard::parse(&layout_path, &current_board_path);

        assert_eq!(board.tiles.len(), 15);
        assert_eq!(board.tiles[0].len(), 15);
        assert_eq!(board.is_opening_turn(), true);
    }

    #[test]
    fn test_parse_board_not_opening() {
        let layout_path = String::from("layout.default.board");
        let current_board_path = String::from("data/test/test_simple.board");
        let board = ParsedBoard::parse(&layout_path, &current_board_path);

        assert_eq!(board.tiles.len(), 15);
        assert_eq!(board.tiles[0].len(), 15);
        assert_eq!(board.is_opening_turn(), false);
    }

    #[test]
    fn test_score_word_with_board() {
        let layout_path = String::from("layout.default.board");
        let current_board_path = String::from("current.board");
        let board = ParsedBoard::parse(&layout_path, &current_board_path);
        let lp_path = String::from("data/test/letterpoints.txt");
        let letter_scorer = LetterScorer::parse(&lp_path);

        // Regular word
        let word = String::from("TEST");
        let letters = String::from("TEST");
        let score = letter_scorer.score_with_board(&word, &letters, &board, 'H', (7, 7));

        assert_eq!(score, 7);

        let word = String::from("ZOUTIG");
        let letters = String::from("ZOUTIG");
        let score = letter_scorer.score_with_board(&word, &letters, &board, 'H', (7, 7));

        assert_eq!(score, 30);
    }

    #[test]
    fn test_score_word() {
        let lp_path = String::from("data/test/letterpoints.txt");
        let letter_scorer = LetterScorer::parse(&lp_path);

        // Regular word
        let word = String::from("TEST");
        let letters = String::from("TEST");
        assert_eq!(letter_scorer.score(&word, &letters), 7);

        // A joker is 0 points same character
        let word = String::from("TEST");
        let letters = String::from("TES?");
        assert_eq!(letter_scorer.score(&word, &letters), 5);

        // A joker is 0 points diff character
        let word = String::from("PEST");
        let letters = String::from("TES?");
        assert_eq!(letter_scorer.score(&word, &letters), 5);

        // Having a joker but not using it
        let word = String::from("PEST");
        let letters = String::from("TESP?");
        assert_eq!(letter_scorer.score(&word, &letters), 9);
    }
}
