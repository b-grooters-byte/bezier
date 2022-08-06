pub mod bezier;

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
    use super::*;

    #[test]
    fn test_contains() {
        let r = Rect::new(10.0, 10.0, 10.0, 10.0);
        assert!(r.contains(Point { x: 15.0, y: 15.0 }));
    }
}
