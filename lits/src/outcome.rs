
///
/// An enum that represents the outcome of a game.
///
pub enum Outcome 
{
    X(f64),
    O(f64),
    InProgress,
    Draw
}

impl std::fmt::Display for Outcome 
{
    fn fmt (& self, f: & mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self 
        {
            Outcome::X(score)   => write!(f, "X wins by {}.", score),
            Outcome::O(score)   => write!(f, "O wins by {}.", - score),
            Outcome::InProgress => write!(f, "The game is in progress."),
            Outcome::Draw       => write!(f, "The game is a draw.")
        }
    }
}

