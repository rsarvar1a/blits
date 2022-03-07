
use crate::config::*;

use std::collections::BTreeSet;

use super::agent::*;
use super::elo::*;

use utils::*;

///
/// An environment in which a self-play tournament is conducted.
///
pub struct Selfplay 
{
    config: Config,
    agents: BTreeSet<Agent>
}

impl Selfplay 
{
}
