
use crate::config::*;

use lazy_static::lazy_static;

use std::sync::RwLock;

///
/// Represents a classic elo value.
///
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Elo
{
    val: f32
}

lazy_static! 
{
    static ref K : RwLock<f32> = RwLock::new(0.0);
    static ref ELO_INIT : RwLock<f32> = RwLock::new(0.0);
    static ref ELO_BOUND : RwLock<f32> = RwLock::new(0.0);
}

impl Elo 
{
    ///
    /// Applies an elo configuration.
    ///
    pub fn initialize (config: & SelfplayConfig)
    {
        * K.write().unwrap() = config.elo_k;
        * ELO_INIT.write().unwrap() = config.elo_init;
        * ELO_BOUND.write().unwrap() = config.elo_bound;
    }

    ///
    /// Returns a new Elo value.
    ///
    pub fn new () -> Elo 
    {
        Elo { val: ELO_INIT.read().unwrap().clone() }
    }

    ///
    /// Computes two new Elos given the loaded selfplay config.
    /// In LITS, draws are impossible, so the function doesn't handle them.
    ///
    pub fn update (lhs: & Elo, rhs: & Elo, result: bool) -> (Elo, Elo)
    {
        let k = K.read().unwrap().clone();
        let bound = ELO_BOUND.read().unwrap().clone();

        let r_lhs = 10.0_f32.powf(lhs.val / bound);
        let r_rhs = 10.0_f32.powf(rhs.val / bound);

        let e_lhs = r_lhs / (r_lhs + r_rhs);
        let e_rhs = 1.0 - e_lhs;

        let s_lhs = if result { 1.0 } else { 0.0 };
        let s_rhs = 1.0 - s_lhs;

        let v_lhs = 1.0_f32.max(lhs.val + k * (s_lhs - e_lhs));
        let v_rhs = 1.0_f32.max(rhs.val + k * (s_rhs - e_rhs));

        (Elo { val: v_lhs }, Elo { val: v_rhs })
    }
}

