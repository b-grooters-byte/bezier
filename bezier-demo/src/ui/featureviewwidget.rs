use geometry::bezier::Bezier;
use geometry::Point;

use gtk::subclass::prelude::*;
use gtk::subclass::drawing_area::DrawingAreaImpl;

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

    fn class_init(class: &mut Self::Class) {

    }

    fn new() -> Self {
        Self{}
    }
}

impl WidgetImpl for FeatureViewPriv {

}

impl DrawingAreaImpl for FeatureViewPriv {
    fn resize(&self, drawing_area: &Self::Type, width: i32, height: i32) {
        self.parent_resize(drawing_area, width, height)
    }
    
}

pub struct FeatureViewPriv {}

impl ObjectImpl for FeatureViewPriv {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
    }

    fn signals() -> &'static [cairo::glib::subclass::Signal] {
        todo!()
    }

    fn properties() -> &'static [cairo::glib::ParamSpec] {
        todo!()
        
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, _value: &cairo::glib::Value, _pspec: &cairo::glib::ParamSpec) {
        
    }

    fn property(&self, _obj: &Self::Type, _id: usize, _pspec: &glib::ParamSpec) -> glib::Value {
        todo!()
    }
}


