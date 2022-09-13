use geometry::{bezier::Bezier, Point};

const DEFAULT_RESOLUTION: f32 = 0.025;
const DERIVED_CTRL_POINT: usize = 3;
const DERIVED_CTRL_POINT_MOD: f32 = 3.0;

pub const DEFAULT_ROAD_WIDTH: f32 = 6500.0;

#[derive(Debug, Clone)]
pub(crate) enum CenterLine {
    Solid,
    DoubleSolid,
    Stripe,
}

#[derive(Debug, Clone)]
pub(crate) struct BezierFeature {
    resolution: f32,
    pub centerline: Vec<Bezier>,
    edge_curve: Vec<[Vec<Point>; 2]>,
    edgeline_curve: Option<[Vec<Vec<Point>>; 2]>,
    ctrl_points: usize,
    width: f32,
    centerline_type: Option<CenterLine>,
    edgeline_visible: bool,
}

impl BezierFeature {
    pub(crate) fn new() -> Self {
        let b = Bezier::new_with_ctrl_point(
            [
                Point { x: 10.0, y: 10.0 },
                Point { x: 50.0, y: 10.0 },
                Point { x: 100.0, y: 10.0 },
                Point { x: 150.0, y: 100.0 },
            ],
            DEFAULT_RESOLUTION,
        );

        let r0 = Vec::<Point>::new();
        let r_pi = Vec::<Point>::new();
        let mut edge_curve = Vec::<[Vec<Point>; 2]>::new();
        edge_curve.push([r0, r_pi]);

        let mut road = BezierFeature {
            resolution: DEFAULT_RESOLUTION,
            centerline: Vec::<Bezier>::new(),
            edge_curve,
            ctrl_points: 4,
            edgeline_curve: None,
            width: DEFAULT_ROAD_WIDTH,
            centerline_type: None,
            edgeline_visible: false,
        };
        road.centerline.push(b);
        road
    }

    pub(crate) fn new_with_attributes(
        width: f32,
        centerline_type: Option<CenterLine>,
        edgeline_visible: bool,
    ) -> Self {
        let b = Bezier::new_with_ctrl_point(
            [
                Point { x: 10.0, y: 10.0 },
                Point { x: 50.0, y: 10.0 },
                Point { x: 100.0, y: 10.0 },
                Point { x: 150.0, y: 100.0 },
            ],
            DEFAULT_RESOLUTION,
        );

        let r0 = Vec::<Point>::new();
        let r_pi = Vec::<Point>::new();
        let mut edge_curve = Vec::<[Vec<Point>; 2]>::new();
        edge_curve.push([r0, r_pi]);

        let mut road = BezierFeature {
            resolution: DEFAULT_RESOLUTION,
            centerline: Vec::<Bezier>::new(),
            edge_curve,
            ctrl_points: 4,
            edgeline_curve: None,
            width,
            centerline_type,
            edgeline_visible,
        };
        road.centerline.push(b);
        road
    }

    pub(crate) fn segments(&self) -> &Vec<Bezier> {
        &self.centerline
    }

    pub(crate) fn mut_segments(&mut self) -> &mut Vec<Bezier> {
        &mut self.centerline
    }

    pub(crate) fn resolution(&self) -> f32 {
        self.resolution
    }

    pub(crate) fn set_resolution(&mut self, resolution: f32) {
        self.resolution = resolution;
        for b in self.centerline.iter_mut() {
            b.set_resolution(resolution);
        }
    }

    /// Gets the polygon path representing the surface of the road feature
    pub(crate) fn surface(&mut self) -> Vec<&geometry::Point> {
        let recalculate: Vec<bool> = self.centerline.iter().map(|b| b.is_modified()).collect();
        //        let points: Vec<&geometry::Point> =
        //            self.centerline.iter_mut().flat_map(|b| b.curve()).collect();
        for (idx, r) in recalculate.iter().enumerate() {
            if *r {
                self.centerline[idx].curve();
                self.calc_edge_curve(idx);
                // recalculate if an inset edge line is present
            }
        }
        let mut points_pi2: Vec<&geometry::Point> = self
            .edge_curve
            .iter()
            .flat_map(|v| v[0].iter())
            .collect::<Vec<&geometry::Point>>();
        let mut points_2pi: Vec<&geometry::Point> = self
            .edge_curve
            .iter().rev()
            .flat_map(|v| v[1].iter().rev())
            .collect::<Vec<&geometry::Point>>();

        let mut polygon = Vec::<&geometry::Point>::with_capacity(points_2pi.len() * 2);
        polygon.append(&mut points_pi2);
        polygon.append(&mut points_2pi);

        polygon
    }

