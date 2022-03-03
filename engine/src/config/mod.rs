
use utils::{Serialize, Deserialize};

pub use crate::mcts::config::Config as MCTSConfig;
pub use crate::neural::config::Config as NeuralConfig;
pub use crate::interfaces::selfplay::config::Config as SelfplayConfig;

///
/// Represents a full configuration.
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config 
{
    #[serde(default)]
    pub mcts: MCTSConfig,

    #[serde(default)]
    pub neural: NeuralConfig,

    #[serde(default)]
    pub selfplay: SelfplayConfig,

    #[serde(default = "log_path")]
    pub log_path: String
}

///
/// Returns the default log path.
///
fn log_path () -> String 
{
    "logs".to_owned()
}

