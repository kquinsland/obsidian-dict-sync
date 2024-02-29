mod build_info;
mod cli;
mod config;
mod user_dictionary;

#[macro_use]
extern crate log;

use crate::config::Config;
use crate::user_dictionary::UserDictionary;
use clap::Parser;
use env_logger::{Builder, Target};
use log::{debug, info};
use std::collections::HashSet;

fn main() {
    let args = cli::Args::parse();
    setup_logging(args.verbose);

    info!("Obsidian Dictionary Sync v{}.", env!("CARGO_PKG_VERSION"));

    debug!(
        "{} built by '{}' from '{}'",
        build_info::FULL_VERSION,
        build_info::BUILD_ENV_STR,
        build_info::BUILD_SRC_STR
    );

    // Get the current working directory and config file path for relative path fixing in a moment...
    let cfg_file_path = args.config_file_path.unwrap();
    info!(
        "Loading config file from: {}",
        cfg_file_path.to_str().unwrap()
    );

    let cwd = std::env::current_dir().unwrap();
    debug!("cwd: {}", cwd.to_str().unwrap());

    // Render/Parse config file
    let config = Config::new_from_file(&cfg_file_path);
    debug!("Parsed config: {:#?}", &config);

    // Load up the authoritative dictionary
    if !config.dictionaries.contains_key("authoritative") {
        panic!("The config file must have a dictionary named `authoritative` present!");
    }

    // Create the authoritative dictionary
    let mut authoritative_dict =
        UserDictionary::new_from_pathbuf(config.get_authoritative_dictionary_path())
            .unwrap_or_else(|err| {
                panic!("Could not open authoritative dictionary: {}", err);
            });

    debug!("authoritative_dict: {:#?}", authoritative_dict);
    info!(
        "Authoritative Dictionary has {} words",
        authoritative_dict
            .words
            .as_ref()
            .expect("Failure to get auth-dict words!")
            .len()
    );
    // Keep track of which dictionaries we found on disk; we'll have to write combined authoritative list to these
    let mut user_dictionaries: Vec<UserDictionary> = Vec::new();
    for (name, data) in config.dictionaries.iter() {
        info!("Processing dictionary: {}", name);
        // Toml gives us a string
        let dict_path = data
            .as_table()
            .unwrap()
            .get("path")
            .unwrap()
            .as_str()
            .unwrap();

        debug!("dictionary '{}' is located at '{}'...", name, dict_path);
        let user_dictionary = UserDictionary::new_from_file_path(dict_path);
        match user_dictionary {
            Err(e) => {
                warn!("Could not parse dictionary from '{}': {}", dict_path, e);
                continue;
            }
            Ok(ud) => {
                debug!("user_dictionary: {:#?}", ud);
                authoritative_dict
                    // TODO: Is there a way to do this w/o clone()?
                    .add_words(ud.words.clone().unwrap());
                user_dictionaries.push(ud);
            }
        }
    }
    debug!("Found '{}' user dictionaries...", user_dictionaries.len());
    // After loading in all words from all dictionaries, remove filtered words from the authoritative dictionary
    debug!("config.filters.remove: {:#?}", config.filters.remove);

    // TODO: figure out how to do the conversion on config parse so filters.remove is already
    // proper type / doesn't need conversion?
    authoritative_dict.remove_words(HashSet::from_iter(config.filters.remove));

    // Write the authoritative dictionary to disk
    debug!("authoritative_dict => '{}' ", &authoritative_dict);
    authoritative_dict.write_to_disk().unwrap();

    // Iterate through the dictionary file(s) we did find on disk and write the authoritative dictionary to them
    for mut user_dict in user_dictionaries {
        // TODO: Is there a way to do this w/o clone()? At this point in code flow, the authoritative dictionary
        // is fixed and will not change.
        user_dict.set_words(authoritative_dict.words.clone().unwrap());
        user_dict.write_to_disk().unwrap();
    }
    info!(
        "Done! All dictionaries have been written to disk with '{}' words.",
        authoritative_dict.words.as_ref().unwrap().len()
    );
}

fn setup_logging(user_verbose: bool) {
    let mut builder = Builder::from_default_env();
    if user_verbose {
        builder.filter_level(log::LevelFilter::Debug);
    } else {
        builder.filter_level(log::LevelFilter::Info);
    }
    builder.target(Target::Stdout);
    builder.init();
}
