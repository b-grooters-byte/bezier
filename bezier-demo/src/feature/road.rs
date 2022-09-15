

pub const DEFAULT_ROAD_WIDTH: f32 = 50.0;

#[derive(Debug, Clone)]
pub(crate) enum CenterLine {
    Solid,
    DoubleSolid,
    Stripe,
}
