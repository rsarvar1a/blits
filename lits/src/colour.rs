
use utils::*;

///
/// A piecetype in the game The Battle of LITS.
/// 
/// There are four piece shapes, namely L, I, T, and S. The piece pool in 
/// The Battle of LITS is a shared piece pool, consisting of five copies 
/// of each piece type.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Colour
{
    L,
    I,
    T,
    S,
    None
}

impl std::fmt::Display for Colour
{
    fn fmt (& self, f: & mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        let token = match self 
        {
            Colour::L    => "🟥".to_string(),
            Colour::I    => "🟨".to_string(),
            Colour::T    => "🟩".to_string(),
            Colour::S    => "🟦".to_string(),
            Colour::None => "⬛".to_string()
        };
        write!(f, "{}", token)
    }
}

impl notate::Notate for Colour 
{
    fn notate (& self) -> String 
    {
        match self 
        {
            Colour::L    => "L".to_string(),
            Colour::I    => "I".to_string(),
            Colour::T    => "T".to_string(),
            Colour::S    => "S".to_string(),
            Colour::None => "_".to_string()
        }
    }

    fn parse (s: & str) -> Result<Colour>
    {
        match s 
        {
            "L" | "l" | "R" | "r" => Ok(Colour::L),
            "I" | "i" | "Y" | "y" => Ok(Colour::I),
            "T" | "t" | "G" | "g" => Ok(Colour::T),
            "S" | "s" | "B" | "b" => Ok(Colour::S),
            "_" | "-" | "." | "," => Ok(Colour::None),
            _                     => Err(error::error!("Invalid notation '{}' for colour.", s))
        }
    }
}

impl Colour 
{
    ///
    /// Returns the index of this colour into its enum.
    ///
    pub fn as_index (& self) -> usize 
    {
        match self 
        {
            Colour::L    => 0,
            Colour::I    => 1,
            Colour::T    => 2,
            Colour::S    => 3,
            Colour::None => panic!("Cannot take index of the null colour.")
        }
    }

    ///
    /// Returns a length-4 one-hot encoding for this colour in LITS order.
    ///
    pub fn one_hot (& self) -> Vec<bool>
    {
        vec![
            * self == Colour::L,
            * self == Colour::I,
            * self == Colour::T,
            * self == Colour::S
        ]
    }
}
