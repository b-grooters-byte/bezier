use windows::Win32::Graphics::Gdi::CreateSolidBrush;
use windows::core::{Result, HSTRING};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    LoadCursorW, RegisterClassW, CS_HREDRAW, CS_VREDRAW, IDC_ARROW, WNDCLASSW, COLOR_WINDOW, WS_CHILD, WINDOW_EX_STYLE, WS_VISIBLE, WS_TABSTOP, CW_USEDEFAULT, HMENU, CreateWindowExW,
};
use windows::w;
use crate::ui::{REGISTER_WINDOW_CLASS, WINDOW_CLASS_NAME};
static FEATURE_CLASS_NAME: &HSTRING = w!("bytetrail.window.feature_view");


pub(crate) struct FeatureView {
    handle: HWND,
}

impl FeatureView {
    pub(crate) fn new(instance: HINSTANCE) -> Result<Box<Self>> {
        let mut view = Box::new(Self { handle: HWND(0) });

        REGISTER_WINDOW_CLASS.call_once(|| {
            // use defaults for all other fields
            let class = WNDCLASSW {
                lpfnWndProc: Some(Self::wnd_proc),
                hbrBackground: unsafe { CreateSolidBrush(COLOR_WINDOW.0) },
                hInstance: instance,
                style: CS_HREDRAW | CS_VREDRAW,
                hCursor: unsafe { LoadCursorW(HINSTANCE(0), IDC_ARROW).ok().unwrap() },
                lpszClassName: FEATURE_CLASS_NAME.into(),
                ..Default::default()
            };
            assert_ne!(unsafe { RegisterClassW(&class) }, 0);
        });

        let mut dpix = 0.0;
        let mut dpiy = 0.0;

        unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                WINDOW_CLASS_NAME,
                None,
                WS_VISIBLE | WS_CHILD | WS_TABSTOP,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                400,
                300,
                HWND(0),
                HMENU(0),
                instance,
                view.as_mut() as *mut _ as _,
            )
        };
        Ok(view)
    }

    unsafe extern "system" fn wnd_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        LRESULT(0)
    }

}