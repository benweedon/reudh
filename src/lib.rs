extern crate futures;
extern crate html5ever;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;

mod parse;

use std::error::Error;

use hyper::Client;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;

pub fn crawl() -> Result<(), Box<Error>> {
    let core = Core::new().unwrap();
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);

    parse::index_site(client, core)?;
    Ok(())
}
