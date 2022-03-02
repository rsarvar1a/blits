
///
/// The available commands in the LITS text protocol.
///
pub enum LtpCommand 
{
    // Special lifecycle commands, not to be called as normal commands.

    Initialize,                 // Initializes the backing engine.
    Shutdown,                   // Halts the backing engine.

    // State commands.

    ApplySetupPosition,         // Applies a board position with the given hashstring.
    NewGame,                    // Starts a new game with a blank scoring set.
    PlaceTetromino,             // Places a tetromino, provided it is legal.
    Undo,                       // Undoes the last move, provided one exists.

    // Analytical commands.

    AnalyzePosition,            // Returns a vector of float values representing X's favour over the course of the game.
    CancelSearch,               // Aborts a running move search early.
    GenMove,                    // Gets the best move for the current player.
}

impl LtpCommand 
{
    ///
    /// Maps this command to a literal command string.
    ///
    pub fn command (& self) -> String 
    {
        match self 
        {
            LtpCommand::Initialize         => "initialize".to_owned(),
            LtpCommand::Shutdown           => "shutdown".to_owned(),

            LtpCommand::ApplySetupPosition => "setup-position".to_owned(),
            LtpCommand::NewGame            => "new-game".to_owned(),
            LtpCommand::PlaceTetromino     => "play-move".to_owned(),
            LtpCommand::Undo               => "undo-move".to_owned(),

            LtpCommand::AnalyzePosition    => "analyze-board".to_owned(),
            LtpCommand::CancelSearch       => "cancel-search".to_owned(),
            LtpCommand::GenMove            => "gen-move".to_owned()
        }
    }

    ///
    /// Determines whether callers of this command should expect a response.
    ///
    pub fn returns (& self) -> bool 
    {
        match self 
        {
            LtpCommand::AnalyzePosition | LtpCommand::GenMove => true,
            _                                                 => false
        }
    }
}

