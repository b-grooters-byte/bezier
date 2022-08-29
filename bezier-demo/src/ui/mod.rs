use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

mod feature;
pub mod featureviewwidget;

glib::wrapper! {
    pub struct MainWindow(ObjectSubclass<MainWindowImpl>)
    @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
    @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl MainWindow {
    pub fn new(app: &gio::Application) -> Self {
        glib::Object::new(&[("application", app)]).expect("Failed to create window")
    }
}

#[derive(CompositeTemplate, Default)]
#[template(resource = "mainwindow.ui")]
pub struct MainWindowImpl {
    #[template_child]
    pub radio: TemplateChild<gtk::ToggleButton>,
}

#[glib::object_subclass]
impl ObjectSubclass for MainWindowImpl {
    const NAME: &'static str = "BezierDemoWindow";
    type Type = MainWindow;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(c: &mut Self::Class) {
        c.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for MainWindowImpl {}
impl WidgetImpl for MainWindowImpl {}
impl WindowImpl for MainWindowImpl {}
impl ApplicationWindowImpl for MainWindowImpl {}
