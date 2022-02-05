use scraper::{Html, Selector};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // let words = ["sun", "there"];
    let input =
        PathBuf::from("/home/sschlinkert/code/common_word_list_maker/example_word_list.txt");
    let words = make_vec_from_filenames(&[input]);
    for word in words {
        println!("{} {:?}", word, get_homophones(&word));
    }
}

fn get_homophones(word: &str) -> Vec<String> {
    let url = "https://en.wiktionary.org/wiki/".to_owned() + word;
    let resp = reqwest::blocking::get(&url).unwrap();
    // println!(
    //     "Resp status is: {:?}",
    //     resp.status().canonical_reason().unwrap()
    // );
    assert!(resp.status().is_success());

    let html = resp.text().unwrap();
    let fragment = Html::parse_fragment(&html);
    let homophones_html = Selector::parse("span.homophones span a").unwrap();
    // Definitely a way to do this with `map` or something similar...
    let mut homophones: Vec<String> = vec![];

    for element in fragment.select(&homophones_html) {
        homophones.push(element.inner_html());
    }
    homophones
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

fn write_vec_to_file(vec: Vec<String>) {
    let output = "homophones.txt";
    let mut f = File::create(output).expect("Unable to create file");
    for string in vec {
        writeln!(f, "{}", string).expect("Unable to write word to file");
    }
}
