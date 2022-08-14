use clap::{ArgEnum, Parser};
use ui::MainWindow;
use windows::Win32::{UI::WindowsAndMessaging::{MSG, GetMessageW, DispatchMessageW, TranslateMessage}, Foundation::HWND};

mod ui;
mod road;

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

fn main() -> windows::core::Result<()> {
    let _args = Args::parse();

    let _ = MainWindow::new("Bézier Demo");
    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, HWND(0), 0, 0).into() {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }

    Ok(())
}
