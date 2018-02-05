extern crate futures;
extern crate html5ever;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;

use std::io;

use futures::{Future, Stream};
use html5ever::{parse_document, serialize};
use html5ever::driver::ParseOpts;
use html5ever::rcdom::RcDom;
use html5ever::tendril::TendrilSink;
use hyper::Client;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);
    let uri = "https://www.etymonline.com/".parse().unwrap();

    let work = client.get(uri).and_then(|res| {
        res.body().concat2().and_then(|body| {
            let opts = ParseOpts {
                ..Default::default()
            };
            let dom = parse_document(RcDom::default(), opts)
                .from_utf8()
                .read_from(&mut &*body)
                .unwrap();
            serialize(&mut io::stdout(), &dom.document, Default::default()).unwrap();
            Ok(())
        })
    });

    core.run(work).unwrap();
}
