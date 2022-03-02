
use std::collections::HashSet;

///
/// Represents the application state. The true state is a set of distinct AppStates.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AppState 
{
    ///
    /// A mode that allows the player to set the X and O tiles on the board.
    ///
    BoardSetupMode,

    ///
    /// A mode that signifies the player is interacting with a binded piece.
    ///
    PieceMode,

    ///
    /// A mode that signifies the player is waiting for an engine response.
    ///
    Waiting,
}

///
/// Represents the status of the gamestate as an additive set of individual logical states.
///
pub type StateSet = HashSet<AppState>;

