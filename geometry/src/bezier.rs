use crate::Point;

#[derive(Debug, Clone)]
pub struct Bezier {
    ctrl_point: [Point; 4],
    resolution: f32,
    length: f32,
    modified: bool,
    curve: Option<Vec<Point>>,
}

impl Bezier {
    pub fn new(resolution: f32) -> Self {
        Bezier {
            ctrl_point: [
                Point { x: 0.0, y: 0.0 },
                Point { x: 0.0, y: 0.0 },
                Point { x: 0.0, y: 0.0 },
                Point { x: 0.0, y: 0.0 },
            ],
            resolution,
            length: 0.0,
            modified: true,
            curve: None,
        }
    }
    /// Creates a new bezier curve with the specified parameters
    /// resolution - a value representing the resolution of the curve.
    /// 0.0 < value <= 1.0. A smaller value is a higher resolution
    pub fn new_with_ctrl_point(ctrl_point: [Point; 4], resolution: f32) -> Self {
        let bezier = Bezier {
            ctrl_point,
            resolution,
            length: 0.0,
            modified: true,
            curve: None,
        };
        bezier
    }

    pub fn len(&mut self) -> f32 {
        if self.modified {
            self.calc_curve();
        }
        let mut len = 0.0;
        let mut prev = &self.curve.as_ref().unwrap()[0];
        for p in self.curve.as_ref().unwrap().iter().skip(1) {
            len += prev.distance(p);
            prev = p;
        }
        self.length = len;
        self.length
    }

    pub fn set_resolution(&mut self, resolution: f32) {
        if self.resolution != resolution {
            self.resolution = resolution;
            self.modified = true;
        }
    }

    pub fn ctrl_point(&self, idx: usize) -> Point {
        assert!(idx < 4);
        self.ctrl_point[idx]
    }

    pub fn ctrl_points(&self) -> &[Point; 4] {
        &self.ctrl_point
    }

    pub fn set_ctrl_point(&mut self, p: Point, idx: usize) {
        assert!(idx < 4);
        self.ctrl_point[idx] = p;
        self.modified = true;
    }

    pub fn translate_point(&mut self, cx: f32, cy: f32, idx: usize) {
        assert!(idx < 4);
        self.ctrl_point[idx].x += cx;
        self.ctrl_point[idx].y += cy;
        self.modified = true;
    }

    pub fn translate(&mut self, cx: f32, cy: f32) {
        if cx != 0. && cy != 0. {
            for p in self.ctrl_point.as_mut() {
                p.x += cx;
                p.y += cy
            }
            self.modified = true;
        }
    }

    pub fn modified(&self) -> bool {
        self.modified
    }

    /// Gets the bezier curve represented as a vector of 2D Points.
    pub fn curve(&mut self) -> &Vec<Point> {
        if self.modified {
            self.calc_curve();
        }
        self.curve.as_ref().unwrap()
    }

    fn calc_curve(&mut self) {
        let size = (1.0 / self.resolution + 1.0) as usize;
        let mut curve = Vec::<Point>::with_capacity(size);
        curve.push(Point {
            x: self.ctrl_point[0].x,
            y: self.ctrl_point[0].y,
        });

        for i in 1..size - 1 {
            let t = self.resolution * (i - 1) as f32;
            let x = self.ctrl_point[0].x * (1.0 - t).powf(3.0)
                + self.ctrl_point[1].x * 3.0 * (1.0 - t).powf(2.0) * t
                + self.ctrl_point[2].x * 3.0 * (1.0 - t) * t * t
                + self.ctrl_point[3].x * t.powf(3.0);
            let y = self.ctrl_point[0].y * (1.0 - t).powf(3.0)
                + self.ctrl_point[1].y * 3.0 * (1.0 - t).powf(2.0) * t
                + self.ctrl_point[2].y * 3.0 * (1.0 - t) * t * t
                + self.ctrl_point[3].y * t.powf(3.0);
            curve.push(Point { x, y });
        }
        curve.push(Point {
            x: self.ctrl_point[3].x,
            y: self.ctrl_point[3].y,
        });
        self.modified = false;
        self.curve = Some(curve);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    const TEST_EPSILON: f32 = 0.005;

    #[test]
    fn test_new() {
        let b = Bezier::new(0.25);
        assert_eq!(0.25, b.resolution);
    }

    #[test]
    fn test_new_with_points() {
        let mut b = Bezier::new_with_ctrl_point(
            [
                Point { x: 0.0, y: 0.0 },
                Point { x: 1.0, y: 0.0 },
                Point { x: 9.0, y: 0.0 },
                Point { x: 10.0, y: 0.0 },
            ],
            0.125,
        );
        assert!((b.len() - 10.0).abs() < TEST_EPSILON);
    }

    #[test]
    fn test_set_resolution() {
        let mut b = Bezier::new(0.25);
        assert!(b.modified);
        b.set_resolution(0.15);
        assert_eq!(0.15, b.resolution);
    }
}
