
use super::error::*;

#[macro_export]
///
/// A format! enhancement for building notational strings where all subcomponents 
/// are implementors of the Notate trait.
///
macro_rules! notate 
{
    ($fmt:expr, $($args:expr),*) => 
    {
        format!($fmt, $($args.notate()),*)
    };
}

///
/// A trait representing the concept of canonical notation.
///
/// An implementor provides a canonical notation by way of notate(),
/// and recognizes potentially non-canonical notation by way of parse().
///
pub trait Notate 
    where Self: Sized
{
    ///
    /// Returns the canonical notational string for this object.
    ///
    fn notate (& self) -> String;

    ///
    /// Constructs a new object from the given notational string, provided
    /// that the notation is valid.
    ///
    fn parse (s: & str) -> Result<Self>;
}

