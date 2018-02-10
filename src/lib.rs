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

pub fn fetch() -> Result<(), Error> {
    const LETTERS: &'static str = "abcdefghijklmnopqrstuvwxyz";

    let letter_urls = LETTERS
        .chars()
        .map(|l| format!("https://www.etymonline.com/search?q={}", l));

    let mut etyms = vec![];
    let bar = ProgressBar::new(26);
    bar.set_style(ProgressStyle::default_bar().template(
        "{prefix}\n[{elapsed_precise}/{eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    ));
    bar.set_prefix("fetching...");
    for url in letter_urls {
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