    fn tangent_points(&mut self, idx: usize) -> Vec<geometry::Point> {
        let size = (1.0 / self.resolution + 1.0) as usize;

        let mut points = Vec::<geometry::Point>::with_capacity(size);

        let b = &self.centerline[idx];
        let mut d: [geometry::Point; DERIVED_CTRL_POINT] = [geometry::Point::default(); 3];
        for idx in 0..DERIVED_CTRL_POINT {
            d[idx] = DERIVED_CTRL_POINT_MOD * (b.ctrl_point(idx + 1) - b.ctrl_point(idx))
        }
        points.push(d[0]);
        for idx in 1..size - 1 {
            let t = self.resolution * idx as f32;
            let current_point = Point {
                x: d[0].x * (1.0 - t).powf(2.0)
                    + d[1].x * 2.0 * (1.0 - t) * t
                    + d[2].x * t.powf(2.0),
                y: d[0].y * (1.0 - t).powf(2.0)
                    + d[1].y * 2.0 * (1.0 - t) * t
                    + d[2].y * t.powf(2.0),
            };
            points.push(current_point);
        }
        points.push(d[2]);
        points
    }

    fn calc_edge_curve(&mut self, idx: usize) {
        let tangent_points = self.tangent_points(idx);
        let mut edge_curve = &mut self.edge_curve[idx];
        edge_curve[0].clear();
        edge_curve[1].clear();
        let curve = self.centerline[idx].curve();
        let width = self.width / 2.0;
        for (idx, point) in tangent_points.iter().enumerate() {
            let mut tan_x = point.x;
            let mut tan_y = point.y;
            let normal = (tan_x * tan_x + tan_y * tan_y).sqrt();
            tan_x /= normal;
            tan_y /= normal;
            edge_curve[0].push(Point {
                x: curve[idx].x + tan_y * width,
                y: curve[idx].y - tan_x * width,
            });
            edge_curve[1].push(Point {
                x: curve[idx].x - tan_y * width,
                y: curve[idx].y + tan_x * width,
            });
        }
    }

    /// Adds a new Bézier segment to an existing feature. Control points 0 and
    /// 1 are control points 3 and control point 2 reflected around control
    /// point 3 of the last segment currently in the feature.
    pub(crate) fn add_segment(&mut self, p2: Point, p3: Point) {
        let p1 = self.centerline.last().unwrap().ctrl_point(2);
        let p0 = self.centerline.last().unwrap().ctrl_point(3);

        let b = Bezier::new_with_ctrl_point([p0, p1.reflect(p0), p2, p3], self.resolution);
        self.ctrl_points += 4;
        self.centerline.push(b);

        let r0 = Vec::<Point>::new();
        let r_pi = Vec::<Point>::new();
        let mut edge_curve = Vec::<[Vec<Point>; 2]>::new();
        edge_curve.push([r0, r_pi]);
    }

    pub(crate) fn ctrl_point(&self, idx: usize) -> Option<geometry::Point> {
        let segment = idx / 4;
        let ctrl_idx = idx % 4;
        if segment >= self.centerline.len() {
            return None;
        }
        Some(self.centerline[segment].ctrl_point(ctrl_idx))
    }

    /// Sets a control point in the compound Beziér curve that defines the feature.
    /// The reflected control point is set when the control point is reflected
    /// around a segment joining control point.
    pub(crate) fn set_ctrl_point(&mut self, idx: usize, point: geometry::Point) {
        let curve = &mut self.centerline;
        let segment = idx / 4;
        let ctrl_point = idx % 4;
        // if this is an interior control point then adjust 2 point
        if curve.len() > 1 && (2..self.ctrl_points - 2).contains(&idx) {
            if ctrl_point == 0 || ctrl_point == 3 {
                // overlapped control point
                let affect: i32 = match ctrl_point {
                    0 => -1,
                    _ => 1,
                };
                let affected_segment = segment as i32 + affect;
                let affected_point = (idx as i32 + affect) % 4;
                curve[affected_segment as usize].set_ctrl_point(point, affected_point as usize);
            } else {
                // reflected control point
                let (reflect, affect) = match ctrl_point {
                    1 => (-1, -3),
                    _ => (1, 3),
                };
                // reflect around this point
                let reflect_segment_idx = segment as i32 + reflect;
                let relected_idx = (idx as i32 + affect) % 4;
                let around = (idx as i32 + reflect) % 4;
                let reflected_point = curve[segment]
                    .ctrl_point(ctrl_point)
                    .reflect(curve[segment].ctrl_point(around as usize));
                curve[reflect_segment_idx as usize]
                    .set_ctrl_point(reflected_point, relected_idx as usize);
            }
        }
        curve[segment].set_ctrl_point(point, ctrl_point);
    }
}

impl<'a> IntoIterator for &'a BezierFeature {
    type Item = &'a geometry::Point;
    type IntoIter = ControlPointIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ControlPointIterator::new(self)
    }
}

pub struct ControlPointIterator<'a> {
    points: Vec<&'a geometry::Point>,
    index: usize,
}

impl<'a> ControlPointIterator<'a> {
    pub(crate) fn new(feature: &'a BezierFeature) -> Self {
        let points: Vec<&geometry::Point> = feature
            .centerline
            .iter()
            .flat_map(|b| b.ctrl_points())
            .collect();

        Self { points, index: 0 }
    }
}

impl<'a> Iterator for ControlPointIterator<'a> {
    type Item = &'a Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.points.len() {
            let result = self.points[self.index];
            self.index += 1;
            return Some(result);
        }
        None
    }
}
