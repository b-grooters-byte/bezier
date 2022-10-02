use geometry::Point;
use windows::Win32::Graphics::Direct2D::{
    ID2D1Factory1, ID2D1HwndRenderTarget, ID2D1PathGeometry, ID2D1SolidColorBrush,
};

use super::BezierFeature;

#[derive(Debug)]
pub(crate) struct River<'a> {
    feature: Option<BezierFeature>,
    surface_brush: Option<ID2D1SolidColorBrush>,
    edgeline: Option<[Vec<Vec<Point>>; 2]>,
    surface: Option<ID2D1PathGeometry>,
    factory: &'a ID2D1Factory1,
}

impl<'a> River<'a> {
    pub(crate) fn new(factory: &'a ID2D1Factory1) -> Self {
        River {
            feature: None,
            surface_brush: None,
            edgeline: None,
            surface: None,
            factory,
        }
    }

    pub(crate) fn feature(&self) -> Option<&BezierFeature> {
        self.feature.as_ref()
    }

    pub(crate) fn feature_mut(&mut self) -> Option<&mut BezierFeature> {
        self.feature.as_mut()
    }

    pub(crate) fn set_feature(&mut self, feature: BezierFeature) {
        self.feature = Some(feature);
    }

    pub(crate) fn take_feature(&mut self) -> Option<BezierFeature> {
        self.feature.take()
    }

    pub(crate) fn modified(&self) -> bool {
        match &self.feature {
            Some(feature) => feature.modified(),
            _ => false,
        }
    }

    pub(crate) fn draw(&mut self, target: ID2D1HwndRenderTarget) {
        if self.feature.is_none() {
            return;
        }

        let feature = self.feature.as_mut().unwrap();
        let rebuild_geom = feature.modified();
        if rebuild_geom {
            self.surface = Some(super::rebuild_geometry(
                self.feature.as_mut().unwrap(),
                self.factory,
            ));
        }
        unsafe {
            target.FillGeometry(
                self.surface.as_ref().unwrap(),
                self.surface_brush.as_ref().unwrap(),
                None,
            )
        };
    }
}
