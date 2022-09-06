use geometry::{bezier::Bezier, Point};

use crate::feature::road::{CenterLine, Road};

const RENDER_CTRL_HANDLE_RADIUS: f32 = 5.0;

trait FeatureVisual {
    fn draw(&mut self, context: &cairo::Context);
}

impl core::fmt::Debug for dyn FeatureVisual {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct RoadVisual {
    road: Road,
    selected: Option<usize>,
    hover: Option<usize>,
}

impl RoadVisual {
    pub(crate) fn new(road_width: f32, centerline: Option<CenterLine>, edgeline: bool) -> Self {
        let road = Road::new_with_attributes(road_width, centerline, edgeline);
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
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 0.0, y: 0.0 },
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_in_ctrl_point() {
        let render_state = RenderState::new();
        let result = render_state.in_control_point(0.0, 0.0);
        assert!(result.is_some());
        assert_eq!(0, result.unwrap());
    }
}
