use std::sync::Once;

use windows::Win32::Graphics::Gdi::CreateSolidBrush;
use windows::core::{Result, HSTRING};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM, GetLastError};
use windows::Win32::UI::WindowsAndMessaging::{
    LoadCursorW, RegisterClassW, CS_HREDRAW, CS_VREDRAW, IDC_ARROW, WNDCLASSW, COLOR_WINDOW, WS_CHILD, WINDOW_EX_STYLE, WS_VISIBLE, WS_TABSTOP, CW_USEDEFAULT, HMENU, CreateWindowExW, WM_CREATE, CREATESTRUCTA, SetWindowLongPtrA, GWLP_USERDATA, GetWindowLongPtrA, DefWindowProcW, COLOR_MENUTEXT, HICON, WS_CHILDWINDOW, COLOR_DESKTOP,
};
use windows::w;

static REGISTER_FEATURE_WINDOW_CLASS: Once = Once::new();
static FEATURE_CLASS_NAME: &HSTRING = w!("bytetrail.window.feature_view");


pub(crate) struct FeatureView {
    pub(crate) handle: HWND,
}

impl FeatureView {
 
    pub(crate) fn new(instance: HINSTANCE, parent: HWND) -> Result<Box<Self>> {
        let mut view = Box::new(Self { handle: HWND(0) });

        REGISTER_FEATURE_WINDOW_CLASS.call_once(|| {
            // use defaults for all other fields
            let class = WNDCLASSW {
                lpfnWndProc: Some(Self::feature_wnd_proc),
                hbrBackground: unsafe { CreateSolidBrush(0x0000FFFF) },
                hInstance: instance,
                style: CS_HREDRAW | CS_VREDRAW,
                hCursor: unsafe { LoadCursorW(HINSTANCE(0), IDC_ARROW).ok().unwrap() },
                lpszClassName: FEATURE_CLASS_NAME.into(),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hIcon: HICON(0),
                ..Default::default()
            };
            assert_ne!(unsafe { RegisterClassW(&class) }, 0);
        });


        unsafe {
            let r = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                FEATURE_CLASS_NAME,
                None,
                WS_VISIBLE | WS_CHILDWINDOW,
                0, 0, 0, 0,
                parent,
                HMENU(0),
                instance,
                view.as_mut() as *mut _ as _,
            );

        };
        Ok(view)   
    }

    fn message_loop(
        &mut self,
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM
    ) -> LRESULT {
        match message {
            _ =>  unsafe { DefWindowProcW(window, message, wparam, lparam) }
        }
    }

    unsafe extern "system" fn feature_wnd_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == WM_CREATE {
            let create_struct = lparam.0 as *const CREATESTRUCTA;
            let this = (*create_struct).lpCreateParams as *mut Self;
            (*this).handle = window;

            SetWindowLongPtrA(window, GWLP_USERDATA, this as _);
        }
        else {
            let this = GetWindowLongPtrA(window, GWLP_USERDATA) as *mut Self;

            if !this.is_null() {
                return (*this).message_loop(window, message, wparam, lparam);
            }
    
        }
        
        DefWindowProcW(window, message, wparam, lparam)
    }

}