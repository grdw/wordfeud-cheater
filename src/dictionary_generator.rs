use rusqlite::{Connection};
use serial_test::serial;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::path::Path;

const ASCII_OFFSET: u8 = 65; // the letter 'A'
const LETTER_COUNT: usize = 26;
const BOARD_SIZE: usize = 15;

fn is_prime(number: u8) -> bool {
    if number < 2 {
        return false
    }

    let mut is_prime: bool = true;
    let end = (number as f64).sqrt().floor() as u8;

    for i in 2..end+1 {
        if number % i == 0 {
            is_prime = false;
            break
        }
    }
    is_prime
}

fn generate_prime_numbers(length: usize) -> Vec<u128> {
    let mut i = 0;
    let mut list = vec![];

    loop {
        if is_prime(i) {
            list.push(i as u128);
        }

        if list.len() == length {
            break;
        }

        i += 1;
    }

    list
}

pub struct Dictionary {
    db_path: String,
    primes: Vec<u128>
}

impl Dictionary {
    fn new(db_path: String) -> Dictionary {
        let primes = generate_prime_numbers(LETTER_COUNT);

        Dictionary { db_path: db_path, primes: primes }
    }

    fn generated(&self) -> bool {
        Path::new(&self.db_path).is_file()
    }

    fn setup_db(&self, wordlist_file: &String) {
        let f = File::open(wordlist_file).unwrap();
        let reader = BufReader::new(f);
        let conn = Connection::open(&self.db_path).unwrap();

        // Setting up the table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS words (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                word VARCHAR(255) NOT NULL UNIQUE,
                prime_factor BIGINT NOT NULL
            )",
            []
        ).unwrap();

        conn.execute(
            "CREATE INDEX IF NOT EXISTS prime_factor_index ON words (prime_factor)",
            []
        ).unwrap();

        for line in reader.lines() {
            match line {
                Ok(word) => {
                    let mut cased_word = word.to_uppercase();
                    let mut valid_word = true;
                    cased_word = cased_word.replace("'", "");

                    for c in cased_word.chars() {
                        if !('A'..='Z').contains(&c) {
                            valid_word = false;
                            break;
                        }
                    }

                    // Skip all the words with non-ASCII chars in them and the one's
                    // that are over the length
                    if !valid_word || cased_word.len() > BOARD_SIZE {
                        continue
                    };

                    let product: u128 = self.prime_factor(&cased_word);

                    conn.execute(
                        "INSERT INTO words (word, prime_factor) VALUES (?1, ?2)",
                        &[&cased_word, &product.to_string()]
                    ).unwrap();
                },
                Err(e) => panic!("Boom! {}", e)
            }
        }
    }

    fn prime_factor(&self, word: &String) -> u128 {
        word
            .bytes()
            .map(|c| self.primes[(c - ASCII_OFFSET) as usize])
            .product()
    }

    pub fn get_anagrams_for(&self, string: &String) -> Vec<String> {
        let conn = Connection::open(&self.db_path).unwrap();
        let factor = self.prime_factor(string);

        let mut stmt = conn.prepare("SELECT word FROM words WHERE prime_factor = ?").unwrap();
        let mut rows = stmt.query([factor.to_string()]).unwrap();
        let mut anagrams = vec![];

        while let Some(row) = rows.next().unwrap() {
            let word: String = row.get(0).unwrap();
            anagrams.push(word);
        }

        anagrams
    }
}

pub fn generate(path: String) -> Dictionary {
    let wordlist_file = format!("{}/wordlist.txt", path);
    if !Path::new(&wordlist_file).is_file() {
        panic!(
            "The 'wordlist.txt' file doesn't exist at '{}'",
            wordlist_file
        );
    }

    let db_file = format!("{}/dictionary.sqlite", path);
    let dictionary = Dictionary::new(db_file);

    if dictionary.generated() {
        return dictionary;
    }

    dictionary.setup_db(&wordlist_file);
    dictionary
}

#[test]
#[should_panic]
fn test_panic_generate() {
    let base_path = String::from("data/does-not-exist");
    generate(base_path);
}

#[test]
#[serial]
fn test_success_generate() {
    // Drop the existing db first
    let db_file = String::from("data/test/dictionary.sqlite");
    if Path::new(&db_file).is_file() {
        fs::remove_file(db_file).unwrap();
    }

    let base_path = String::from("data/test");
    let dictionary = generate(base_path);

    assert_eq!(dictionary.db_path, String::from("data/test/dictionary.sqlite"));

    // The 2nd time it fetches it from cache
    let base_path = String::from("data/test");
    let dictionary = generate(base_path);
    assert_eq!(dictionary.db_path, String::from("data/test/dictionary.sqlite"));
}

#[test]
#[serial]
fn get_anagrams() {
    // Drop the existing db first
    let db_file = String::from("data/test/dictionary.sqlite");
    if Path::new(&db_file).is_file() {
        fs::remove_file(db_file).unwrap();
    }

    let base_path = String::from("data/test");
    let dictionary = generate(base_path);

    let word = String::from("XYZ");
    let word2 = String::from("TEERS");
    assert_eq!(dictionary.get_anagrams_for(&word).len(), 0);
    assert_eq!(
        dictionary.get_anagrams_for(&word2),
        vec![
            String::from("EERST"),
            String::from("ESTER"),
            String::from("RESET")
        ]
    );
}
