
use utils::*;

///
/// A player in the game The Battle of LITS.
///
/// There are two players, X and O. Each player has a corresponding set 
/// of scoring tiles on the gameboard, and the player's objective is to 
/// defend their tiles by keeping them uncovered while attacking their 
/// opponent's tiles by covering them up.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Player 
{
    X,
    O,
    None
}

impl std::fmt::Display for Player 
{
    fn fmt (& self, f: & mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let token = match self 
        {
            Player::X    => "❌".to_string(),
            Player::O    => "⭕".to_string(),
            Player::None => "⬛".to_string()
        };
        write!(f, "{}", token)
    }
}

impl notate::Notate for Player 
{
    fn notate (& self) -> String 
    {
        match self 
        {
            Player::X    => "X".to_string(),
            Player::O    => "O".to_string(),
            Player::None => "_".to_string()
        }
    }

    fn parse (s: & str) -> Result<Player>
    {
        match s 
        {
            "X" | "x"             => Ok(Player::X),
            "O" | "o"             => Ok(Player::O),
            "_" | "-" | "." | "," => Ok(Player::None),
            _                     => Err(error::error!("Invalid notation '{}' for player.", s))
        }
    }
}

impl Player 
{
    ///
    /// Returns the player opposite this one.
    ///
    pub fn next (& self) -> Player 
    {
        match self 
        {
            Player::X    => Player::O,
            Player::O    => Player::X,
            Player::None => panic!("Something has gone terribly wrong: tried to get next() of a null player.")
        }
    }

    ///
    /// Returns a length-2 one-hot encoding for this player, in XO order.
    ///
    pub fn one_hot (& self) -> Vec<bool>
    {
        vec![
            * self == Player::X,
            * self == Player::O
        ]
    }

    ///
    /// Returns the inherent value factor for this player, in terms of X's perspective.
    ///
    pub fn value (& self) -> f64
    {
        match self 
        {
            Player::X    =>  1.0,
            Player::O    => -1.0,
            Player::None =>  0.0 
        }
    }
}
