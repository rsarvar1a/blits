
use super::board::Board;
use super::player::Player;
use super::tetromino::Tetromino;

use utils::notate::Notate;
use utils::*;

///
/// A convenience structure that wraps a board of The Battle of Lits into
/// a game, and provides:
/// - linear history manipulation (push and pop); and 
/// - notating to, and parsing from, file-like objects or strings.
///
/// The linear history works as follows:
/// - when a move is undone, it goes to the redo stack;
/// - when a move is played, it goes to the hist stack; and:
///     - if the redo stack is non-empty and does not match the move, it is cleared;
///     - otherwise the top of redo stack is popped.
///
/// In this way, the linear history essentially works as a single 
/// variation tree; you can read up and down the history until a new 
/// move is made at which point the future of that variation is lost.
///
/// The view is special in that it has a base board and a current board.
/// The base board is an unrestricted board which should, in normal 
/// circumstances, contain only the scoring tiles for each player as well
/// as have all 5 copies of each tile available to play. However, this 
/// board can also contain setup pieces, which are pieces played into 
/// the starting position of the game outside of the scope of the game
/// history. 
///
/// Note that using the unrestricted setup feature could result in 
/// misleading UI, because the user will reach a setup position that 
/// appears to have pieces left to remove but the user will nevertheless 
/// be unable to rewind the position. It is also probably highly buggy 
/// because we are trying to optimize attach point calculation.
///
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Game 
{
    // State.

    curr_board: Board,
    base_board: Board,
    hist_stack: Vec<Tetromino>,
    redo_stack: Vec<Tetromino>,

    // A helper that stops us from having to count the history stack.

    to_move: Player
}

impl notate::Notate for Game 
{
    fn notate (& self) -> String 
    {
        let mut result = self.base_board.notate();
        
        let mut reverse_redo_stack = self.redo_stack.clone();
        reverse_redo_stack.reverse();
        let move_stack = [self.hist_stack.clone(), reverse_redo_stack].concat();

        for tetromino in & move_stack 
        {
            result += & notate!("\n{}", tetromino);
        }

        result
    }

    fn parse (s: & str) -> Result<Game>
    {
        let context = format!("Invalid notation '{}' for game.", s);

        let line_vec = s.to_owned().split('\n').map(|s| s.to_owned()).collect::<Vec<String>>();
        
        if line_vec.len() == 0 
        {
            return Err(error::error!("Game notation must be non-empty.")).context(context.clone());
        }

        let base_board = Board::parse(& line_vec[0]).context(context.clone())?;
        let mut curr_board = base_board.clone();
        let mut hist_stack : Vec<Tetromino> = Vec::new();
        let redo_stack = Vec::new();
        let mut to_move = Player::X;

        for i in 1 .. line_vec.len()
        {
            let move_context = format!("Invalid notation in move {}.", i);

            let tetromino = Tetromino::parse(& line_vec[i]).context(move_context.clone()).context(context.clone())?;
            curr_board.place_tetromino(& tetromino).context(move_context.clone()).context(context.clone())?;

            hist_stack.push(tetromino);
            to_move = to_move.next();
        }

        Ok(Game { base_board, curr_board, hist_stack, redo_stack, to_move })
    }
}

impl Game 
{
    ///
    /// Applies the tetromino to the board if the tetromino is valid in this position.
    ///
    pub fn apply (& mut self, tetromino: & Tetromino) -> Result<()>
    {
        match self.curr_board.place_tetromino(tetromino)
        {
            Ok(_) => 
            {
                self.hist_stack.push(tetromino.clone());
                if ! self.redo_stack.is_empty()
                {
                    if self.redo_stack.last().unwrap() == tetromino 
                    {
                        self.redo_stack.pop();
                    }
                    else 
                    {
                        self.redo_stack.clear();
                    }
                }
                Ok(())
            },
            Err(err) => 
            {
                Err(err).context(notate!("Failed to apply tetromino '{}' to this game.", tetromino))
            }
        }
    }

    ///
    /// Cycles the colour at a tile for setup purposes.
    ///
    pub fn cycle_colour (& mut self, i: i32, j: i32)
    {
        self.get_board().cycle_colour(i, j);
    }

    ///
    /// Cycles the player at a tile for setup purposes.
    ///
    pub fn cycle_player (& mut self, i: i32, j: i32)
    {
        self.get_board().cycle_player(i, j);
    }

    ///
    /// Returns the current state of the board.
    ///
    pub fn get_board (& mut self) -> & mut Board 
    {
        & mut self.curr_board
    }

    ///
    /// Returns the original state of the board.
    ///
    pub fn get_board_base (& mut self) -> & mut Board 
    {
        & mut self.base_board
    }

    ///
    /// Returns the future of the board; the next tetromino is at the top.
    ///
    pub fn get_future (& self) -> & Vec<Tetromino>
    {
        & self.redo_stack
    }

    ///
    /// Returns the history of the board; the most recent tetromino is at the top.
    ///
    pub fn get_history (& self) -> & Vec<Tetromino>
    {
        & self.hist_stack
    }

    ///
    /// Returns a blank starting game.
    ///
    pub fn new () -> Game 
    {
        Game 
        { 
            base_board: Board::blank(), 
            curr_board: Board::blank(), 
            hist_stack: vec![], 
            redo_stack: vec![], 
            to_move: Player::X 
        }
    }

    ///
    /// Sets a tile on the game board to the given scoring tile.
    ///
    pub fn set_scoring_tile (& mut self, i: usize, j: usize, player: & Player)
    {
        self.base_board.set_scoring_tile(i, j, player);
        self.curr_board.set_scoring_tile(i, j, player);
    }

    ///
    /// Determines the next player to move in this game.
    ///
    pub fn to_move (& self) -> Player 
    {
        self.to_move
    }

    ///
    /// Undoes the last move played.
    ///
    pub fn undo (& mut self) -> Result<()>
    {
        let context = "Failed to undo the last tetromino played in this game.";

        match ! self.hist_stack.is_empty()
        {
            true => 
            {
                let tetromino = self.hist_stack.last().unwrap().clone();
                self.curr_board.undo_tetromino(& tetromino).context(context.clone())?;

                self.hist_stack.pop();
                self.redo_stack.push(tetromino);

                Ok(())
            },
            false => 
            {
                Err(error::error!("There is no tetromino in the history.")).context(context.clone())
            }
        }
    }
}
