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
extern crate uuid;

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
use uuid::Uuid;

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
    let bar = Arc::new(ProgressBar::new(26));
    bar.set_style(ProgressStyle::default_bar().template(
        "{prefix}\n[{elapsed_precise}/{eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    ));
    bar.set_prefix("fetching...");

    let (page_sender, page_receiver) = chan::sync(1);

    let mut threads = vec![];
    threads.push(thread::spawn(move || {
        let pages = PageIter::new();
        for url in pages {
            page_sender.send(url);
        }
    }));

    if cache_dir.exists() {
        fs::remove_dir_all(&*cache_dir)?;
    }
    fs::create_dir(&*cache_dir)?;

    let cache_dir = Arc::new(cache_dir);
    for i in 0..num_cpus::get() {
        let page_receiver = page_receiver.clone();
        let cache_dir = Arc::clone(&cache_dir);
        let bar = Arc::clone(&bar);
        let thread = thread::Builder::new().name(i.to_string()).spawn(move || {
            let (mut core, client) = new_core_and_client().unwrap();
            for url in page_receiver {
                let etyms = parse::etyms_from_letter_url(url, &client, &mut core).unwrap();
                write_etyms_to_file(etyms, &*cache_dir).unwrap();
                bar.inc(1);
            }
        })?;
        threads.push(thread);
    }
    drop(page_receiver);

    for thread in threads {
        thread.join()?;
    }
    bar.finish_and_clear();

    Ok(())
}

fn write_etyms_to_file(etyms: Vec<parse::Etym>, cache_dir: &PathBuf) -> Result<(), Error> {
    if etyms.len() > 0 {
        let filename = Uuid::new_v4().simple().to_string();
        let mut file = File::create(cache_dir.join(PathBuf::from(filename)))?;
        let mut etyms = etyms.iter();
        let etym = etyms.next().unwrap();
        write!(file, "{}\n{}", etym.word, etym.definition)?;
        for etym in etyms {
            write!(file, "\n*\n{}\n{}", etym.word, etym.definition)?;
        }
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
