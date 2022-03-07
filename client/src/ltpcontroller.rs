
use gtp::Command;
use gtp::controller::Engine; 

use lazy_static::lazy_static;

use std::sync::Mutex;
use std::time::Duration;

use super::ltpcommand::LtpCommand;
use utils::notate::Notate;
use lits::*;
use utils::*;

///
/// A wrapper around a GtpEngine controller that provides calls for 
/// LITS text protocol communication.
///
/// A call to an engine command returns a unique command ID that corresponds
/// to the request made to the engine. The caller recieves the ID and 
/// the engine command returns without blocking. When the ID response is 
/// found in the process stdout, the response is added to the response map,
/// and made available when the caller queries the map and consumes the 
/// response with their held ID. Stdout polling is done non-blocking by the 
/// engine on a background thread.
///
pub struct LtpController
{
    handle: Engine 
}

lazy_static!
{
    static ref EXE_PATH : Mutex<String> = Mutex::new(String::new());
}

impl LtpController 
{
    ///
    /// Requests the engine to perform an analysis on the current game, returning the 
    /// analytical score (rather than the actual score derived from the scoring tiles) 
    /// after each move of the game. from X's perspective.
    ///
    pub fn cmd_analyze (& mut self)
    {
        self.dispatch(LtpCommand::AnalyzePosition, & vec![]);
    }

    ///
    /// Applies the given board as a setup position. This is a state-breaking operation,
    /// and will halt any incoming search requests.
    ///
    pub fn cmd_apply_setup (& mut self, board: & Board)
    {
        self.dispatch(LtpCommand::ApplySetupPosition, & vec![board.notate()]);
    }

    ///
    /// Tells the engine to abort a genmove search early, and to return the best move found 
    /// so far in the execution of the search tree.
    ///
    pub fn cmd_cancel (& mut self)
    {
        self.dispatch(LtpCommand::CancelSearch, & vec![]);
    }

    ///
    /// Requests the engine to find the best move for the given player. How the 
    /// engine manages resources is a matter of engine configuration and no behaviour 
    /// is mandated by the controller.
    ///
    pub fn cmd_gen_move (& mut self, who: & Player)
    {
        self.dispatch(LtpCommand::GenMove, & vec![who.notate()]);
    }

    ///
    /// Starts a blank game on the engine, erasing any history. Whether or not 
    /// the engine keeps its search trees intact is a matter of engine configuration
    /// and no behaviour is mandated by the controller.
    ///
    pub fn cmd_new_game (& mut self)
    {
        self.dispatch(LtpCommand::NewGame, & vec![]);
    }

    ///
    /// Applies the given tetromino to the position. Note that despite modifying the state,
    /// provided that the move is legal it is not a state-breaking operation, and the 
    /// engine is required to pivot its search tree to accomodate the state change.
    ///
    pub fn cmd_play (& mut self, tetromino: & Tetromino) 
    {
        self.dispatch(LtpCommand::PlaceTetromino, & vec![tetromino.notate()]);
    }

    ///
    /// Undoes the last move in the position, provided one exists.
    ///
    pub fn cmd_undo (& mut self)
    {
        self.dispatch(LtpCommand::Undo, & vec![]);
    }

    ///
    /// Dispatches the given LITS text protocol command, and returns a UUID if 
    /// and only if the command expects a response.
    ///
    pub fn dispatch (& mut self, command: LtpCommand, args: & Vec<String>)
    {
        // Forms the command line from the given command and args.

        let commandline = match args.len()
        {
            0 => format!(
                "{}\n",
                command.command()
            ),
            _ => format!(
                "{} {}\n",
                command.command(), args.join(" ")
            )
        };

        // Delivers the command to the engine via stdin.

        let cmd = Command::new(& commandline);
        self.handle.send(cmd.clone());
        log::info!("Sent command: {}", cmd.to_string());
    }

    ///
    /// Shuts down the process backing this engine.
    ///
    pub fn halt (& mut self)
    {
        self.dispatch(LtpCommand::Shutdown, & vec![]);
    }

    ///
    /// Initializes the controller executable path.
    ///
    pub fn initialize (exe_path: & str) 
    {
        * EXE_PATH.lock().unwrap() = exe_path.to_string();
    }

    ///
    /// Initializes this engine.
    ///
    pub fn new () -> LtpController
    {
        let path = EXE_PATH.lock().unwrap();
        let engine = Engine::new(& path, & []);
        let mut controller = LtpController { handle: engine };
        controller.handle.start().expect(& format!("Could not start engine (with path {}).", path));

        controller
    }

    ///
    /// Polls responses from the engine, erroring if the response has not 
    /// yet been received.
    ///
    pub fn poll_response (& mut self) -> Result<String>
    {
        if let Ok(resp) = self.handle.wait_response(Duration::from_millis(100))
        {
            log::info!("Received response '{}'.", resp.text());
            return Ok(resp.text());
        }
        Err(error::error!("Could not find a response; try again later."))
    }
}

