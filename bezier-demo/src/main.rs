use clap::{ArgEnum, Parser};
use gtk::{prelude::ApplicationExt, traits::WidgetExt, Application, ApplicationWindow};

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
    let app = Application::builder()
        .application_id("org.bytetrail.Bezier")
        .build();

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(600)
            .title("Bézier Curves")
            .build();

        window.show();
    });
}
