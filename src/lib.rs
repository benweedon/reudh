extern crate futures;
extern crate html5ever;
extern crate hyper;
extern crate hyper_tls;
extern crate kuchiki;
extern crate native_tls;
extern crate tokio_core;

mod errors;
mod parse;

use errors::Error;

pub fn fetch() -> Result<(), Error> {
    const LETTERS: &'static str = "abcdefghijklmnopqrstuvwxyz";

    let letter_urls = LETTERS
        .chars()
        .map(|l| format!("https://www.etymonline.com/search?q={}", l));

    let mut etyms = vec![];
    for url in letter_urls {
        let e = parse::etyms_from_letter_url(url)?;
        etyms.extend(e);
    }

    for etym in etyms {
        println!("{}\n---------------------\n\n", etym);
    }
    Ok(())
}
