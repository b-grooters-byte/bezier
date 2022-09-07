mod ui;

use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, MSG},
};

fn main() -> windows::core::Result<()> {
    let _m = ui::MainWindow::new("Bézier Demo");
    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, HWND(0), 0, 0).into() {
            DispatchMessageW(&message);
        }
    }
    Ok(())
}
