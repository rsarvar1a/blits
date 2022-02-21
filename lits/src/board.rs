
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
    attach_points: BTreeMap<Point, BTreeSet<Colour>>
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
                boardstr += & self.score_tiles[i][j].notate();
                boardstr += & self.piece_tiles[i][j].notate();
            }
        }
        boardstr += & ",".to_string();

        for archetype in vec![Colour::L, Colour::I, Colour::T, Colour::S]
        {
            boardstr += & self.pieces_remaining[archetype.as_index()].to_string();
        }

        b65k::encode(& boardstr)
    }

    fn parse(s: & str) -> Result<Board> 
    {
        let context = format!("Invalid notation '{}' for board.", s);

        // The hashstring has length 205: 200 characters representing the 200 tiles of the board in
        // (p, c) order; a comma; and 4 characters representing the number of pieces remaining for 
        // each piece colour in LITS order.

        let uncompressed = b65k::decode(s); 
        match uncompressed.len()
        {
            205 => {},
            _   => return Err(error::error!("Expected a length-205 uncompressed string.")).context(context.clone())
        };

        let mut score_tiles : Vec<Vec<Player>> = vec![vec![Player::None; 10]; 10];
        let mut piece_tiles : Vec<Vec<Colour>> = vec![vec![Colour::None; 10]; 10];

        for idx in (0 .. 200).step_by(2)
        {
            let (i, j) = ((idx / 2) / 10, (idx / 2) % 10);
            let score = Player::parse(& uncompressed[idx ..= idx]).context(context.clone())?;
            let piece = Colour::parse(& uncompressed[idx + 1 ..= idx + 1]).context(context.clone())?;

            score_tiles[i][j] = score;
            piece_tiles[i][j] = piece;
        }

        match & uncompressed[200 ..= 200]
        {
            "," => {},
            _   => return Err(error::error!("Expected a comma separating the board and piece counts.")).context(context.clone())
        };

        let mut piece_pool = Vec::new();
        for archetype in [Colour::L, Colour::I, Colour::T, Colour::S]
        {
            let idx = 201 + archetype.as_index();
            let remaining = (& uncompressed[idx ..= idx]).parse::<usize>().context(context.clone())?;
            match remaining 
            {
                0 ..= 5 => piece_pool.push(remaining),
                _       => return Err(error::error!("Invalid number of remaining pieces {} for type '{}'.", remaining, archetype.notate())).context(context.clone()) 
            };
        }

        Board::new(& score_tiles, & piece_tiles, & piece_pool)
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
            attach_points: BTreeMap::new()
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
    pub fn new (score_tiles: & Vec<Vec<Player>>, piece_tiles: & Vec<Vec<Colour>>, remaining: & Vec<usize>) -> Result<Board>
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

        let mut b = Board { score_tiles, piece_tiles, pieces_remaining, attach_points };
        b.calculate_attach_points_from_scratch();
        Ok(b)
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

        // Update the attach points, using the real points as hints.

        self.update_attach_points_add(tetromino);
        Ok(())
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
                    return Outcome::Draw;
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
