extern crate reudh;

fn main() {
    println!("Version {}\n", env!("CARGO_PKG_VERSION"));
    match reudh::fetch() {
        Ok(_) => println!("\nSuccess!"),
        Err(err) => eprintln!("\nFailure: {}", err),
    }
}
