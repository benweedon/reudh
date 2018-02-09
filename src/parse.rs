use errors::Error;

use futures::{Future, Stream};
use html5ever::tendril::TendrilSink;
use hyper::{Body, Client};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use kuchiki::{parse_html, NodeRef};
use tokio_core::reactor::Core;

type HttpsClient = Client<HttpsConnector<HttpConnector>, Body>;

pub fn index_site(client: HttpsClient, mut core: Core) -> Result<Vec<String>, Error> {
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

fn get_dom(url: String, client: &HttpsClient, core: &mut Core) -> Result<NodeRef, Error> {
    let work = client.get(url.parse()?).and_then(|res| {
        res.body().concat2().and_then(|body| {
            let document = parse_html().from_utf8().read_from(&mut &*body)?;
            Ok(document)
        })
    });
    let document = core.run(work)?;
    Ok(document)
}
