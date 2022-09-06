use clap::{ArgEnum, Parser};
use gtk::prelude::*;
use gtk::Application;
use ui::MainWindow;

mod feature;
mod ui;

#[derive(ArgEnum, Debug, Clone)]
enum Feature {
    PavedRoad,
    River,
    Railroad,
}

#[derive(Parser, Debug)]
struct Args {
    /// Initial feature type to display
    #[clap(short, long, value_parser)]
    feature: Option<Feature>,
    /// The number of connected bezier segments to start with
    #[clap(short, long, value_parser)]
    segments: Option<u8>,
}

fn main() {
    gio::resources_register_include!("bezier_demo.gresource")
        .expect("failed to register resources");
    let app = Application::builder()
        .application_id("org.bytetrail.Bezier")
        .build();
    println!("application initialized");
    app.connect_activate(|app| {
        let window = MainWindow::new(app);

        window.show();
    });

    app.run();
}
