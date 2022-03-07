
use std::collections::BTreeSet;

use super::colour::Colour;
use super::point::Point;
use super::tetromino::Tetromino;

use utils::*;

///
/// An enum that represents the 8 possible transforms on the cartesian tetrominoes.
///
/// Identity refers to the null transformation, while Reflect refers to reflecting 
/// the tetromino in a mirror parallel to the y-axis.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Transform
{
    Identity,
    IdenRot90,
    IdenRot180,
    IdenRot270,
    Reflect,
    ReflRot90,
    ReflRot180,
    ReflRot270
}

impl std::ops::Add for & Transform 
{
    type Output = Transform;
    
    fn add (self, rhs: & Transform) -> Transform 
    {
        match rhs 
        {
            Transform::Identity   => * self, 
            Transform::IdenRot90  => self.rotate(),
            Transform::IdenRot180 => self.rotate().rotate(),
            Transform::IdenRot270 => self.rotate().rotate().rotate(),
            Transform::Reflect    => self.reflect(),
            Transform::ReflRot90  => self.reflect().rotate(),
            Transform::ReflRot180 => self.reflect().rotate().rotate(),
            Transform::ReflRot270 => self.reflect().rotate().rotate().rotate()
        }
    }
}

impl Transform 
{
    ///
    /// Applies this transform to the given point, treating it as a reference point.
    ///
    pub fn apply_to_point (& self, target: & Point) -> Point
    {
        let x  = target.x();
        let y  = target.y();

        match self 
        {
            Transform::Identity   => Point::new( x,  y),
            Transform::IdenRot90  => Point::new( y, -x),
            Transform::IdenRot180 => Point::new(-x, -y),
            Transform::IdenRot270 => Point::new(-y,  x),
            Transform::Reflect    => Point::new(-x,  y),
            Transform::ReflRot90  => Point::new( y,  x),
            Transform::ReflRot180 => Point::new( x, -y),
            Transform::ReflRot270 => Point::new(-y, -x)
        }
    }

    ///
    /// Applies this transform to the given tetromino, guarding by canonicalizing 
    /// against the colour. The anchor is preserved over transformation.
    ///
    pub fn apply_to_tetromino (& self, target: & Tetromino) -> Tetromino 
    {
        let mut points = target.points().clone();
        
        for p in & mut points 
        { 
            * p = self.canonicalize(& target.colour()).apply_to_point(& p);
        }
        Transform::normalize(& mut points);

        Tetromino::construct_raw(& target.colour(), & target.anchor(), & points, & (& target.transform() + self).canonicalize(& target.colour()))
    }

    ///
    /// Returns a vector of all of the transforms.
    ///
    pub fn as_array () -> Vec<Transform>
    {
        vec!
        [
            Transform::Identity,
            Transform::IdenRot90,
            Transform::IdenRot180,
            Transform::IdenRot270,
            Transform::Reflect,
            Transform::ReflRot90,
            Transform::ReflRot180,
            Transform::ReflRot270
        ]
    }

    ///
    /// Returns the canonical (most direct) transform for this transform and the given colour.
    ///
    pub fn canonicalize (& self, colour: & Colour) -> Transform 
    {
        match colour 
        {
            Colour::I => match self 
            {
                Transform::IdenRot180 | Transform::Reflect   | Transform::ReflRot180 => Transform::Identity,
                Transform::IdenRot270 | Transform::ReflRot90 | Transform::ReflRot270 => Transform::IdenRot90,
                _                                                                    => * self
            },
            Colour::T => match self 
            {
                Transform::Reflect    => Transform::Identity,
                Transform::ReflRot90  => Transform::IdenRot90,
                Transform::ReflRot180 => Transform::IdenRot180,
                Transform::ReflRot270 => Transform::IdenRot270,
                _                     => * self
            },
            Colour::S => match self 
            {
                Transform::IdenRot180 => Transform::Identity,
                Transform::IdenRot270 => Transform::IdenRot90,
                Transform::ReflRot180 => Transform::Reflect,
                Transform::ReflRot270 => Transform::ReflRot90,
                _                     => * self
            },
            _         => * self
        }
    }

    ///
    /// Returns the in-order vector of all canonical transforms of the given colour. 
    ///
    pub fn enumerate (colour: & Colour) -> Vec<Transform>
    {
        let mut set : BTreeSet<Transform> = BTreeSet::new();

        for transform in Transform::as_array()
        {
            set.insert(transform.canonicalize(colour));
        }

        set.into_iter().collect::<Vec<Transform>>()
    }

    ///
    /// Normalizes the points against the origin point, so that the top-left 
    /// corner of the bounding box over these points is the origin; returns 
    /// the true anchor of the input points.
    ///
    pub fn normalize (points: & mut Vec<Point>) -> Point 
    {
        let min_x  = points.into_iter().map( |p| p.x()).into_iter().min().unwrap();
        let min_y  = points.into_iter().map( |p| p.y()).into_iter().min().unwrap();
        let anchor = Point::new(min_x, min_y);
    
        for point in points 
        {
            * point = * point - anchor;
        }

        anchor
    }

    ///
    /// Returns the transform given by reflecting this transform.
    ///
    pub fn reflect (& self) -> Transform
    {
        match self 
        {
            Transform::Identity   => Transform::Reflect,
            Transform::IdenRot90  => Transform::ReflRot90,
            Transform::IdenRot180 => Transform::ReflRot180,
            Transform::IdenRot270 => Transform::ReflRot270,
            Transform::Reflect    => Transform::Identity, 
            Transform::ReflRot90  => Transform::IdenRot90,
            Transform::ReflRot180 => Transform::IdenRot180,
            Transform::ReflRot270 => Transform::IdenRot270
        }
    }

    ///
    /// Returns the transform given by rotating this transform by 90 degrees.
    ///
    pub fn rotate (& self) -> Transform 
    {
        match self 
        {
            Transform::Identity   => Transform::IdenRot90,
            Transform::IdenRot90  => Transform::IdenRot180,
            Transform::IdenRot180 => Transform::IdenRot270,
            Transform::IdenRot270 => Transform::Identity,
            Transform::Reflect    => Transform::ReflRot90,
            Transform::ReflRot90  => Transform::ReflRot180,
            Transform::ReflRot180 => Transform::ReflRot270,
            Transform::ReflRot270 => Transform::Reflect
        }
    }
}

