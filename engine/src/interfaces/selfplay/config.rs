
use utils::{Serialize, Deserialize};

///
/// Represents a selfplay config.
///
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Config 
{
    #[serde(default = "elo_k")]
    pub elo_k: f32,

    #[serde(default = "elo_init")]
    pub elo_init: f32,

    #[serde(default = "elo_bound")]
    pub elo_bound: f32,

    #[serde(default = "num_agents")]
    pub num_agents: usize,

    #[serde(default = "rounds")]
    pub rounds: usize,

    #[serde(default = "match_length")]
    pub match_length: usize
}

impl Default for Config 
{
    fn default () -> Config 
    { 
        Config 
        {
            elo_k: elo_k(),
            elo_init: elo_init(),
            elo_bound: elo_bound(),
            num_agents: num_agents(),
            rounds: rounds(),
            match_length: match_length()
        }
    }
}

fn elo_k () -> f32 
{
    20.0
}

fn elo_init () -> f32 
{
    1000.0
}

fn elo_bound () -> f32 
{
    500.0
}

fn num_agents () -> usize 
{
    10
}

fn rounds () -> usize 
{
    20
}

fn match_length () -> usize 
{
    5
}
