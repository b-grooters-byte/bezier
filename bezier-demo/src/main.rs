mod feature;
mod ui;

use ui::direct2d;
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, MSG},
};

fn main() -> windows::core::Result<()> {
    let factory = direct2d::create_factory()?;
    let _m = ui::MainWindow::new("Bézier Demo", &factory);
    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, HWND(0), 0, 0).into() {
            DispatchMessageW(&message);
        }
    }
    Ok(())
}
