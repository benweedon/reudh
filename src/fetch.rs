use std::fmt;

use errors::Error;

use futures::{Future, Stream};
use html5ever::tendril::TendrilSink;
use hyper::{Body, Client};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use kuchiki::{parse_html, NodeRef};
use tokio_core::reactor::Core;

pub type HttpsClient = Client<HttpsConnector<HttpConnector>, Body>;

#[derive(Debug)]
pub struct Etym {
    pub word: String,
    pub definition: String,
}

impl fmt::Display for Etym {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}\n{}", self.word, self.definition)
    }
}

pub fn etyms_from_letter_url(
    url: String,
    client: &HttpsClient,
    mut core: &mut Core,
) -> Result<Vec<Etym>, Error> {
    let document = get_dom(url, &client, &mut core)?;
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
                .ok_or(Error::new("href attribute not found on page"))?;
        let etym = etym_from_page(page, &client, &mut core)?;
        etyms.push(etym)
    }
    Ok(etyms)
}

fn etym_from_page(url: String, client: &HttpsClient, mut core: &mut Core) -> Result<Etym, Error> {
    let document = get_dom(url, &client, &mut core)?;
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
