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
use std::sync::Arc;
use std::thread;

use errors::Error;

use hyper::Client;
use hyper_tls::HttpsConnector;
use indicatif::{ProgressBar, ProgressStyle};
use tokio_core::reactor::Core;

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

pub fn fetch(cache_dir: PathBuf) -> Result<(), Error> {
    let bar = ProgressBar::new(26);
    bar.set_style(ProgressStyle::default_bar().template(
        "{prefix}\n[{elapsed_precise}/{eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    ));
    bar.set_prefix("fetching...");

    let (page_sender, page_receiver) = chan::sync(1);
    let (bar_sender, bar_receiver) = chan::sync(1);

    let mut threads = vec![];
    threads.push(thread::spawn(move || {
        let pages = PageIter::new();
        for url in pages {
            page_sender.send(url);
            bar.inc(1);
        }
        bar_sender.send(bar);
    }));

    if cache_dir.exists() {
        fs::remove_dir_all(&*cache_dir)?;
    }
    fs::create_dir(&*cache_dir)?;

    let cache_dir = Arc::new(cache_dir);
    for i in 0..num_cpus::get() {
        let page_receiver = page_receiver.clone();
        let cache_dir = Arc::clone(&cache_dir);
        let thread = thread::Builder::new().name(i.to_string()).spawn(move || {
            let (mut core, client) = new_core_and_client().unwrap();
            for url in page_receiver {
                let etyms = parse::etyms_from_letter_url(url, &client, &mut core).unwrap();
                write_etyms_to_files(etyms, &*cache_dir).unwrap();
            }
        })?;
        threads.push(thread);
    }
    drop(page_receiver);

    for thread in threads {
        thread.join()?;
    }
    let bar = bar_receiver
        .recv()
        .ok_or(Error::new("Progress bar not received"))?;
    bar.finish_and_clear();

    Ok(())
}

fn write_etyms_to_files(etyms: Vec<parse::Etym>, cache_dir: &PathBuf) -> Result<(), Error> {
    for etym in etyms {
        let mut file = File::create(cache_dir.join(PathBuf::from(&etym.word)))?;
        write!(file, "{}\n{}", etym.word, etym.definition)?;
    }
    Ok(())
}

fn new_core_and_client() -> Result<(Core, parse::HttpsClient), Error> {
    let core = Core::new()?;
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle)?)
        .build(&handle);

    Ok((core, client))
}
