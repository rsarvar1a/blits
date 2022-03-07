
use lazy_static::lazy_static;
use regex::Regex;

use std::collections::{BTreeSet, HashMap};
use std::sync::RwLock;

use super::board::Board;
use super::colour::Colour;
use super::point::Point;
use super::transform::Transform;

use utils::error::Context;
use utils::notate::Notate;
use utils::*;

///
/// A representation of a tetromino in the game The Battle of LITS.
///
/// A tetromino consists of four things:
/// - a colour, which restricts its overall shape;
/// - an anchor, which is the absolute position of the top-left point in the tetromino's bounding box;
/// - a list of points, which are positive offsets to the anchor and outline the shape;
/// - a transform, which is the canonical transform in use, in terms of the reference shape.
///
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Tetromino 
{
    colour: Colour,
    anchor: Point,
    points: Vec<Point>,
    transform: Transform
}

impl std::fmt::Display for Tetromino 
{
    fn fmt (& self, f: & mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        let p1 = self.points[0] + self.anchor;
        let p2 = self.points[1] + self.anchor;
        let p3 = self.points[2] + self.anchor;
        let p4 = self.points[3] + self.anchor;
        write!(f, "{}[{},{},{},{}]", self.colour, p1, p2, p3, p4)
    }
}

impl notate::Notate for Tetromino 
{
    fn notate (& self) -> String 
    {
        let p1 = self.points[0] + self.anchor;
        let p2 = self.points[1] + self.anchor;
        let p3 = self.points[2] + self.anchor;
        let p4 = self.points[3] + self.anchor;
        notate!("{}[{},{},{},{}]", self.colour, p1, p2, p3, p4)
    }

    fn parse (s: & str) -> Result<Tetromino>
    {
        lazy_static!
        {
            static ref TET_RE : Regex = Regex::new(r"^([LITS])\[(\d{2}),(\d{2}),(\d{2}),(\d{2})\]$").unwrap();
        }

        let context = format!("Invalid notation '{}' for tetromino.", s);

        if let Some(capture) = TET_RE.captures(s)
        {
            let colour = Colour::parse(capture.get(1).unwrap().as_str()).context(context.clone())?;

            if colour != Colour::None 
            {
                let mut points = Vec::new();
                
                for i in 2 ..= 5
                {
                    let p = Point::parse(capture.get(i).unwrap().as_str()).context(context.clone())?;
                    points.push(p);
                }

                return Tetromino::from_points_with_colour(& colour, & points);
            }
            else 
            {
                return Err(error::error!("Tetromino cannot use the null colour.")).context(context.clone());
            }
        }
        else 
        {
            return Err(error::error!("No capture found.")).context(context.clone());
        }
    }
}

pub const TETROMINO_RANGE : usize = 1293;

lazy_static! 
{
    static ref MOVEMAP_FWD : RwLock<HashMap<Tetromino, usize>> = RwLock::new(HashMap::new());
    static ref MOVEMAP_REV : RwLock<HashMap<usize, Tetromino>> = RwLock::new(HashMap::new()); 
}

impl std::convert::From<usize> for Tetromino 
{
    fn from (n: usize) -> Tetromino 
    {
        MOVEMAP_REV.read().unwrap().get(& n).unwrap().clone()    
    }
}

impl std::convert::Into<usize> for Tetromino 
{
    fn into (self) -> usize 
    {
        MOVEMAP_FWD.read().unwrap().get(& self).unwrap().clone()    
    }
}

impl Tetromino 
{
    ///
    /// Returns the anchor of this tetromino.
    ///
    pub fn anchor (& self) -> Point 
    {
        self.anchor
    }

    ///
    /// Returns the colour of this tetromino.
    ///
    pub fn colour (& self) -> Colour 
    {
        self.colour
    }

    ///
    /// Returns a new raw tetromino, not guaranteed to be valid.
    ///
    pub fn construct_raw (colour: & Colour, anchor: & Point, points: & Vec<Point>, transform: & Transform) -> Tetromino 
    {
        Tetromino { colour: * colour, anchor: * anchor, points: points.clone(), transform: * transform }
    }

    ///
    /// Returns the in-order vector of all possible transforms of this tetromino.
    /// Each returned transform retains the same anchor point as described by this instance.
    ///
    pub fn enumerate_transforms (& self) -> Vec<Tetromino>
    {
        Transform::enumerate(& self.colour).iter().map( |& t| t.apply_to_tetromino(self) ).collect::<Vec<Tetromino>>()
    }

    ///
    /// Returns a new Tetromino with the given shape, deducing the colour and transform.
    /// The given points must correspond to a valid colour's shape.
    ///
    pub fn from_points (points: & Vec<Point>) -> Result<Tetromino>
    {
        let context = "Could not create tetromino from absolute points.";

        let mut points = points.clone();
        let anchor = Transform::normalize(& mut points);

        for colour in [Colour::L, Colour::I, Colour::T, Colour::S]
        {
            if let Ok(tetromino) = Tetromino::from_points_with_anchor(& colour, & anchor, & points)
            {
                return Ok(tetromino);
            }
        }
        return Err(error::error!("Points '{:#?}' do not form a valid shape.", points)).context(context.clone()); 
    }

