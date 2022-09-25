use geometry::Point;
use windows::Win32::Graphics::Direct2D::{ID2D1Factory1, ID2D1PathGeometry, ID2D1SolidColorBrush};

use super::BezierFeature;

#[derive(Debug, Clone)]
pub(crate) struct River<'a> {
    feature: BezierFeature,
    surface_brush: Option<ID2D1SolidColorBrush>,
    edgeline: Option<[Vec<Vec<Point>>; 2]>,
    surface: Option<ID2D1PathGeometry>,
    factory: &'a ID2D1Factory1,
}

impl<'a> River<'a> {
    pub(crate) fn new(factory: &'a ID2D1Factory1) -> Self {
        River {
            feature: BezierFeature::new(),
            surface_brush: None,
            edgeline: None,
            surface: None,
            factory,
        }
    }
}
