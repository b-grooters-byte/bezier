use geometry::bezier::Bezier;


pub(crate) enum CenterLine {
    Solid, 
    DoubleSolid,
    Stripe,
}

pub(crate) struct Road {
    centerline: Bezier,
    edge_curve: [Bezier; 2],
    edgeline_curve: Option<[Bezier; 2]>,
    width: f32,
    centerline_type: Option<CenterLine>,
    edgeline_visible: bool,
}

impl Road {

}


#[cfg(test)]
mod test {
    #[test]
    fn test_default() {

    }
}