    ///
    /// Returns a new Tetromino with the given shape, where the points are relative positive 
    /// offsets in terms of the provided anchor, and the transform is deduced.
    ///
    pub fn from_points_with_anchor (colour: & Colour, anchor: & Point, points: & Vec<Point>) -> Result<Tetromino>
    {
        let template = Tetromino::get_reference_tetromino(colour, anchor);

        for transformed_tetromino in template.enumerate_transforms()
        {
            let transform = transformed_tetromino.transform();
            let transformed_points = transformed_tetromino.points();

            if transformed_points.iter().all( |t_p| points.contains(t_p) )
            {
                return Ok(Tetromino { colour: * colour, anchor: * anchor, points: transformed_points.clone(), transform });
            }
        }

        let context = "Could not create tetromino from reference points.";
        Err(error::error!("Points '{:#?}' do not form a valid transform of piece '{}'.", points, colour.notate())).context(context.clone())
    }

    ///
    /// Returns a new Tetromino with the given colour and shape, deducing the transform.
    /// The given points must correspond to the shape given by the colour. The anchor is 
    /// taken to be the top-left corner of the bounding box containing the points.
    ///
    pub fn from_points_with_colour (colour: & Colour, points: & Vec<Point>) -> Result<Tetromino>
    {
        let mut points = points.clone();
        let anchor = Transform::normalize(& mut points);
        Tetromino::from_points_with_anchor(colour, & anchor, & points)
    }

    ///
    /// Gets all attach points generated when this tetromino is played to the board.
    ///
    pub fn get_attaches (& self) -> BTreeSet<Point>
    {
        let points = self.points_real();
        let mut result : BTreeSet<Point> = BTreeSet::new();

        for point in & points 
        {
            for neighbour in point.neighbours_on_board()
            {
                if ! points.contains(& neighbour)
                {
                    result.insert(neighbour);
                }
            }
        }

        result
    }

    ///
    /// Returns the identity tetromino at the given anchor position.
    ///
    pub fn get_reference_tetromino (colour: & Colour, anchor: & Point) -> Tetromino
    {
        let point_set = match colour 
        {
            Colour::L => vec!
            [
                Point::new(0, 0),
                Point::new(0, 1),
                Point::new(0, 2),
                Point::new(1, 2)
            ],
            Colour::I => vec! 
            [
                Point::new(0, 0),
                Point::new(0, 1),
                Point::new(0, 2),
                Point::new(0, 3)
            ],
            Colour::T => vec! 
            [
                Point::new(0, 0),
                Point::new(1, 1),
                Point::new(1, 0),
                Point::new(2, 0)
            ],
            Colour::S => vec! 
            [
                Point::new(0, 1),
                Point::new(1, 1),
                Point::new(1, 0),
                Point::new(2, 0)
            ],
            _         => panic!("Cannot get the reference of the null tetromino.")
        };

        Tetromino { colour: * colour, anchor: * anchor, points: point_set, transform: Transform::Identity }
    }

    ///
    /// Sets up the movemap, which is the bijection between tetromino and integer.
    ///
    /// This function MUST be called before any type conversions are applied to 
    /// the Tetromino type, otherwise panics will occur.
    ///
    pub fn initialize ()
    {
        let board = Board::blank();
        let mut idx = 1;

        let mut fwd = MOVEMAP_FWD.write().unwrap(); 
        let mut rev = MOVEMAP_REV.write().unwrap();

        for tetromino in & board.enumerate_moves()
        {
            fwd.insert(tetromino.clone(), idx);
            rev.insert(idx, tetromino.clone());
            idx += 1;
        }

        // Null tetromino.

        let mut template = rev.get(& 1).unwrap().clone();
        template.colour = Colour::None;

        fwd.insert(template.clone(), 0);
        rev.insert(0, template.clone());
    }

    ///
    /// Determines if the given tetromino is null.
    ///
    pub fn is_null (& self) -> bool 
    {
        self.colour == Colour::None
    }

    ///
    /// Moves this tetromino.
    ///
    pub fn move_anchor (& mut self, anchor: & Point)
    {
        self.anchor = * anchor;
    }

    ///
    /// Generates a new tetromino with the given shape and transform, canonicalizing it.
    ///
    pub fn new (colour: & Colour, anchor: & Point, transform: & Transform) -> Tetromino 
    {
        let template = Tetromino::get_reference_tetromino(colour, anchor);
        transform.canonicalize(colour).apply_to_tetromino(& template)
    }

    ///
    /// Returns the null tetromino.
    ///
    pub fn null () -> Tetromino
    {
        0.into()
    }

    ///
    /// Returns a view on this tetromino's points.
    ///
    pub fn points (& self) -> & Vec<Point>
    {
        & self.points
    }

    ///
    /// Maps this tetromino's points against its anchor and returns the real positions of this
    /// tetromino's tiles on the board.
    ///
    pub fn points_real (& self) -> Vec<Point>
    {
        self.points.iter().map( |& p| self.anchor + p ).collect::<Vec<Point>>()
    }

    ///
    /// Gets the number of tetrominos possible, including null.
    ///
    pub fn range () -> usize 
    {
        MOVEMAP_FWD.read().unwrap().len()
    }

    ///
    /// Returns the transform on this piece in terms of its transformation from the identity.
    ///
    pub fn transform (& self) -> Transform 
    {
        self.transform
    }
}
