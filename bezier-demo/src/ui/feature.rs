use geometry::{bezier::Bezier, Point};

use crate::feature::road::{Road, CenterLine};

const RENDER_CTRL_HANDLE_RADIUS: f32 = 5.0;

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
    fn new(road_width: f32, centerline: Option<CenterLine>, edgeline: bool) -> Self {
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
    const DEFAULT_CTRL_PT_Y: f32 = 0.0;
    const DEFAULT_CTRL_PT_0_X: f32 = 0.0;
    const DEFAULT_CTRL_PT_1_X: f32 = 100.0;
    const DEFAULT_CTRL_PT_2_X: f32 = 200.0;
    const DEFAULT_CTRL_PT_3_X: f32 = 300.0;

    pub(crate) fn new() -> Self {
        RenderState {
            bezier: Bezier::new_with_ctrl_point(
                [
                    Point { x: RenderState::DEFAULT_CTRL_PT_0_X, y: RenderState::DEFAULT_CTRL_PT_Y },
                    Point { x: RenderState::DEFAULT_CTRL_PT_1_X, y: RenderState::DEFAULT_CTRL_PT_Y },
                    Point { x: RenderState::DEFAULT_CTRL_PT_2_X, y: RenderState::DEFAULT_CTRL_PT_Y },
                    Point { x: RenderState::DEFAULT_CTRL_PT_3_X, y: RenderState::DEFAULT_CTRL_PT_Y },
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
        let result = render_state.in_control_point(RenderState::DEFAULT_CTRL_PT_0_X, RenderState::DEFAULT_CTRL_PT_Y);
        assert!(result.is_some());
        assert_eq!(0, result.unwrap());
    }
}
