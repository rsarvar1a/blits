
use utils::{Serialize, Deserialize};

///
/// A configuration object for an MCTS manager.
///
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Config 
{
    #[serde(default = "num_threads")]
    pub num_threads: usize,

    #[serde(default = "max_time_ms")]
    pub max_time_ms: usize,

    #[serde(default = "discount")]
    pub discount: f32,

    #[serde(default = "uct_const")]
    pub uct_const: f32
}

impl Default for Config 
{
    fn default () -> Config 
    {
        Config 
        {
            num_threads: num_threads(),
            max_time_ms: max_time_ms(),
            discount: discount(),
            uct_const: uct_const()
        }
    }
}

fn num_threads () -> usize 
{
    2
}

fn max_time_ms () -> usize 
{
    5000
}

fn discount () -> f32 
{
    0.99
}

fn uct_const () -> f32 
{
    1.0
}
