mod window;

use windows::{Win32::{UI::WindowsAndMessaging::{MessageBoxA, MB_OK, GetMessageW, TranslateMessage, DispatchMessageW, MSG}, Foundation::HWND}, s};




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
