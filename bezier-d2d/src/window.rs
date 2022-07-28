use std::{sync::Once, ptr};

use windows::{
    Win32::{
        Foundation::{
            HINSTANCE,
            HWND, WPARAM, LPARAM, LRESULT,
        },
        UI::WindowsAndMessaging::{
            WNDCLASSW, LoadCursorW, IDC_ARROW, DefWindowProcW, RegisterClassW, CreateWindowExW, CW_USEDEFAULT, 
            HMENU, WS_OVERLAPPEDWINDOW, ShowWindow, SW_SHOW, WINDOW_EX_STYLE, CS_HREDRAW, CS_VREDRAW, PostQuitMessage, WS_VISIBLE, WM_PAINT, WM_DESTROY, COLOR_WINDOW,
        }, Graphics::Gdi::{ValidateRect, CreateSolidBrush}, 
    }, w, core::{Result, HSTRING}
};
use windows::{
    Win32::System::LibraryLoader::GetModuleHandleW,
};

static REGISTER_WINDOW_CLASS: Once = Once::new();
static WINDOW_CLASS_NAME: &HSTRING = w!("bytetrail.window.bezier");
pub(crate) struct Window {
    handle: HWND,
}

impl Window {
    pub(crate) fn new(title: &str) -> Result<Box<Self>> {
        let instance = unsafe { GetModuleHandleW(None)? };
        // synchronization for a one time initialization of FFI call
        REGISTER_WINDOW_CLASS.call_once(|| {

            // use defaults for all other fields
            let class = WNDCLASSW {
                lpfnWndProc: Some(Self::wnd_proc),
                hbrBackground: unsafe { CreateSolidBrush(COLOR_WINDOW.0) },
                hInstance: instance,
                style: CS_HREDRAW | CS_VREDRAW,
                hCursor: unsafe { LoadCursorW(HINSTANCE(0), IDC_ARROW).ok().unwrap()},
                lpszClassName: WINDOW_CLASS_NAME.into(),
                ..Default::default()
            };
            assert_ne!(unsafe { RegisterClassW(&class)}, 0);
        });

        let mut window_internal = Box::new(Self {
            handle: HWND(0),
        });


        // create the window using Self reference 
        let window = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(), 
                WINDOW_CLASS_NAME,
                &HSTRING::from(title), 
                WS_VISIBLE | WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT, 
                CW_USEDEFAULT, 
                400, 
                300, 
                HWND(0), 
                HMENU(0), 
                instance, 
                window_internal.as_mut() as *mut _ as _,
            )
        };
        unsafe { ShowWindow(window, SW_SHOW) };
        Ok(window_internal)
    }

    unsafe extern "system" fn wnd_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message as u32 {
            WM_PAINT => {
                println!("WM_PAINT");
                ValidateRect(window, ptr::null());
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ =>  DefWindowProcW(window, message, wparam, lparam),
        }
    }
}