
use utils::error::Context;
use utils::*;

///
/// Represents a normal cartesian point.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Point 
{
    x: i32,
    y: i32
}

impl std::fmt::Display for Point 
{
    fn fmt (& self, f: & mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl notate::Notate for Point 
{
    fn notate (& self) -> String 
    {
        format!("{:02}", self.x * 10 + self.y)
    }

    fn parse (s: & str) -> Result<Point>
    {
        let context = format!("Invalid notation '{}' for point.", s);

        if s.len() != 2
        {
            return Err(error::error!("Invalid length {}, expected 2.", s.len())).context(context.clone());
        }

        let val = s.parse::<i32>().context(context.clone())?;

        if ! (0 <= val && val <= 99)
        {
            return Err(error::error!("Invalid value {}, expected a number in the range 0 ..= 99.", val)).context(context.clone());
        }

        Ok(Point { x: val / 10 as i32, y: val % 10 as i32 })
    }
}

impl std::ops::Add for Point 
{
    type Output = Point;

    fn add (self, rhs: Point) -> Point 
    {
        Point { x: self.x + rhs.x, y: self.y + rhs.y } 
    }
}

impl std::ops::Sub for Point 
{
    type Output = Point;
    
    fn sub (self, rhs: Point) -> Point
    {
        Point { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl Point 
{
    ///
    /// Gets the potential anchors that reach this real point.
    ///
    pub fn get_potential_anchors (& self) -> Vec<Point>
    {
        let mut result = Vec::new();
        for x in 0 ..= 3 
        {
            let x = -x;
            for y in 0 ..= 3 + x
            {
                let y = -y;
                let anchor = Point::new(x, y) + * self;
                if anchor.in_bounds()
                {
                    result.push(anchor);
                }
            }
        }
        result
    }

    ///
    /// Determines whether this point is on the board.
    ///
    pub fn in_bounds (& self) -> bool
    {
        0 <= self.x && self.x <= 9 && 0 <= self.y && self.y <= 9
    }

    ///
    /// Gets all neighbours of this point.
    ///
    pub fn neighbours (& self) -> Vec<Point>
    {
        vec! 
        [
            Point::new(self.x - 1, self.y),
            Point::new(self.x + 1, self.y),
            Point::new(self.x, self.y - 1),
            Point::new(self.x, self.y + 1)
        ]
    }

    ///
    /// Gets all on-board neighbours of this point.
    ///
    pub fn neighbours_on_board (& self) -> Vec<Point>
    {
        self.neighbours().into_iter().filter( |& p| p.in_bounds() ).collect::<Vec<Point>>()
    }

    ///
    /// Returns a new point.
    ///
    pub fn new (x: i32, y: i32) -> Point
    {
        Point { x, y }
    }

    ///
    /// Returns x.
    ///
    pub fn x (& self) -> i32
    {
        self.x
    }

    ///
    /// Returns y.
    ///
    pub fn y (& self) -> i32
    {
        self.y
    }
}
