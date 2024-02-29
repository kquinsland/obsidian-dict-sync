use std::path::PathBuf;

use clap::Parser;

use crate::build_info;

#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
#[command(version = build_info::VERBOSE_VERSION)]
pub struct Args {
    // TODO: add support for XDG_CONFIG_HOME; likely will need to do this in main()
    // if not set, Should default to $HOME/.config/obsidian-dict-sync.
    #[arg(short, long, env = "ODS_CFG_FILE", default_value = "./config.toml")]
    /// Location of configuration.toml file.
    ///
    /// If not found, an example file with default values will be created at this location.
    ///
    pub config_file_path: Option<PathBuf>,

    #[arg(short, long, env = "ODS_LOG_VERBOSE")]
    /// Enable verbose logging
    pub verbose: bool,
}

//TODO: implement log level selection? For now, verbose on/off is good enough
