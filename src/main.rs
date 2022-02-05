extern crate reqwest;
extern crate select;

use scraper::{Html, Selector};

fn main() {
    // scrape("https://news.ycombinator.com");
    scrape("https://en.wiktionary.org/wiki/sun");
    scrape("https://en.wiktionary.org/wiki/there");
}

fn scrape(url: &str) {
    let resp = reqwest::blocking::get(url).unwrap();
    println!(
        "Resp status is: {:?}",
        resp.status().canonical_reason().unwrap()
    );
    // assert!(resp.status().is_success());

    let html = resp.text().unwrap();
    // println!("HTML is {}", html);
    let fragment = Html::parse_fragment(&html);
    let _link_selector = Selector::parse("a").unwrap();
    let homophones = Selector::parse("span.homophones span a").unwrap();
    for element in fragment.select(&homophones) {
        println!("{}", element.inner_html());
    }
    //
    // Document::from_read(resp)
    //     .unwrap()
    //     .find(Name("a"))
    //     .filter_map(|n| n.attr("href"))
    //     .for_each(|x| println!("{}", x));
}
