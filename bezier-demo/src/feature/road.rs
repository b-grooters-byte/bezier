use geometry::Point;
use windows::Win32::Graphics::Direct2D::{ID2D1SolidColorBrush, ID2D1HwndRenderTarget, ID2D1Factory, Common::{D2D1_FILL_MODE_WINDING, D2D1_FIGURE_BEGIN_FILLED, D2D1_FIGURE_END_CLOSED}, ID2D1PathGeometry, ID2D1StrokeStyle, ID2D1Factory1};
use crate::ui::direct2d::{create_brush, self};

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

#[derive(Debug, Clone)]
pub(crate) struct Road<'a> {
    feature: BezierFeature,
    surface_brush: Option<ID2D1SolidColorBrush>,
    centerline_brush: Option<ID2D1SolidColorBrush>,
    centerline: Option<[Vec<Vec<Point>>; 2]>,
    edgeline: Option<[Vec<Vec<Point>>; 2]>,
    surface: Option<ID2D1PathGeometry>,
    factory: &'a ID2D1Factory1,
    line_style: ID2D1StrokeStyle,
}

impl<'a> Road<'a> {

    pub(crate) fn new(factory: &'a ID2D1Factory1) -> Self {
        let line_style = direct2d::create_style(factory, None).expect("unable to create stroke style");
        Road { 
            feature: BezierFeature::new(),
            surface_brush: None,
            centerline_brush: None,
            centerline: None, 
            edgeline: None,
            surface: None,
            factory,
            line_style,
        }
    }

    pub(crate) fn feature(&self) -> &BezierFeature {
        &self.feature
    }

    pub(crate) fn feature_mut(&mut self) -> &mut BezierFeature {
        &mut self.feature
    }

    pub(crate) fn set_feature(&mut self, feature: BezierFeature) {
        self.feature = feature;
    }

    pub(crate) fn modified(&self) -> bool {
        self.feature.modified()
    }

    fn create_resources(& mut self, target: &ID2D1HwndRenderTarget) -> windows::core::Result<()> {
        self.surface_brush = Some(create_brush(target, ASPHALT_GRAY, ASPHALT_GRAY, ASPHALT_GRAY, 1.0)?);
        self.centerline_brush = Some(create_brush(target, CENTERLINE.0, CENTERLINE.1, CENTERLINE.2, CENTERLINE.3)?);

        Ok(())
    }

    fn release_resources(&mut self) {
        self.surface_brush = None;
        self.centerline_brush = None;
        self.surface = None;
    }

    fn draw(&mut self, target: ID2D1HwndRenderTarget) {
        let rebuild_geom = self.feature.modified();
        let centerline = self.feature.curve();
        if rebuild_geom {
            self.rebuild_geometry();
        }
        unsafe { target.FillGeometry(self.surface.as_ref().unwrap(), self.surface_brush.as_ref().unwrap(), None, ) };
        direct2d::draw_line(&target, &centerline, self.centerline_brush.as_ref().unwrap(), &self.line_style, 2.0);
    }

    fn rebuild_geometry(&mut self) {
        let surface_geom = unsafe { self.factory.CreatePathGeometry()}.unwrap();
        let points = self.feature.surface();
        let sink = unsafe {surface_geom.Open().unwrap()};
        unsafe {
            sink.SetFillMode(D2D1_FILL_MODE_WINDING);
            sink.BeginFigure((*points[0]).into(), D2D1_FIGURE_BEGIN_FILLED);
            for (_, point) in points.iter().enumerate().skip(1) {
                sink.AddLine((**point).into());
            }
            sink.EndFigure(D2D1_FIGURE_END_CLOSED);
            sink.Close().expect("unable to create geometry");
        }
        self.surface = Some(surface_geom);

    }

}