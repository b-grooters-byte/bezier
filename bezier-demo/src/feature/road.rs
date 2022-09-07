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
pub(crate) struct Road {
    resolution: f32,
    pub centerline: Vec<Bezier>,
    edge_curve: Vec<[Vec<Point>; 2]>,
    edgeline_curve: Option<[Vec<Vec<Point>>; 2]>,
    width: f32,
    centerline_type: Option<CenterLine>,
    edgeline_visible: bool,
}

impl Road {
    pub(crate) fn new() -> Self {
        Road {
            resolution: DEFAULT_RESOLUTION,
            centerline: Vec::<Bezier>::new(),
            edge_curve: Vec::<[Vec<Point>; 2]>::new(),
            edgeline_curve: None,
            width: DEFAULT_ROAD_WIDTH,
            centerline_type: None,
            edgeline_visible: false,
        }
    }

    pub(crate) fn new_with_attributes(
        width: f32,
        centerline_type: Option<CenterLine>,
        edgeline_visible: bool,
    ) -> Self {
        Road {
            resolution: DEFAULT_RESOLUTION,
            centerline: Vec::<Bezier>::new(),
            edge_curve: Vec::<[Vec<Point>; 2]>::new(),
            edgeline_curve: None,
            width,
            centerline_type,
            edgeline_visible,
        }
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
            .iter()
            .flat_map(|v| v[1].iter())
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
        let edge_curve = &mut self.edge_curve[idx];
        let curve = self.centerline[idx].curve();
        for (idx, point) in tangent_points.iter().enumerate() {
            let mut tan_x = point.x;
            let mut tan_y = point.y;
            let normal = (tan_x * tan_x + tan_y * tan_y).sqrt();
            tan_x /= normal;
            tan_y /= normal;
            edge_curve[idx][0] = Point {
                x: curve[idx].x + tan_y * self.width,
                y: curve[idx].y - tan_x * self.width,
            };
            edge_curve[idx][1] = Point {
                x: curve[idx].x - tan_y * self.width,
                y: curve[idx].y + tan_x * self.width,
            };
        }
    }

    /// Adds a new Bézier segment to an existing feature. Control points 0 and
    /// 1 are control points 3 and control point 2 reflected around control
    /// point 3 of the last segment currently in the feature.
    pub(crate) fn add_segment(&mut self, p2: Point, p3: Point) {
        let p1 = self.centerline.last().unwrap().ctrl_point(2);
        let p0 = self.centerline.last().unwrap().ctrl_point(3);

        let b = Bezier::new_with_ctrl_point([p0, p0.reflect(p1), p2, p3], self.resolution);
    }
}
