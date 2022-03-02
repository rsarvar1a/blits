
#![feature(thread_spawn_unchecked)]

mod config;
mod interfaces;
mod mcts;
mod neural;

use std::fs::OpenOptions;
use std::io::Read;

use clap::Parser;

use interfaces::*;
use lits::Tetromino;

use utils::*;

///
/// A structure representing command line arguments.
///
#[derive(Parser)]
struct CLIArgs 
{
    #[clap(short, long, default_value = "ltpi")]
    mode: String,

    #[clap(short, long, default_value = "config/config.toml")]
    config: String
}

fn main () -> Result<()>
{
    let args = CLIArgs::parse();

    let mut config_str = String::new();
    OpenOptions::new().read(true).open(& args.config)?.read_to_string(& mut config_str)?;
    let config : config::Config = toml::from_str(& config_str)?;

    Tetromino::initialize();
    let _logger = log::initialize(& config.log_path, "engine");

    match args.mode.as_str() 
    {
        "ltpi" => 
        {
            let mut ltpinterface = ltpi::LTPInterface::new(& config)?;
            ltpinterface.run_loop();
        },
        _ => 
        {
            return Err(error::error!("Mode '{}' is unsupported.", & args.mode)); 
        }
    };

    Ok(())
}
