
use crate::config::*;
use crate::mcts::mcts::MCTS;

use super::elo::Elo;

use utils::error::*;

///
/// Represents a player in a selfplay, which is a rated MCTS instance.
///
pub struct Agent 
{
    mcts: MCTS,
    config: Config,
    elo: Elo
}

impl Agent 
{
    ///
    /// Creates a new agent. Agents are always created from the template model.
    ///
    pub fn new (config: & Config) -> Result<Agent>
    {
        let mut config = config.clone();
        config.neural.use_best = false;

        let mcts = MCTS::new(config.clone())?;
        Ok(Agent { mcts, config, elo: Elo::new() })
    }
}
