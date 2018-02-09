extern crate reudh;

fn main() {
    println!("Version {}\n", env!("CARGO_PKG_VERSION"));
    match reudh::crawl() {
        Ok(_) => println!("Success!"),
        Err(err) => eprintln!("Failure: {}", err),
    }
}
