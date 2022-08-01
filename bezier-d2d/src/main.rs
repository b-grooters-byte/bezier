mod direct2d;
mod window;

use windows::{
    s,
    Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, MessageBoxA, TranslateMessage, MB_OK, MSG,
        },
    },
};

fn main() {
    let main_window = window::Window::new("Bezier");

    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, HWND(0), 0, 0).into() {
            //TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}
