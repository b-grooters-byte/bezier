use crate::ui::direct2d::{self, create_brush};
use geometry::Point;
use windows::Win32::Graphics::Direct2D::{
    ID2D1Factory1, ID2D1HwndRenderTarget, ID2D1PathGeometry, ID2D1SolidColorBrush, ID2D1StrokeStyle,
};

use super::BezierFeature;

pub const DEFAULT_ROAD_WIDTH: f32 = 50.0;
const ASPHALT_GRAY: f32 = 0.65;
const CENTERLINE: (f32, f32, f32, f32) = (0.98, 0.665, 0.0, 1.0);

#[derive(Debug, Clone)]
pub(crate) enum CenterLine {
    Solid,
    DoubleSolid,
    Stripe,
    StripeSolid,
}

pub(crate) struct Road<'a> {
    modified: bool,
    pub(crate) surface_brush: Option<ID2D1SolidColorBrush>,
    pub(crate) centerline_brush: Option<ID2D1SolidColorBrush>,
    centerline: Option<[Vec<Vec<Point>>; 2]>,
    edgeline: Option<[Vec<Vec<Point>>; 2]>,
    surface: Option<ID2D1PathGeometry>,
    factory: &'a ID2D1Factory1,
    pub(crate) line_style: ID2D1StrokeStyle,
}

impl<'a> Road<'a> {
    pub(crate) fn new(factory: &'a ID2D1Factory1) -> Self {
        let line_style =
            direct2d::create_style(factory, None).expect("unable to create stroke style");
        Road {
            modified: false,
            surface_brush: None,
            centerline_brush: None,
            centerline: None,
            edgeline: None,
            surface: None,
            factory,
            line_style,
        }
    }

    pub(crate) fn set_modified(&mut self, modified: bool) {
        self.modified = modified
    }

    pub(crate) fn modified(&self) -> bool {
         self.modified
    }

    pub(crate) fn create_resources(
        &mut self,
        target: &ID2D1HwndRenderTarget,
    ) -> windows::core::Result<()> {
        self.surface_brush = Some(create_brush(
            target,
            ASPHALT_GRAY,
            ASPHALT_GRAY,
            ASPHALT_GRAY,
            1.0,
        )?);
        self.centerline_brush = Some(create_brush(
            target,
            CENTERLINE.0,
            CENTERLINE.1,
            CENTERLINE.2,
            CENTERLINE.3,
        )?);

        Ok(())
    }

    pub(crate) fn release_resources(&mut self) {
        self.surface_brush = None;
        self.centerline_brush = None;
        self.surface = None;
    }

}
