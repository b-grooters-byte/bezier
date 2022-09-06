use std::cell::Cell;

use glib::subclass::Signal;
use glib::{once_cell::sync::Lazy, StaticType, ToValue};
use glib::{ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecFloat, Type};
use gtk::subclass::drawing_area::DrawingAreaImpl;
use gtk::subclass::prelude::*;

use crate::feature::road::{CenterLine, DEFAULT_ROAD_WIDTH};

use super::feature::RoadVisual;

#[derive(Debug, Eq, PartialEq, Clone, Copy, glib::Enum)]
#[repr(i32)]
#[enum_type(name = "FeatureTypeEnum")]
pub enum FeatureType {
    Road = 0,
    River = 1,
    Railroad = 2,
}

impl Default for FeatureType {
    fn default() -> Self {
        FeatureType::Road
    }
}

glib::wrapper! {
    pub struct FeatureViewWidget(
        ObjectSubclass<FeatureViewPriv>)
        @extends gtk::Box, gtk::Widget, gtk::DrawingArea;
}

impl FeatureViewWidget {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create FeatureViewWidget")
    }
}

#[glib::object_subclass]
impl ObjectSubclass for FeatureViewPriv {
    const NAME: &'static str = "FeatureViewWidget";
    type Type = FeatureViewWidget;
    type ParentType = gtk::DrawingArea;

    fn class_init(_class: &mut Self::Class) {}

    fn new() -> Self {
        Self {
            feature_type: Cell::new(FeatureType::Road),
            feature: Cell::new(RoadVisual::new(
                DEFAULT_ROAD_WIDTH,
                Some(CenterLine::Solid),
                false,
            )),
            map_scale: Cell::new(200.0),
        }
    }
}

impl WidgetImpl for FeatureViewPriv {}

impl DrawingAreaImpl for FeatureViewPriv {
    fn resize(&self, drawing_area: &Self::Type, width: i32, height: i32) {
        self.parent_resize(drawing_area, width, height)
    }

}

pub struct FeatureViewPriv {
    feature_type: Cell<FeatureType>,
    feature: Cell<RoadVisual>,
    map_scale: Cell<f32>,
}

impl ObjectImpl for FeatureViewPriv {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
    }

    fn signals() -> &'static [cairo::glib::subclass::Signal] {
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![Signal::builder(
                // Signal name
                "maximum-segment-count",
                // Types of the values which will be sent to the signal handler
                &[i32::static_type().into()],
                // Type of the value the signal handler sends back
                <()>::static_type().into(),
            )
            .build()]
        });
        SIGNALS.as_ref()
    }

    fn properties() -> &'static [cairo::glib::ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![
                ParamSpecEnum::new(
                    "feature-type",
                    "type",
                    "Bezier feature type",
                    FeatureType::static_type(),
                    FeatureType::default() as i32,
                    ParamFlags::READWRITE,
                ),
                ParamSpecFloat::new(
                    "map-scale",
                    "scale",
                    "Map scale",
                    10.0,
                    1e10,
                    200.0,
                    ParamFlags::READWRITE,
                ),
            ]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(
        &self,
        _obj: &Self::Type,
        _id: usize,
        value: &cairo::glib::Value,
        pspec: &cairo::glib::ParamSpec,
    ) {
        match pspec.name() {
            "feature-type" => {
                let feature_type = value
                    .get()
                    .expect("type conformity check: Object::set_property");
                self.feature_type.set(feature_type);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "feature-type" => self.feature_type.get().to_value(),
            _ => unimplemented!(),
        }
    }

    fn dispose(&self, _obj: &Self::Type) {}
}
