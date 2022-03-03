
use std::collections::{BTreeMap, BTreeSet};

use super::colour::Colour;
use super::outcome::Outcome;
use super::player::Player;
use super::point::Point;
use super::tetromino::Tetromino;
use super::transform::Transform;

use utils::error::Context;
use utils::notate::Notate;
use utils::*;

///
/// Represents a game board in the game The Battle of LITS. A game board is a 10x10 grid
/// of tiles.
///
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Board 
{
    score_tiles: Vec<Vec<Player>>,
    piece_tiles: Vec<Vec<Colour>>,
    pieces_remaining: Vec<usize>,
    attach_points: BTreeMap<Point, BTreeSet<Colour>>,
    to_move: Player
}

impl notate::Notate for Board 
{
    fn notate (& self) -> String 
    {
        let mut boardstr : String = String::new();
        
        for i in 0 .. 10
        {
            for j in 0 .. 10 
            {
                boardstr += & self.notate_tile(i, j);
            }
        }
        boardstr += & ",".to_string();

        for archetype in vec![Colour::L, Colour::I, Colour::T, Colour::S]
        {
            boardstr += & self.pieces_remaining[archetype.as_index()].to_string();
        }
        boardstr += & ",".to_string();

        boardstr += & self.to_move().notate();

        boardstr
    }

    fn parse(s: & str) -> Result<Board> 
    {
        let context = format!("Invalid notation '{}' for board.", s);

        // The hashstring has length 107: 100 characters representing the 100 tiles of the board in
        // (p, c) order; a comma; 4 characters representing the number of pieces remaining for 
        // each piece colour in LITS order; a comma; and a character representing the player to
        // move.

        let uncompressed = s.to_string();
        match uncompressed.len()
        {
            107 => {},
            _   => return Err(error::error!("Expected a length-205 uncompressed string.")).context(context.clone())
        };

        let mut score_tiles : Vec<Vec<Player>> = vec![vec![Player::None; 10]; 10];
        let mut piece_tiles : Vec<Vec<Colour>> = vec![vec![Colour::None; 10]; 10];

        for idx in 0 .. 100
        {
            let (i, j) = (idx / 10, idx % 10);
            let (score, piece) = Board::parse_tile(& uncompressed[idx ..= idx])?;

            score_tiles[i][j] = score;
            piece_tiles[i][j] = piece;
        }

        match & uncompressed[100 ..= 100]
        {
            "," => {},
            _   => return Err(error::error!("Expected a comma separating the board and piece counts.")).context(context.clone())
        };

        let mut piece_pool = Vec::new();
        for archetype in [Colour::L, Colour::I, Colour::T, Colour::S]
        {
            let idx = 101 + archetype.as_index();
            let remaining = (& uncompressed[idx ..= idx]).parse::<usize>().context(context.clone())?;
            match remaining 
            {
                0 ..= 5 => piece_pool.push(remaining),
                _       => return Err(error::error!("Invalid number of remaining pieces {} for type '{}'.", remaining, archetype.notate())).context(context.clone()) 
            };
        }

        match & uncompressed[105 ..= 105]
        {
            "," => {},
            _   => return Err(error::error!("Expected a comma separating the piece counts and moving player.")).context(context.clone())
        }

        let who_to_move = Player::parse(& uncompressed[106 ..= 106]).context(context.clone())?;
        match who_to_move
        {
            Player::X | Player::O => {},
            _ => return Err(error::error!("The player to move cannot be null.")).context(context.clone())
        };

        Board::new(& score_tiles, & piece_tiles, & piece_pool, who_to_move)
    }
}

