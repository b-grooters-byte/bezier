use clap::{ArgEnum, Parser};

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

    Ok(())
}
