mod direct2d;
mod window;

use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, MSG},
};

fn main() -> windows::core::Result<()>{
    let _main_window = window::Window::new("Bezier")?;
    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, HWND(0), 0, 0).into() {
            //TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    Ok(())
}
