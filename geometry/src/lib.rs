pub mod bezier;

use std::ops;

#[cfg(feature = "direct2d")]
use windows::Win32::Graphics::Direct2D::Common::D2D_POINT_2F;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn distance(&self, p: &Point) -> f32 {
        f32::sqrt((self.x - p.x) * (self.x - p.x) + (self.y - p.y) * (self.y - p.y))
    }

    pub fn dist_to_xy(&self, x: f32, y: f32) -> f32 {
        f32::sqrt((self.x - x) * (self.x - x) + (self.y - y) * (self.y - y))
    }

    /// Gets the slope of a line segment defined by the endpoints self and p
    ///
    /// Gets the slope for 2 points where:
    ///      m =  cy / cx
    /// or:
    ///          (y2 - y1)
    ///      m = ---------
    ///          (x2 - x1)
    /// the point passed in the parameter list is considered as x2, y2
    pub fn slope(&self, p: &Point) -> f32 {
        let cy = p.y - self.y;
        let cx = p.x - self.x;
        if cx == 0.0 {
            return f32::NAN;
        }
        cy / cx
    }

    /// Creates a point "reflected" around the point passed as a parameter.
    /// The reflection is both the X and Y axis so that a point {5.0,5.0}
    /// reflected around the orgin would result in a point of {-5.0, -5.0}
    pub fn reflect(&self, around: Point) -> Point {
        let delta = around - *self;
        *self - delta
    }
}

impl ops::Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::Sub<Point> for Point {
    type Output = Point;

    fn sub(self, rhs: Point) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[cfg(feature = "direct2d")]
impl From<Point> for D2D_POINT_2F {
    fn from(p: Point) -> D2D_POINT_2F {
        D2D_POINT_2F { x: p.x, y: p.y }
    }
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x && p.y <= self.x + self.width && p.y >= self.y && p.y <= self.y + self.height
    }
}

#[cfg(test)]
mod test {

    const EPSILON: f32 = 0.0001;

    use super::*;

    #[test]
    fn test_contains() {
        let r = Rect::new(10.0, 10.0, 10.0, 10.0);
        assert!(r.contains(Point { x: 15.0, y: 15.0 }));
    }

    #[test]
    fn test_reflect() {
        let p0 = Point { x: 0.0, y: 0.0 };
        let p1 = Point { x: -5.0, y: 5.0 };

        let p3 = p0.reflect(p1);

        assert_eq!(5.0, p3.x);
        assert_eq!(-5.0, p3.y);

        let p0 = Point { x: 5.0, y: 5.0 };
        let p1 = Point { x: 10.0, y: 10.0 };

        let p3 = p0.reflect(p1);

        assert_eq!(0.0, p3.x);
        assert_eq!(0.0, p3.y);
    }

    #[test]
    fn test_add() {
        let p0 = Point { x: 5., y: -5. };
        let p1 = Point { x: 10., y: 10. };

        let p3 = p0 + p1;

        assert!(p3.x - 15.0 < EPSILON);
        assert!(p3.y - 5.0 < EPSILON);
    }

    #[test]
    fn test_sub() {
        let p0 = Point { x: 5., y: -5. };
        let p1 = Point { x: 10., y: 10. };

        let p3 = p0 - p1;

        assert!(f32::abs(p3.x + 5.0) < EPSILON);
        assert!(f32::abs(p3.y + 15.0) < EPSILON);
    }
}
