
pub mod appstate;
pub mod floatingtetromino;
pub mod ltpcommand;
pub mod ltpcontroller;
pub mod states;
pub mod view;

use clap::Parser;

use coffee::graphics::WindowSettings;
use coffee::ui::UserInterface;

use std::fs::OpenOptions;
use std::io::Read;

use ltpcontroller::LtpController;
use view::View;

use lits::*;
use utils::*;

///
/// A structure representing command line arguments.
///
#[derive(Parser)]
struct CLIArgs 
{
    #[clap(short, long, default_value = "/home/rsarvaria/Development/projects/blits/env/client.toml")]
    config: String
}

///
/// A structure representing the configuration file.
///
#[derive(Serialize, Deserialize)]
struct Config 
{
    log_path: String,
    exe_path: String
}

fn main() -> Result<()>
{
    // Use CLI args to determine the config file; if not found, 
    // fallback to the default configuration located in the XDG_CONFIG_DIR.

    let args = CLIArgs::parse();

    let mut config_str = String::new();
    OpenOptions::new().read(true).open(& args.config)?.read_to_string(& mut config_str)?;
    let config : Config = toml::from_str(& config_str)?;

    // Run any required global initializers.

    Tetromino::initialize();
    let _logger = log::initialize(& config.log_path, "client", "info, wgpu_core::device=warn")?;
    LtpController::initialize(& config.exe_path);

    // Create state and feed resources to application.
   
    let window_settings = WindowSettings
    {
        title: "The Battle of LITS".to_owned(),
        size: (950, 1000),
        resizable: true,
        fullscreen: false,
        maximized: false
    };

    <View as UserInterface>::run(window_settings)?;

    // Clean up.

    Ok(())
}
