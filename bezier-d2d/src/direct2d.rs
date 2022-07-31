use std::ptr;

use windows::Win32::Graphics::Direct2D::{ID2D1Factory1, D2D1_FACTORY_OPTIONS, D2D1_DEBUG_LEVEL_INFORMATION, D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1CreateFactory, ID2D1StrokeStyle, D2D1_STROKE_STYLE_PROPERTIES, D2D1_CAP_STYLE_ROUND};
use windows::core::Result;

/// Creates a single threaded Direct2D factory with default options.
pub(crate) fn create_factory() -> Result<ID2D1Factory1> {
    let mut options = D2D1_FACTORY_OPTIONS::default();

    if cfg!(debug_assertions) {
        options.debugLevel = D2D1_DEBUG_LEVEL_INFORMATION;
    }

    unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, &options) }
}

/// Create a stroke style with the specified dash pattern
pub(crate) fn create_style(factory: &ID2D1Factory1, dashes: &[f32]) -> Result<ID2D1StrokeStyle> {
    let props = D2D1_STROKE_STYLE_PROPERTIES { startCap: D2D1_CAP_STYLE_ROUND, 
        endCap: D2D1_CAP_STYLE_ROUND, ..Default::default() };
    unsafe { factory.CreateStrokeStyle(&props, dashes) }
}