impl std::fmt::Display for Board 
{
    fn fmt (& self, f: & mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        for j in 0 ..= 9
        {
            let j = 9 - j;
            for i in 0 ..= 9
            {
                match self.piece_tiles[i][j]
                {
                    Colour::None => write!(f, "{}", self.score_tiles[i][j]),
                    _            => write!(f, "{}", self.piece_tiles[i][j])
                }?;
            }
            write!(f, "\n")?;
        }

        write!(
            f, "{} {} {} {}  {} {} {} {} \n",
            Colour::L, self.pieces_remaining[Colour::L.as_index()],
            Colour::I, self.pieces_remaining[Colour::I.as_index()], 
            Colour::T, self.pieces_remaining[Colour::T.as_index()],
            Colour::S, self.pieces_remaining[Colour::S.as_index()]
        )?;

        Ok(())
    }
}

impl Board 
{
    ///
    /// Returns a blank board.
    ///
    pub fn blank () -> Board
    {
        let mut board = Board 
        { 
            score_tiles: vec![vec![Player::None; 10]; 10],
            piece_tiles: vec![vec![Colour::None; 10]; 10],
            pieces_remaining: vec![5; 4],
            attach_points: BTreeMap::new(),
            to_move: Player::X
        };

        for i in 0 .. 10 
        {
            for j in 0 .. 10 
            {
                board.attach_points.insert(Point::new(i, j), BTreeSet::from([Colour::L, Colour::I, Colour::T, Colour::S]));
            }
        }

        board
    }

    ///
    /// Recalculates the minimal set of attach points on this board; very inefficient
    /// as it simply brute forces.
    ///
    pub fn calculate_attach_points_from_scratch (& mut self)
    {
        self.attach_points.clear();

        let mut is_empty = true;

        for i in 0 .. 10 
        {
            for j in 0 .. 10 
            {
                if self.piece_tiles[i][j] != Colour::None 
                {
                    is_empty = false;
                }
            }
        }

        if ! is_empty
        {
            for i in 0 .. 10 
            {
                for j in 0 .. 10 
                {
                    let point = Point::new(i, j);

                    // If there is no colour at the point, and it has at least one coloured neighbour,
                    // then compute the colourset and add the attach point if and only if the colourset 
                    // is non-empty.

                    if self.piece_tiles[point.x() as usize][point.y() as usize] == Colour::None 
                        && point.neighbours_on_board().iter().any(|& p| self.piece_tiles[p.x() as usize][p.y() as usize] != Colour::None)
                    {
                        let mut colourset : BTreeSet<Colour> = BTreeSet::from([Colour::L, Colour::I, Colour::T, Colour::S]);
                        point.neighbours_on_board().iter().for_each(|& p| { colourset.remove(& self.piece_tiles[p.x() as usize][p.y() as usize]); });
                        if ! colourset.is_empty()
                        {
                            self.attach_points.insert(point, colourset);
                        }
                    }
                }
            }
        }
        else 
        {
            for i in 0 .. 10 
            {
                for j in 0 .. 10 
                {
                    let point = Point::new(i, j);
                    let colourset = BTreeSet::from([Colour::L, Colour::I, Colour::T, Colour::S]);
                    self.attach_points.insert(point, colourset);
                }
            }
        }
    }

    ///
    /// Returns the colour at the given tile.
    ///
    pub fn colour_at (& self, i: i32, j: i32) -> Colour 
    {
        self.piece_tiles[i as usize][j as usize]
    }

    ///
    /// Cycles the colour at this tile for setup purposes.
    ///
    pub fn cycle_colour (& mut self, i: i32, j: i32)
    {
        self.piece_tiles[i as usize][j as usize] = self.piece_tiles[i as usize][j as usize].next_and_none();
    }

    ///
    /// Cycles the colour at this tile for setup purposes.
    ///
    pub fn cycle_player (& mut self, i: i32, j: i32)
    {
        self.score_tiles[i as usize][j as usize] = self.score_tiles[i as usize][j as usize].next_and_none();
    }

