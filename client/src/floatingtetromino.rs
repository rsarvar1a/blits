
use lits::*;

///
/// An encapsulation of a tetromino that binds to a user's mouse location and snaps 
/// to the playing field.
///
pub struct FloatingTetromino 
{
    tetromino: Tetromino,
    rel_x: f32, 
    rel_y: f32
}

impl FloatingTetromino 
{
    ///
    /// Sets the tetromino to its next transform.
    ///
    pub fn next (& mut self)
    {
        let transforms = Tetromino::get_reference_tetromino(& self.tetromino.colour(), & self.tetromino.anchor()).enumerate_transforms();
        let mut index = transforms.iter().position(|t| t.clone() == self.tetromino).unwrap();

        index = match index + 1 == transforms.len()
        {
            true  => 0,
            false => index + 1
        };

        self.tetromino = transforms.get(index).unwrap().clone();
    }

    ///
    /// Returns a new floating tetromino.
    /// 
    pub fn new (tetromino: & Tetromino, x: f32, y: f32) -> FloatingTetromino 
    {
        FloatingTetromino 
        {
            tetromino: tetromino.clone(),
            rel_x: x,
            rel_y: y
        }
    }

    ///
    /// Gets the previous transform.
    ///
    pub fn prev (& mut self) 
    {
        let transforms = Tetromino::get_reference_tetromino(& self.tetromino.colour(), & self.tetromino.anchor()).enumerate_transforms();
        let index = transforms.iter().position(|t| t.clone() == self.tetromino).unwrap() as i32;

        let index = match index - 1 == -1
        {
            true  => transforms.len() as i32 - 1,
            false => index - 1
        } as usize;

        self.tetromino = transforms.get(index).unwrap().clone();
    }

    ///
    /// Sets a new anchor.
    ///
    pub fn set_anchor (& mut self, point: Point)
    {
        self.tetromino.move_anchor(& point);
    }

    ///
    /// Snaps the current float position to a point, if possible.
    ///
    pub fn snap (& self) -> Option<Point>
    {
        if (self.rel_x.round() - self.rel_x).abs() < 0.333
            && (self.rel_y.round() - self.rel_y).abs() < 0.333 
        {
            return Some(lits::Point::new(self.rel_x.round() as i32, self.rel_y.round() as i32));
        }
        None
    }

    ///
    /// Returns the backing tetromino.
    ///
    pub fn tetromino (& self) -> Tetromino 
    {
        self.tetromino.clone()
    }

    ///
    /// Returns a mut reference to the x position.
    ///
    pub fn x (& mut self) -> & mut f32 
    {
        & mut self.rel_x 
    }

    ///
    /// Returns a mut reference to the y position.
    ///
    pub fn y (& mut self) -> & mut f32
    {
        & mut self.rel_y
    }
}

