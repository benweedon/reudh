#[macro_use]
extern crate clap;
extern crate reudh;

use std::env;

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
        match reudh::fetch(
            env::home_dir()
                .expect("home directory not found")
                .join(".reudh"),
        ) {
            Ok(_) => println!("\nSuccess!"),
            Err(err) => eprintln!("\nFailure: {}", err),
        }
    }
}
