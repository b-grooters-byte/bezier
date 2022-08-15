use geometry::{bezier::Bezier, Point};

const DEFAULT_RESOLUTION: f32 = 0.025;

pub(crate) enum CenterLine {
    Solid,
    DoubleSolid,
    Stripe,
}

pub(crate) struct Road {
    resolution: f32,
    centerline: Vec<Bezier>,
    edge_curve: [Vec<Vec<Point>>; 2],
    edgeline_curve: Option<[Vec<Vec<Point>>; 2]>,
    width: f32,
    centerline_type: Option<CenterLine>,
    edgeline_visible: bool,
}

impl Road {
    fn new() -> Self {
        Road {
            resolution: DEFAULT_RESOLUTION,
            centerline: Vec::<Bezier>::new(),
            edge_curve: [Vec::<Vec<Point>>::new(), Vec::<Vec<Point>>::new()],
            edgeline_curve: None,
            width: 0.0,
            centerline_type: None,
            edgeline_visible: false,
        }
    }

    pub(crate) fn add_segment(&mut self, p2: Point, p3: Point) {
        let p1 = self.centerline.last().unwrap().ctrl_point(2);
        let p0 = self.centerline.last().unwrap().ctrl_point(3);

        let b = Bezier::new_with_ctrl_point(
            [p0, p0.reflect(p1), p2, p3],
            self.resolution,
        );
    }

}
