#[macro_use]
extern crate clap;
extern crate reudh;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

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

    let reudh_dir = env::home_dir()
        .expect("home directory not found")
        .join(PathBuf::from(".reudh"));
    if !reudh_dir.exists() {
        if let Err(err) = fs::create_dir(&reudh_dir) {
            eprintln!("Failed to create reudh directory: {}", err);
            process::exit(1);
        }
    }

    if let Some(matches) = matches.subcommand_matches("fetch") {
        let default = reudh_dir.join(PathBuf::from("cache"));
        let default = default.to_str().unwrap();
        let cache_dir = matches.value_of("cache").unwrap_or(default);
        let cache_dir = PathBuf::from(cache_dir);

        if !matches.is_present("force") && cache_dir.exists() {
            eprintln!(
                "Cache dir {} already exists. Use -f to overwrite it.",
                cache_dir.to_str().unwrap()
            );
            process::exit(1);
        }

        match reudh::fetch(cache_dir) {
            Ok(_) => println!("\nSuccess!"),
            Err(err) => eprintln!("\nFailure: {}", err),
        }
    }
}
