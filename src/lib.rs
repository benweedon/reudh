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

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
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

pub fn fetch(reudh_path: PathBuf) -> Result<(), Error> {
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

    write_etyms_to_files(etym_receiver, reudh_path)?;
    let bar = bar_receiver
        .recv()
        .ok_or(Error::new("Progress bar not received"))?;
    bar.finish_and_clear();

    Ok(())
}

fn write_etyms_to_files(
    receiver: chan::Receiver<parse::Etym>,
    reudh_path: PathBuf,
) -> Result<(), Error> {
    if !reudh_path.exists() {
        fs::create_dir(&reudh_path)?;
    }
    let cache_path = reudh_path.join(PathBuf::from("cache"));
    if cache_path.exists() {
        fs::remove_dir_all(&cache_path)?;
    }
    fs::create_dir(&cache_path)?;

    for etym in receiver {
        let mut file = File::create(cache_path.join(PathBuf::from(&etym.word)))?;
        write!(file, "{}\n{}", etym.word, etym.definition)?;
    }
    Ok(())
}
