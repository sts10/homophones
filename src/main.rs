use scraper::{Html, Selector};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;
use std::{thread, time};

fn main() {
    // let words = ["sun", "there"];
    let input = PathBuf::from("input_list.txt");
    let words = make_vec_from_filenames(&[input]);
    write_tuples_to_file(make_pairs_of_homophones(words));
}

fn make_pairs_of_homophones(input_words: Vec<String>) -> Vec<(String, String)> {
    let mut lines_to_print = vec![];
    for word in input_words {
        if let Some(list_of_homophones) = get_homophones(&word) {
            println!("{} {:?}", word, list_of_homophones);
            for homophone in list_of_homophones {
                lines_to_print.push((word.clone(), homophone));
            }
        }
    }
    lines_to_print
}

fn get_homophones(word: &str) -> Option<Vec<String>> {
    let url = "https://en.wiktionary.org/wiki/".to_owned() + word;
    let resp = match reqwest::blocking::get(&url) {
        Ok(r) => r,
        Err(e) => {
            let seconds_to_wait = 20;
            eprintln!(
                "Error scraping word '{}': {}\nWaiting {} seconds and then will try again",
                word, e, seconds_to_wait
            );
            thread::sleep(time::Duration::from_secs(seconds_to_wait));
            reqwest::blocking::get(&url).expect("Waited to scrape again, but still failed")
        }
    };
    // assert!(resp.status().is_success());
    if !resp.status().is_success() {
        return None;
    }

    let html = resp.text().unwrap();
    let fragment = Html::parse_fragment(&html);
    let homophones_html = Selector::parse("span.homophones span a").unwrap();
    // Definitely a way to do this with `map` or something similar...
    let mut homophones: Vec<String> = vec![];

    for element in fragment.select(&homophones_html) {
        homophones.push(element.inner_html());
    }
    if homophones.is_empty() {
        return None;
    } else {
        Some(homophones)
    }
}

/// Takes a slice of `PathBuf`s representing the word list(s)
/// that the user has inputted to the program. Then iterates
/// through each file and addes each line to Vec<String>. (Blank
/// lines and duplicate links will be handled elsewhere.)
pub fn make_vec_from_filenames(filenames: &[PathBuf]) -> Vec<String> {
    let mut word_list: Vec<String> = [].to_vec();
    for filename in filenames {
        let f = match File::open(filename) {
            Ok(file) => file,
            Err(e) => panic!("Error opening file {:?}: {}", filename, e),
        };
        let file = BufReader::new(&f);
        for line in file.lines() {
            let l = match line {
                Ok(l) => l,
                Err(e) => {
                    eprintln!(
                        "Error reading a line from file {:?}: {}\nWill continue reading file.",
                        filename, e
                    );
                    continue;
                }
            };
            word_list.push(l);
        }
    }
    word_list
}

fn write_tuples_to_file(vec: Vec<(String, String)>) {
    let output = "homophones.txt";
    let mut f = File::create(output).expect("Unable to create file");
    for tuple in vec {
        println!("Writing {} and {} to file", tuple.0, tuple.1);
        writeln!(f, "{},{}", tuple.0, tuple.1).expect("Unable to write word to file");
    }
}

fn _write_vec_to_file(vec: Vec<String>) {
    let output = "homophones.txt";
    let mut f = File::create(output).expect("Unable to create file");
    for string in vec {
        writeln!(f, "{}", string).expect("Unable to write word to file");
    }
}
