extern crate futures;
extern crate html5ever;
extern crate hyper;
extern crate hyper_tls;
extern crate kuchiki;
extern crate tokio_core;

mod errors;
mod parse;

use errors::Error;

use hyper::Client;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;

pub fn crawl() -> Result<(), Error> {
    let core = Core::new().unwrap();
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);

    let pages = parse::index_site(client, core)?;
    for page in pages {
        println!("{}", page);
    }
    Ok(())
}
