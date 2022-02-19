use clap::Parser;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::{thread, time};

/// Generate lists of homophones
#[derive(Parser, Debug)]
#[clap(version, about, name = "homophones")]
struct Args {
    /// Path for outputted file for list of PAIRS of homohpones.
    #[clap(short = 'p', long = "pairs", parse(from_os_str))]
    pairs_output: Option<PathBuf>,

    /// Path for outputted file for list of SINGLE homohpones.
    #[clap(short = 's', long = "singles", parse(from_os_str))]
    singles_output: Option<PathBuf>,

    /// Force overwrite of output file if it exists.
    #[clap(short = 'f', long = "force")]
    force_overwrite: bool,

    /// Word list input files. Can be more than one.
    #[clap(name = "Inputted Word Lists", parse(from_os_str), required = true)]
    inputted_word_lists: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();
    if !validate(&args) {
        return;
    }

    let words = make_vec_from_filenames(&args.inputted_word_lists);

    // This make_pairs_of_homophones does the actual web-scraping
    // so we only want to run it once
    let pairs_of_homophones = make_pairs_of_homophones(&words);
    if let Some(ref pairs_output) = args.pairs_output {
        write_tuples_to_file(&pairs_of_homophones, pairs_output);
    }
    if let Some(ref singles_output) = args.singles_output {
        write_vec_to_file(singularize(&pairs_of_homophones), singles_output);
    }
}

fn validate(args: &Args) -> bool {
    match (&args.pairs_output, &args.singles_output) {
        (Some(pairs_output), Some(singles_output)) => {
            if !args.force_overwrite
                && (Path::new(&pairs_output).exists() || Path::new(&singles_output).exists())
            {
                eprintln!(
                    "One of the specified output files already exists. Use --force flag to force an overwrite."
                );
                return false;
            }
        }
        (Some(pairs_output), None) => {
            if !args.force_overwrite && Path::new(&pairs_output).exists() {
                eprintln!(
                    "Specified output file already exists. Use --force flag to force an overwrite."
                );
                return false;
            }
        }
        (None, Some(singles_output)) => {
            if !args.force_overwrite && Path::new(&singles_output).exists() {
                eprintln!(
                    "Specified output file already exists. Use --force flag to force an overwrite."
                );
                return false;
            }
        }
        (None, None) => {
            eprintln!("Error: Must specify an output file location for either a list of homophone pairs (-p) or list of single homophones (-s)");
            return false;
        }
    }
    true
}

fn singularize(pairs_of_homophones: &[(String, String)]) -> Vec<String> {
    let mut homophones = vec![];
    for pair in pairs_of_homophones {
        homophones.push(pair.0.clone());
        homophones.push(pair.1.clone());
    }
    sort_and_dedup(&mut homophones)
}

fn make_pairs_of_homophones(input_words: &[String]) -> Vec<(String, String)> {
    let mut tuples_of_homophones = vec![];
    for word in input_words {
        if let Some(list_of_homophones) = get_homophones(word) {
            for homophone in list_of_homophones {
                tuples_of_homophones.push((word.clone(), homophone));
            }
        }
    }
    tuples_of_homophones
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
    if !resp.status().is_success() {
        return None;
    }

    let html = resp.text().unwrap();
    let fragment = Html::parse_fragment(&html);
    let homophones_html = Selector::parse("span.homophones span a").unwrap();
    // Definitely a way to do this with `map` or something similar...
    let mut homophones: Vec<String> = vec![];

    for element in fragment.select(&homophones_html) {
        homophones.push(element.inner_html().trim().to_string());
    }
    if homophones.is_empty() {
        None
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
    // word_list
    sort_and_dedup(&mut word_list)
}

/// Alphabetizes and de-duplicates a Vector of `String`s.
///
/// For Rust's [`dedup()`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.dedup)
/// function to remove all duplicates, the Vector needs to be
/// [`sort()`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.sort)ed first.
fn sort_and_dedup(list: &mut Vec<String>) -> Vec<String> {
    list.sort();
    list.dedup();
    list.to_vec()
}

fn write_tuples_to_file(vec: &[(String, String)], output: &Path) {
    // let output = "homophone_pairs.txt";
    let mut f = File::create(output).expect("Unable to create file");
    for tuple in vec {
        writeln!(f, "{},{}", tuple.0, tuple.1).expect("Unable to write word to file");
    }
}

fn write_vec_to_file(vec: Vec<String>, output: &Path) {
    // let output = "homophones.txt";
    let mut f = File::create(output).expect("Unable to create file");
    for string in vec {
        writeln!(f, "{}", string).expect("Unable to write word to file");
    }
}