    ///
    /// Returns all possible moves in this position.
    ///
    pub fn enumerate_moves (& self) -> BTreeSet<Tetromino>
    {
        let mut result : BTreeSet<Tetromino> = BTreeSet::new();

        let available_colours = [Colour::L, Colour::I, Colour::T, Colour::S].into_iter()
            .filter(|& c| self.pieces_remaining[c.as_index()] > 0)
            .collect::<BTreeSet<Colour>>();

        for (attach, colours) in & self.attach_points
        {
            for anchor in attach.get_potential_anchors()
            {
                for colour in colours.intersection(& available_colours)
                {
                    for tetromino in Tetromino::get_reference_tetromino(& colour, & anchor).enumerate_transforms()
                    {
                        if self.validate_tetromino(& tetromino).is_ok()
                        {
                            result.insert(tetromino);
                        }
                    }
                }
            }
        }

        result
    }

    ///
    /// Determines whether any more moves are possible in this position.
    ///
    pub fn has_moves (& self) -> bool 
    {
        ! self.enumerate_moves().is_empty()
    }

    ///
    /// Returns a new board with the given state.
    ///
    pub fn new (score_tiles: & Vec<Vec<Player>>, piece_tiles: & Vec<Vec<Colour>>, remaining: & Vec<usize>, to_move: Player) -> Result<Board>
    {
        let context = "Failed to create a new board.";
        let score_tiles = score_tiles.clone();
        let piece_tiles = piece_tiles.clone();
        let pieces_remaining = remaining.clone();
        let attach_points = BTreeMap::new();
        
        for archetype in [Colour::L, Colour::I, Colour::T, Colour::S]
        {
            let num = pieces_remaining[archetype.as_index()];
            match num 
            {
                0 ..= 5 => {},
                _       => return Err(error::error!("Invalid number of remaining pieces {} for colour '{}'.", num, archetype.notate()))
                            .context(context.clone())
            }
        }

        let mut b = Board { score_tiles, piece_tiles, pieces_remaining, attach_points, to_move };
        b.calculate_attach_points_from_scratch();
        Ok(b)
    }

    ///
    /// Returns the hexadecimal notation for the tile.
    ///
    pub fn notate_tile (& self, i: i32, j: i32) -> String 
    {
        let value = 5 * self.player_at(i, j).as_index_null() + self.colour_at(i, j).as_index_null();
        ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f"].get(value).unwrap().to_string()
    }

    ///
    /// Parses the tile.
    ///
    pub fn parse_tile (s: & str) -> Result<(Player, Colour)>
    {
        match s 
        {
            "0" => Ok((Player::None, Colour::None)),
            "1" => Ok((Player::None, Colour::L)),
            "2" => Ok((Player::None, Colour::I)),
            "3" => Ok((Player::None, Colour::T)),
            "4" => Ok((Player::None, Colour::S)),
            "5" => Ok((Player::X,    Colour::None)),
            "6" => Ok((Player::X,    Colour::L)),
            "7" => Ok((Player::X,    Colour::I)),
            "8" => Ok((Player::X,    Colour::T)),
            "9" => Ok((Player::X,    Colour::S)),
            "a" => Ok((Player::O,    Colour::None)),
            "b" => Ok((Player::O,    Colour::L)),
            "c" => Ok((Player::O,    Colour::I)),
            "d" => Ok((Player::O,    Colour::T)),
            "e" => Ok((Player::O,    Colour::S)),
            _   => Err(error::error!(format!("Invalid notation '{}' for tile.", s)))
        }
    }

    ///
    /// Places the tetromino, provided it is a legal move, and updates the attach points 
    /// on this board.
    ///
    pub fn place_tetromino (& mut self, tetromino: & Tetromino) -> Result<()>
    {
        // Check if the tetromino is valid in the position.

        let context = notate!("Failed to play tetromino '{}' in position '{}'.", tetromino, self);
        self.validate_tetromino(tetromino).context(context.clone())?;

        // Play the tetromino.

        self.pieces_remaining[tetromino.colour().as_index()] -= 1;
        let points = tetromino.points_real();
        points.iter().for_each(|& p| { self.piece_tiles[p.x() as usize][p.y() as usize] = tetromino.colour(); } );
        self.to_move = self.to_move.next();

        // Update the attach points, using the real points as hints.

        self.update_attach_points_add(tetromino);
        Ok(())
    }

