use scraper::{Html, Selector};

fn main() {
    let words = ["sun", "there"];
    for word in words {
        println!("{} {:?}", word, get_homophones(word));
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
