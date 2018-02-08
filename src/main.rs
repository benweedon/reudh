extern crate reudh;

fn main() {
    println!("Version {}\n", env!("CARGO_PKG_VERSION"));
    reudh::crawl();
}