    ///
    /// Returns the player at the given tile.
    ///
    pub fn player_at (& self, i: i32, j: i32) -> Player 
    {
        self.score_tiles[i as usize][j as usize]
    }

    ///
    /// Determines whether the given real point attaches.
    ///
    pub fn point_attach_exists (& self, point: & Point) -> bool 
    {
        self.attach_points.contains_key(& point) 
    }

    ///
    /// Determines whether the given real point attaches to a tile of the same colour.
    ///
    pub fn point_attach_same_colour (& self, point: & Point, colour: & Colour) -> bool 
    {
        for neighbour in point.neighbours_on_board()
        {
            if self.piece_tiles[neighbour.x() as usize][neighbour.y() as usize] == * colour
            {
                return true;
            }
        }
        false
    }

    ///
    /// Prints this board's attach points.
    ///
    pub fn print_attach_points (& self)
    {
        for j in 0 ..= 9
        {
            let j = 9 - j;
            let mut linestr = "".to_owned();
            for i in 0 ..= 9 
            {
                linestr += & match self.attach_points.contains_key(& Point::new(i, j))
                {
                    true  => format!("{}", Player::X),
                    false => format!("{}", Player::None) 
                };
            }
            println!("{}", linestr);
        }
        println!("");
    }

    ///
    /// Gets the number of tetrominos of the given colour remaining to be played.
    ///
    pub fn remaining_of (& self, colour: & Colour) -> usize 
    {
        self.pieces_remaining[colour.as_index()]
    }

    ///
    /// Gets the result of this game.
    ///
    pub fn result (& self) -> Outcome 
    {
        match self.has_moves()
        {
            true  => Outcome::InProgress,
            false => 
            {
                let score = self.score();
                if score > 0.0
                {
                    return Outcome::X(score);
                }
                else if score < 0.0
                {
                    return Outcome::O(score);
                }
                else 
                {
                    // If it's a draw, the result goes to whoever 
                    // played the last tetromino.

                    return match self.to_move().next() == Player::X 
                    {
                        true  => Outcome::X(0.0),
                        false => Outcome::O(0.0)
                    };
                }
            }
        }
    }

    ///
    /// Returns the integer score of this board in terms of X's perspective.
    ///
    pub fn score (& self) -> f64 
    {
        let mut sum = 0.0;
        for i in 0 .. 10 
        {
            for j in 0 .. 10 
            {
                if self.piece_tiles[i][j] == Colour::None 
                {
                    sum += self.score_tiles[i][j].value();
                }
            }
        }
        sum
    }

    ///
    /// Sets a scoring tile at the given position.
    ///
    pub fn set_scoring_tile (& mut self, i: usize, j: usize, player: & Player)
    {
        * self.score_tiles.get_mut(i).unwrap().get_mut(j).unwrap() = * player;
    }

    ///
    /// Determines whether the given tetromino forms an o.
    ///
    pub fn tetromino_attach_forms_o (& self, points: & Vec<Point>) -> bool 
    {
        // Normalize the points, and take the anchor position as if the points are 
        // contained in a bounding box with padding size 1.
        
        let mut points = points.clone();
        let anchor = Transform::normalize(& mut points) - Point::new(1, 1);
        points.iter_mut().for_each(|p| { * p = * p + Point::new(1, 1); } );

        // Form the 6x6 grid; this is the only local window in which a violation 
        // could occur.

        let mut grid = vec![vec![false; 6]; 6];
        for i in 0 .. 6 
        {
            for j in 0 .. 6 
            {
                let here = Point::new(i, j) + anchor;
                if here.in_bounds()
                {
                    if self.piece_tiles[here.x() as usize][here.y() as usize] != Colour::None
                    {
                        grid[i as usize][j as usize] = true;
                    }
                }
            }
        }
        points.iter().for_each(|& p| { grid[p.x() as usize][p.y() as usize] = true; } );

        // Check all 2x2 windows in the grid for truthiness.

        for i in 0 .. 5
        {
            for j in 0 .. 5 
            {
                if grid[i][j] && grid[i + 1][j] && grid[i][j + 1] && grid[i + 1][j + 1]
                {
                    return true;
                }
            }
        }
        false
    }

