use crate::feature::road::Road;
use geometry::bezier::Bezier;
use geometry::Point;

trait FeatureVisual {
    fn draw(&mut self, context: &cairo::Context);
}

impl core::fmt::Debug for dyn FeatureVisual {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug)]
struct RoadVisual {
    road: Road,
    selected: Option<usize>,
    hover: Option<usize>,
}

impl RoadVisual {
    fn new(road_width: f32, centerline: bool, edgeline: bool) -> Self {
        let road = Road::new_with_attributes(road_width, None, false);
        RoadVisual {
            road,
            selected: None,
            hover: None,
        }
    }

    fn draw_surface(&mut self, context: &cairo::Context) {
        let surface = self.road.surface();
    }
}

impl FeatureVisual for RoadVisual {
    fn draw(&mut self, context: &cairo::Context) {}
}

const RENDER_CTRL_HANDLE_RADIUS: f32 = 5.0;

#[derive(Debug, Clone)]
pub struct RenderState {
    pub bezier: Bezier,
    pub hover: Option<usize>,
    pub selected: Option<usize>,
}

impl RenderState {
    pub(crate) fn new() -> Self {
        RenderState {
            bezier: Bezier::new_with_ctrl_point(
                [
                    Point { x: 20.0, y: 20.0 },
                    Point { x: 120.0, y: 20.0 },
                    Point { x: 220.0, y: 220.0 },
                    Point { x: 320.0, y: 20.0 },
                ],
                0.025,
            ),
            hover: None,
            selected: None,
        }
    }

    fn in_control_point(&self, x: f32, y: f32) -> Option<usize> {
        for (idx, ctrl) in self.bezier.ctrl_points().iter().enumerate() {
            if ctrl.dist_to_xy(x, y) <= RENDER_CTRL_HANDLE_RADIUS {
                return Some(idx);
            }
        }
        None
    }
}
