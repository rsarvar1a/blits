
use utils::{Serialize, Deserialize};

///
/// A configuration for the neural network policy agent.
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config 
{
    #[serde(default = "path")]
    pub path: String,

    #[serde(default = "template")]
    pub template: String,

    #[serde(default = "use_best")]
    pub use_best: bool,

    #[serde(default = "best")]
    pub best: String,

    #[serde(default = "learning_rate")]
    pub learning_rate: f32,

    #[serde(default = "loss_exp")]
    pub exp: f32,

    #[serde(default = "epochs")]
    pub epochs: i32
}

impl Default for Config 
{
    fn default () -> Config 
    {
        Config 
        {
            path: path(),
            template: template(),
            use_best: use_best(),
            best: best(),
            learning_rate: learning_rate(),
            exp: loss_exp(),
            epochs: epochs()
        }
    }
}

fn path () -> String 
{
    "models".to_owned()
}

fn template () -> String 
{
    "template.pt".to_owned()
}

fn use_best () -> bool 
{
    true
}

fn best () -> String 
{
    "best.pt".to_owned()
}

fn learning_rate () -> f32 
{
    0.00001
}

fn loss_exp () -> f32 
{
    1.5
}

fn epochs () -> i32 
{
    20
}