    ///
    /// Determines whether the given tetromino exists on this board.
    ///
    pub fn tetromino_exists (& self, tetromino: & Tetromino) -> bool 
    {
        tetromino.points_real().iter().all(|& p| self.piece_tiles[p.x() as usize][p.y() as usize] == tetromino.colour())
    }

    ///
    /// Returns the player to move.
    ///
    pub fn to_move (& self) -> Player 
    {
        self.to_move
    }

    ///
    /// Removes the given tetromino from the board, provided it was even there.
    ///
    pub fn undo_tetromino (& mut self, tetromino: & Tetromino) -> Result<()>
    {
        // Check if the piece can be removed.

        let context = notate!("Failed to undo tetromino '{}' in position '{}'.", tetromino, self);

        let _ = self.pieces_remaining[tetromino.colour().as_index()] < 5
            || return Err(error::error!(notate!("There are no '{}'s on the board.", tetromino.colour()))).context(context.clone());

        let _ = self.tetromino_exists(tetromino)
            || return Err(error::error!(notate!("Tetromino '{}' was not matched on the board.", tetromino))).context(context.clone());

        // Remove the piece.

        self.pieces_remaining[tetromino.colour().as_index()] += 1;
        let points = tetromino.points_real();
        points.iter().for_each(|& p| { self.piece_tiles[p.x() as usize][p.y() as usize] = Colour::None; } );
        self.to_move = self.to_move.next();

        // Update the attach points.

        self.update_attach_points_sub(tetromino);
        Ok(())
    }

    ///
    /// Updates the attach points on this board given the hinting points that were 
    /// added in a placement.
    ///
    pub fn update_attach_points_add (& mut self, tetromino: & Tetromino) 
    {
        // Remove all attach points that overlap with the played piece.

        if self.pieces_remaining.iter().sum::<usize>() == 19 
        {
            // Then we need to recalculate, because the first move is either a blank 
            // board (which has full attach points) or has a special position.

            self.calculate_attach_points_from_scratch();
        }
        else 
        {
            let points = tetromino.points_real();
            points.iter().for_each(|p| { self.attach_points.remove(p); });

            // Get the new attach points and do the following: if the attach 
            // point exists, subtract this tetromino's colour from its colourset 
            // and remove the attach point if it results in an empty colourset;
            // otherwise, add an attach point that lacks the colour of the piece 
            // played. The new colourset in this case is guaranteed to be non-empty, 
            // because the tetromino played here could not neighbour its own colour.

            let new_attaches = tetromino.get_attaches().into_iter()
                .filter(|& p| self.piece_tiles[p.x() as usize][p.y() as usize] == Colour::None)
                .collect::<BTreeSet<Point>>();

            for new_attach in & new_attaches
            {
                if self.attach_points.contains_key(new_attach)
                {
                    self.attach_points.get_mut(new_attach).unwrap().remove(& tetromino.colour());
                    if self.attach_points[new_attach].len() == 0 
                    {
                        self.attach_points.remove(new_attach);
                    }
                }
                else 
                {
                    let mut colourset = BTreeSet::from([Colour::L, Colour::I, Colour::T, Colour::S]);
                    colourset.remove(& tetromino.colour());
                    self.attach_points.insert(* new_attach, colourset);
                }
            }
        }
    }

