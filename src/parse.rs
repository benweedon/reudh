use std::error::Error;

use futures::{Future, Stream};
use html5ever::parse_document;
use html5ever::driver::ParseOpts;
use html5ever::rcdom::RcDom;
use html5ever::tendril::TendrilSink;
use hyper::{Body, Client};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;

type HttpsClient = Client<HttpsConnector<HttpConnector>, Body>;

pub fn index_site(client: HttpsClient, mut core: Core) -> Result<Vec<String>, Box<Error>> {
    const LETTERS: &'static str = "abcdefghijklmnopqrstuvwxyz";

    let letter_urls = LETTERS
        .chars()
        .map(|l| format!("https://www.etymonline.com/search?q={}", l));
    for url in letter_urls {
        get_dom(url, &client, &mut core)?;
        println!("hi");
    }
    Ok(vec![])
}

fn get_dom(url: String, client: &HttpsClient, core: &mut Core) -> Result<RcDom, Box<Error>> {
    let work = client.get(url.parse()?).and_then(|res| {
        res.body().concat2().and_then(|body| {
            let opts = ParseOpts {
                ..Default::default()
            };
            let dom = parse_document(RcDom::default(), opts)
                .from_utf8()
                .read_from(&mut &*body)?;
            Ok(dom)
        })
    });
    let dom = core.run(work)?;
    Ok(dom)
}
