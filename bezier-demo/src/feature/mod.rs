pub mod road;

pub(crate) trait Feature {
    /// gets the control point count of the Bézier feature
    fn ctrl_points(&self) -> usize;
    /// Gets a single control point
    fn ctrl_point(&self, idx: usize) -> geometry::Point;
    /// Sets a control point to the point passed in the parameter list.
    fn set_ctrl_point(&mut self, idx: usize, point: geometry::Point);
}
