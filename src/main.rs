#[macro_use]
extern crate clap;
extern crate reudh;

use clap::{App, AppSettings};

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml)
        .name(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

    if let Some(_) = matches.subcommand_matches("fetch") {
        match reudh::fetch() {
            Ok(_) => println!("\nSuccess!"),
            Err(err) => eprintln!("\nFailure: {}", err),
        }
    }
}