    ///
    /// Updates the attach points on this board, given the hinting points that were 
    /// removed in an undo.
    ///
    pub fn update_attach_points_sub (& mut self, tetromino: & Tetromino)
    {
        if self.pieces_remaining.iter().sum::<usize>() == 20 
        {
            self.calculate_attach_points_from_scratch();
        }
        else 
        {
            // Any attach point that was potentially generated by this tetromino is visited 
            // and only kept if it has another neighbour, in which case the colourset is 
            // recomputed. The resulting colourset cannot be null, because there was a 
            // tile of a non-null colour here previously, which could not have neighboured
            // itself.

            let created_attaches = tetromino.get_attaches().into_iter()
                .filter(|& p| self.piece_tiles[p.x() as usize][p.y() as usize] == Colour::None)
                .collect::<BTreeSet<Point>>();
            
            for old_attach in & created_attaches
            {
                if old_attach.neighbours_on_board().iter().any(|& p| self.piece_tiles[p.x() as usize][p.y() as usize] != Colour::None)
                {
                    let mut colourset : BTreeSet<Colour> = BTreeSet::from([Colour::L, Colour::I, Colour::T, Colour::S]);
                    old_attach.neighbours_on_board().iter().for_each(|& p| { colourset.remove(& self.piece_tiles[p.x() as usize][p.y() as usize]); });
                    
                    self.attach_points.remove(old_attach);
                    self.attach_points.insert(* old_attach, colourset);
                }
                else 
                {
                    self.attach_points.remove(old_attach);
                }
            }

            // Then, we add back each point of the tetromino as an attach point if it has any
            // neighbours; if so, it is an attach point that existed before the piece was played 
            // (and cannot have an empty colourset, because a tile of a non-null colour occupied
            // this space).

            for point in & tetromino.points_real()
            {
                if point.neighbours_on_board().iter().any(|& p| self.piece_tiles[p.x() as usize][p.y() as usize] != Colour::None)
                {
                    let mut colourset : BTreeSet<Colour> = BTreeSet::from([Colour::L, Colour::I, Colour::T, Colour::S]);
                    point.neighbours_on_board().iter().for_each(|& p| { colourset.remove(& self.piece_tiles[p.x() as usize][p.y() as usize]); });
                    
                    self.attach_points.remove(point);
                    self.attach_points.insert(* point, colourset);
                }
            }
        }
    }

    ///
    /// Determines whether playing the given tetromino is valid in this state.
    ///
    pub fn validate_tetromino (& self, tetromino: & Tetromino) -> Result<()>
    {
        let context = notate!("Tetromino '{}' is not valid in position '{}'.", tetromino, self);

        let points = tetromino.points_real();
        let colour = tetromino.colour();

        let _ = self.pieces_remaining[colour.as_index()] > 0 
            || return Err(error::error!(notate!("There are no more copies of the '{}' tetromino.", colour))).context(context.clone());

        let _ = points.iter().all(|& p| p.in_bounds()) 
            || return Err(error::error!(notate!("Tetromino '{}' is not in bounds.", tetromino))).context(context.clone());
       
        let _ = ! points.iter().any(|& p| self.piece_tiles[p.x() as usize][p.y() as usize] != Colour::None)
            || return Err(error::error!(notate!("Tetromino '{}' overlaps an existing piece.", tetromino))).context(context.clone());

        let _ = points.iter().any(|& p| self.point_attach_exists(& p))
            || return Err(error::error!(notate!("Tetromino '{}' has no attach point.", tetromino))).context(context.clone());

        let _ = ! points.iter().any(|& p| self.point_attach_same_colour(& p, & colour))
            || return Err(error::error!(notate!("Tetromino '{}' attaches to a tetromino of the same colour.", tetromino))).context(context.clone()); 
        
        let _ = ! self.tetromino_attach_forms_o(& points)
            || return Err(error::error!(notate!("Tetromino '{}' forms a 2-by-2 square.", tetromino))).context(context.clone());

        Ok(())
    }
}
