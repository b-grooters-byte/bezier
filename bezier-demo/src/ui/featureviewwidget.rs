use std::cell::Cell;

use geometry::bezier::Bezier;
use geometry::Point;

use glib::subclass::Signal;
use glib::{ParamSpecEnum, ParamSpec, ParamFlags, Type};
use glib::{StaticType, ToValue, once_cell::sync::Lazy};
use gtk::subclass::drawing_area::DrawingAreaImpl;
use gtk::subclass::prelude::*;


#[derive(Debug, Eq, PartialEq, Clone, Copy, glib::Enum)]
#[repr(i32)]
#[enum_type(name="FeatureTypeEnum")]
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
            feature_type: Cell::new(FeatureType::Road)
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
    feature_type: Cell<FeatureType>
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
        static PROPERTIES: Lazy<Vec<ParamSpec>> = 
        Lazy::new(|| vec![ParamSpecEnum::new("feature-type", 
            "type", 
            "Bezier feature type",
            FeatureType::static_type(), 
            FeatureType::default() as i32, 
            ParamFlags::READWRITE)]);
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
                let feature_type = value.get().expect("type conformity check: Object::set_property");
                self.feature_type.set(feature_type);
            },
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
