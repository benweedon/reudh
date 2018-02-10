extern crate chan;
extern crate futures;
extern crate html5ever;
extern crate hyper;
extern crate hyper_tls;
extern crate indicatif;
extern crate kuchiki;
extern crate native_tls;
extern crate num_cpus;
extern crate tokio_core;

mod errors;
mod parse;

use std::thread;

use errors::Error;

use indicatif::{ProgressBar, ProgressStyle};

struct PageIter {
    curr_letter: char,
}
impl PageIter {
    pub fn new() -> PageIter {
        PageIter { curr_letter: 'a' }
    }
}
impl Iterator for PageIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_letter <= 'z' {
            let url = format!("https://www.etymonline.com/search?q={}", self.curr_letter);
            self.curr_letter = (self.curr_letter as u8 + 1) as char;
            Some(url)
        } else {
            None
        }
    }
}

pub fn fetch() -> Result<(), Error> {
    let bar = ProgressBar::new(26);
    bar.set_style(ProgressStyle::default_bar().template(
        "{prefix}\n[{elapsed_precise}/{eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    ));
    bar.set_prefix("fetching...");

    let (page_sender, page_receiver) = chan::sync(1);
    let (etym_sender, etym_receiver) = chan::sync(1);
    let (bar_sender, bar_receiver) = chan::sync(1);

    thread::spawn(move || {
        let pages = PageIter::new();
        for url in pages {
            page_sender.send(url);
            bar.inc(1);
        }
        bar_sender.send(bar);
    });
    for _ in 0..num_cpus::get() {
        let page_receiver = page_receiver.clone();
        let etym_sender = etym_sender.clone();
        thread::spawn(move || loop {
            match page_receiver.recv() {
                Some(url) => {
                    let etyms = parse::etyms_from_letter_url(url).unwrap();
                    for etym in etyms {
                        etym_sender.send(etym);
                    }
                }
                None => return,
            }
        });
    }
    drop(page_receiver);
    drop(etym_sender);

    for etym in etym_receiver {
        println!("{}\n---------------------\n\n", etym);
    }
    let bar = bar_receiver
        .recv()
        .ok_or(Error::new("Progress bar not received"))?;
    bar.finish_and_clear();

    Ok(())
}
