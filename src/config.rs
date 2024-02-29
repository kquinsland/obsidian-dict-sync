use log::{debug, error};
use serde::Deserialize;
use std::fs::File;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::{fs, process::exit};
use toml::{self, Table};

/// Top level structure to hold entirety of the config file
#[derive(Deserialize, Debug)]
pub struct Config {
    // TODO: instead of Table, this should be Dictionary type :/
    pub dictionaries: Table,
    pub filters: Filters,
}
/// Each dictionary has a name/path.
/// The dictionary named "authoritative" will be the source of truth
// #[derive(Deserialize, Debug)]
// pub struct Dictionary {
//     name: String,
//     path: String,
// }

/// User can supply a list of words that they do not want in the final results
#[derive(Deserialize, Debug)]
pub struct Filters {
    pub remove: Vec<String>,
}

const EXAMPLE_CONFIG_FILE: &str = include_str!("../config/example.toml");

// TODO: tests!
impl Config {
    pub fn new_from_file(file_path: &PathBuf) -> Self {
        // TODO: support shell expansion with shellexpand
        let file_path_str = file_path.to_str().unwrap();

        debug!("reading '{}'", file_path_str);

        // Would be nice to File::create_new but it's unstable :(
        let content = fs::read_to_string(file_path).unwrap_or_else(|err| {
            if err.kind() == ErrorKind::NotFound {
                warn!("Config file at '{}' not found. Creating....", file_path_str);

                File::create(file_path).unwrap_or_else(|err| {
                    error!("Could not create '{}'. Error: {}", file_path_str, err);
                    exit(1);
                });

                fs::write(file_path, EXAMPLE_CONFIG_FILE).unwrap_or_else(|err| {
                    error!("Failure to create '{}'/ Error: {}", file_path_str, err)
                });
                String::from(EXAMPLE_CONFIG_FILE)
            } else {
                error!("could not open '{}': {}", file_path_str, err);
                exit(1);
            }
        });

        toml::from_str(&content).unwrap()
    }

    /// Gets canonical path to the authoritative dictionary
    // TODO: tests!
    pub fn get_authoritative_dictionary_path(&self) -> PathBuf {
        let raw_path = self
            .dictionaries
            .get("authoritative")
            .unwrap()
            .as_table()
            .unwrap()
            .get("path")
            .unwrap()
            .as_str()
            .unwrap();

        let path = PathBuf::from(raw_path).canonicalize();
        match path {
            Ok(p) => {
                debug!("canonical path: {}", p.to_str().unwrap());
                p
            }
            Err(err) => {
                // Ideally we'd be able to canonicalize() w/o testing that the file exists so we'd pass the cannon path into the create
                // call. Instead, we pass in the raw path since i'm not writing my own canonicalize() function that doesn't check for
                //  file existence.
                if err.kind() == ErrorKind::NotFound {
                    warn!(
                        "Authoritative dictionary '{}' not found. Creating....",
                        raw_path
                    );
                    File::create(raw_path).unwrap_or_else(|err| {
                        panic!("Could not create '{}'. Error: {}", raw_path, err);
                    });
                    PathBuf::from(raw_path).canonicalize().unwrap()
                } else {
                    panic!("Other failure opening '{}': {}", raw_path, err);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: need to pull in some mock-fs crate or just include a test config file and sample dictionary files
    // That way I can also test out hash gen
    #[test]
    fn test_new_from_file() {
        let cfg = Config::new_from_file(&PathBuf::from("config/example.toml"));
        // TODO: check actual content, not just the length :)
        assert_eq!(cfg.dictionaries.len(), 5);
        assert_eq!(cfg.filters.remove.len(), 4);
    }
}
