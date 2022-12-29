use rusqlite::Connection;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::path::Path;

const ASCII_OFFSET: u8 = 65; // the letter 'A'
const LETTER_COUNT: usize = 26;
const BOARD_SIZE: usize = 15;
const BATCH_SIZE: usize = 1000;

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

    pub fn get_anagrams_for(&self, strings: &HashSet<String>) -> Vec<String> {
        for string in strings {
            if !self.valid_word(string) {
                panic!("Invalid letters given: {}", string);
            }
        }

        let conn = Connection::open(&self.db_path).unwrap();
        let mut prime_factors = vec![];

        for string in strings {
            let factor = self.prime_factor(string);
            prime_factors.push(factor);
        }

        let factors = prime_factors
            .iter()
            .map(|factor| factor.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        let q = format!("SELECT word FROM words WHERE prime_factor IN ({}) ORDER BY word", factors);
        let mut stmt = conn.prepare(&q).unwrap();
        let mut rows = stmt.query([]).unwrap();
        let mut anagrams = vec![];

        while let Some(row) = rows.next().unwrap() {
            let word: String = row.get(0).unwrap();
            anagrams.push(word);
        }

        anagrams
    }

    fn generated(&self) -> bool {
        Path::new(&self.db_path).is_file()
    }

    fn valid_word(&self, word: &String) -> bool {
        let mut valid_chars = true;
        for c in word.chars() {
            if !('A'..='Z').contains(&c) && c != '?' {
                valid_chars = false;
                break;
            }
        }

        valid_chars && word.len() <= BOARD_SIZE && word.len() > 1
    }

    fn setup_db(&self, wordlist_file: &String) {
        let f = File::open(wordlist_file).unwrap();
        let reader = BufReader::new(f);
        let conn = Connection::open(&self.db_path).unwrap();

        let mut batch = HashSet::new();

        self.create_db_schema(&conn);

        for line in reader.lines() {
            match line {
                Ok(word) => {
                    let mut cased_word = word.to_uppercase();
                    cased_word = cased_word.replace("'", "");

                    // Skip all the words with non-ASCII chars in them and the one's
                    // that are over the length
                    if !self.valid_word(&cased_word) {
                        continue
                    };

                    let product: u128 = self.prime_factor(&cased_word);
                    batch.insert((cased_word, product));
                },
                Err(e) => panic!("Something went wrong reading a line {}", e)
            }

            if batch.len() >= BATCH_SIZE {
                self.insert_batch_to_db(&conn, &mut batch);
            }
        }

        // Do the final batch
        self.insert_batch_to_db(&conn, &mut batch);
    }

    fn create_db_schema(&self, conn: &Connection) {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS words (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                word VARCHAR(15) NOT NULL UNIQUE,
                prime_factor BIGINT NOT NULL
            )",
            []
        ).unwrap();

        conn.execute(
            "CREATE INDEX IF NOT EXISTS prime_factor_index ON words (prime_factor)",
            []
        ).unwrap();
    }

    fn insert_batch_to_db(&self, conn: &Connection, batch: &mut HashSet<(String, u128)>) {
        let values: String = batch
            .iter()
            .map(|b| format!("(\"{}\", \"{}\")", b.0, b.1))
            .collect::<Vec<String>>()
            .join(",");

        // We use IGNORE here because the wordlist sometimes contains words like
        // Aaltjes en aaltjes. If "aaltjes" just so happened to be on the next batch,
        // this can result in a duplicate insert, which individual value(s) we
        // should just ignore.
        let query = format!(
            "INSERT OR IGNORE INTO words (word, prime_factor) VALUES {}",
            values
        );

        conn.execute(&query, []).unwrap_or_else(|e| {
            println!("FAILURE {}", e);
            0
        });

        batch.clear();
    }

    fn prime_factor(&self, word: &String) -> u128 {
        word
            .bytes()
            .map(|c| self.primes[(c - ASCII_OFFSET) as usize])
            .product()
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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;

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
        let mut set = HashSet::new();
        set.insert(word);
        assert_eq!(dictionary.get_anagrams_for(&set).len(), 0);

        let word = String::from("TEERS");
        let mut set = HashSet::new();
        set.insert(word);
        assert_eq!(
            dictionary.get_anagrams_for(&set),
            vec![
                String::from("EERST"),
                String::from("ESTER"),
                String::from("RESET")
            ]
        );

        let word = String::from("T??RS");
        let mut set = HashSet::new();
        set.insert(word);
        assert_eq!(
            dictionary.get_anagrams_for(&set),
            vec![
                String::from("EERST"),
                String::from("ESTER"),
                String::from("RESET"),
                String::from("STEUR"),
                String::from("STAAR")
            ]
        );
    }
}
