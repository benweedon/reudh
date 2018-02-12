use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use errors::Error;

use chan;
use futures::{Future, Stream};
use html5ever::tendril::TendrilSink;
use hyper::{Body, Client};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use indicatif::{ProgressBar, ProgressStyle};
use kuchiki::{parse_html, NodeRef};
use num_cpus;
use tokio_core::reactor::Core;
use uuid::Uuid;

type HttpsClient = Client<HttpsConnector<HttpConnector>, Body>;

#[derive(Debug)]
struct Etym {
    word: String,
    definition: String,
}
impl fmt::Display for Etym {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}\n{}", self.word, self.definition)
    }
}

struct PageIter {
    curr_letter: char,
    curr_page: u64,
    letter_page_counts: HashMap<char, u64>,
}
impl PageIter {
    fn new() -> PageIter {
        PageIter {
            curr_letter: 'a',
            curr_page: 1,
            letter_page_counts: HashMap::new(),
        }
    }

    fn estimate_length(&self) -> u64 {
        self.letter_page_counts.values().sum()
    }

    fn initialize(&mut self, bar: &ProgressBar) -> Result<(), Error> {
        let mut letter = 'a';
        while letter <= 'z' {
            let url = format!("https://www.etymonline.com/search?q={}", letter);
            self.update_page_count(url, &letter)?;
            letter = (letter as u8 + 1) as char;
            bar.inc(1);
        }
        Ok(())
    }

    fn update_page_count(&mut self, url: String, letter: &char) -> Result<(), Error> {
        let (mut core, client) = new_core_and_client()?;
        let document = get_dom(&url, &client, &mut core)?;
        let selected = document.select(".ant-pagination-item")?;
        let page_nums = selected
            .map(|item| {
                let elt = item.as_node()
                    .as_element()
                    .ok_or(Error::new("Link is not element"))?;
                let attrs = elt.attributes.borrow();
                let title = attrs
                    .get("title")
                    .ok_or(Error::new("title attribute not found"))?;
                let title = title.parse()?;
                Ok(title)
            })
            .collect::<Result<Vec<u64>, Error>>()?;
        let num_pages = page_nums
            .iter()
            .max()
            .ok_or(Error::new("maximum page number not found"))?;
        self.letter_page_counts.insert(*letter, *num_pages);
        Ok(())
    }
}
impl Iterator for PageIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_letter <= 'z' {
            let url = format!(
                "https://www.etymonline.com/search?q={}&page={}",
                self.curr_letter, self.curr_page
            );
            self.curr_page += 1;
            if self.curr_page > self.letter_page_counts[&self.curr_letter] {
                self.curr_page = 1;
                self.curr_letter = (self.curr_letter as u8 + 1) as char;
            }
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
    bar.set_prefix("indexing...");

    let (page_sender, page_receiver) = chan::sync(1);

    let mut threads = vec![];
    let mut pages = PageIter::new();
    pages.initialize(&*bar)?;
    bar.set_position(0);
    bar.set_prefix("fetching...");
    let pages = Arc::new(Mutex::new(pages));
    {
        let pages = pages.clone();
        threads.push(thread::spawn(move || loop {
            let url;
            {
                let mut pages = pages.lock().unwrap();
                match pages.next() {
                    Some(u) => url = u,
                    None => break,
                }
            }
            page_sender.send(url);
        }));
    }

    if cache_dir.exists() {
        fs::remove_dir_all(&*cache_dir)?;
    }
    fs::create_dir(&*cache_dir)?;

    let cache_dir = Arc::new(cache_dir);
    for i in 0..num_cpus::get() {
        let page_receiver = page_receiver.clone();
        let cache_dir = Arc::clone(&cache_dir);
        let bar = Arc::clone(&bar);
        let pages = pages.clone();
        let thread = thread::Builder::new().name(i.to_string()).spawn(move || {
            let (mut core, client) = new_core_and_client().unwrap();
            let mut etyms = vec![];
            for url in page_receiver {
                let curr_etyms = etyms_from_letter_url(url, &client, &mut core).unwrap();
                etyms.extend(curr_etyms);
                // Try to make files contain at least 100 etyms.
                if etyms.len() >= 100 {
                    write_etyms_to_file(&etyms, &*cache_dir).unwrap();
                    etyms.clear();
                }
                {
                    let pages = pages.lock().unwrap();
                    bar.set_length(pages.estimate_length());
                }
                bar.inc(1);
            }
            // Write out the remaining etyms in the buffer.
            write_etyms_to_file(&etyms, &*cache_dir).unwrap();
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

fn etyms_from_letter_url(
    url: String,
    client: &HttpsClient,
    mut core: &mut Core,
) -> Result<Vec<Etym>, Error> {
    let document = get_dom(&url, &client, &mut core)?;
    let mut etyms = vec![];
    // Select all <a> tags with class beginning with "word--".
    let selection = document.select("a[class^=word--]")?;
    for selected in selection {
        let elt = selected
            .as_node()
            .as_element()
            .ok_or(Error::new("Link is not element"))?;
        let attrs = elt.attributes.borrow();
        let page = "https://www.etymonline.com".to_owned()
            + attrs
                .get("href")
                .ok_or(Error::new("href attribute not found on link"))?;
        let etym = etym_from_page(page, &client, &mut core)?;
        etyms.push(etym)
    }
    Ok(etyms)
}

fn etym_from_page(url: String, client: &HttpsClient, mut core: &mut Core) -> Result<Etym, Error> {
    let document = get_dom(&url, &client, &mut core)?;
    let word = document
        .select_first("h1[class^=word__name--]")?
        .text_contents();
    let definition = document
        .select_first("section[class^=word__defination--]")?
        .text_contents();
    Ok(Etym {
        word: word,
        definition: definition,
    })
}

fn write_etyms_to_file(etyms: &Vec<Etym>, cache_dir: &PathBuf) -> Result<(), Error> {
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

fn get_dom(url: &String, client: &HttpsClient, core: &mut Core) -> Result<NodeRef, Error> {
    let work = client.get(url.parse()?).and_then(|res| {
        res.body().concat2().and_then(|body| {
            let document = parse_html().from_utf8().read_from(&mut &*body)?;
            Ok(document)
        })
    });
    let document = core.run(work)?;
    Ok(document)
}

fn new_core_and_client() -> Result<(Core, HttpsClient), Error> {
    let core = Core::new()?;
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle)?)
        .build(&handle);

    Ok((core, client))
}
