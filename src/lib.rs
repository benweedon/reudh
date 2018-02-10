extern crate futures;
extern crate html5ever;
extern crate hyper;
extern crate hyper_tls;
extern crate indicatif;
extern crate kuchiki;
extern crate native_tls;
extern crate tokio_core;

mod errors;
mod parse;

use errors::Error;

use indicatif::{ProgressBar, ProgressStyle};

const LETTERS: &'static str = "abcdefghijklmnopqrstuvwxyz";

struct PageIter {
    curr_letter_index: usize,
}
impl PageIter {
    pub fn new() -> PageIter {
        PageIter {
            curr_letter_index: 0,
        }
    }
}
impl Iterator for PageIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_letter_index < 26 {
            let url = format!(
                "https://www.etymonline.com/search?q={}",
                LETTERS.as_bytes()[self.curr_letter_index] as char
            );
            self.curr_letter_index += 1;
            Some(url)
        } else {
            None
        }
    }
}

pub fn fetch() -> Result<(), Error> {
    let mut etyms = vec![];
    let bar = ProgressBar::new(26);
    bar.set_style(ProgressStyle::default_bar().template(
        "{prefix}\n[{elapsed_precise}/{eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    ));
    bar.set_prefix("fetching...");

    let pages = PageIter::new();
    for url in pages {
        let e = parse::etyms_from_letter_url(url)?;
        etyms.extend(e);
        bar.inc(1);
    }
    bar.finish_and_clear();

    for etym in etyms {
        println!("{}\n---------------------\n\n", etym);
    }
    Ok(())
}
