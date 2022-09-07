use self::feature::FeatureWindow;
use std::sync::Once;
use windows::{
    core::HSTRING,
    w,
    Win32::{
        Foundation::{GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{HBRUSH},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{ 
            CreateWindowExW, DefWindowProcW, GetWindowLongPtrA, LoadCursorW, MoveWindow,
            PostQuitMessage, RegisterClassW, SetWindowLongPtrA, ShowWindow, COLOR_WINDOW,
            CREATESTRUCTA, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, HMENU, IDC_ARROW,
            SW_SHOW, WINDOW_EX_STYLE, WM_CREATE, WM_DESTROY, WM_SIZE, WNDCLASSW,
            WS_OVERLAPPEDWINDOW, WS_VISIBLE, WS_CHILD, BS_RADIOBUTTON, WINDOW_STYLE, COLOR_BACKGROUND, BS_GROUPBOX,
        },
    },
};

mod direct2d;
mod feature;

static REGISTER_WINDOW_CLASS: Once = Once::new();
static WINDOW_CLASS_NAME: &HSTRING = w!("bytetrail.window.bezier_demo_main");

pub(crate) struct MainWindow {
    handle: HWND,
    feature_wnd: Option<Box<FeatureWindow>>,
}

impl MainWindow {
    pub(crate) fn new(title: &'static str) -> windows::core::Result<Box<Self>> {
        let instance = unsafe { GetModuleHandleW(None)? };
        // synchronization for a one time initialization of FFI call
        REGISTER_WINDOW_CLASS.call_once(|| {
            // use defaults for all other fields
            let class = WNDCLASSW {
                lpfnWndProc: Some(Self::wnd_proc),
                hbrBackground: HBRUSH(COLOR_WINDOW.0 as isize),
                hInstance: instance,
                style: CS_HREDRAW | CS_VREDRAW,
                hCursor: unsafe { LoadCursorW(HINSTANCE(0), IDC_ARROW).ok().unwrap() },
                lpszClassName: WINDOW_CLASS_NAME.into(),
                ..Default::default()
            };
            assert_ne!(unsafe { RegisterClassW(&class) }, 0);
        });

        let mut main_window = Box::new(MainWindow {
            handle: HWND(0),
            feature_wnd: None,
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
                main_window.as_mut() as *mut _ as _,
            )
        };
        unsafe { ShowWindow(window, SW_SHOW) };
        Ok(main_window)
    }

    fn message_loop(
        &mut self,
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message {
            WM_CREATE => {
                let result = unsafe { GetModuleHandleW(None) };
                match result {
                    Ok(instance) => {
                        let feature_wnd = FeatureWindow::new(self.handle);
                        // TODO manage errors
                        self.feature_wnd = Some(feature_wnd.unwrap());

                        let hwnd = unsafe { 
                            CreateWindowExW(
                            WINDOW_EX_STYLE::default(),
                             &HSTRING::from("button"),
                              &HSTRING::from("Feature Type"), 
                        WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_GROUPBOX as u32),
                         10, 10, 125, 110, 
                         self.handle, 
                         HMENU(100),
                          instance,
                          std::ptr::null_mut())
                            };
                        MainWindow::create_selector(hwnd, instance, 20, "Road", 101);
                        MainWindow::create_selector(hwnd, instance, 45, "River", 102);
                        MainWindow::create_selector(hwnd, instance, 70, "Railroad", 103);

                        LRESULT(0)
                    }
                    Err(_e) => {
                        let err = unsafe { GetLastError() };
                        LRESULT(err.0 as isize)
                    }
                }
            }
            WM_SIZE => {
                let cx = lparam.0 & 0x0000_FFFF;
                let cy = (lparam.0 & 0xFFFF_0000) >> 16;
                unsafe {
                    MoveWindow(
                        self.feature_wnd.as_ref().unwrap().handle,
                        140,
                        0,
                        cx as i32,
                        cy as i32,
                        true,
                    )
                };

                LRESULT(0)
            }
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
        }
    }

    fn create_selector(parent: HWND, instance: HINSTANCE, y_off: i32, label: &str, id: isize) -> HWND{
        unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                &HSTRING::from("button"),
                &HSTRING::from(label),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_RADIOBUTTON as u32),
                10,
                y_off, 
                110,
                20,
                parent,
                HMENU(id),
                instance,
                std::ptr::null_mut(),
            )
        }

    }

    unsafe extern "system" fn wnd_proc(
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
        let this = GetWindowLongPtrA(window, GWLP_USERDATA) as *mut Self;

        if !this.is_null() {
            return (*this).message_loop(window, message, wparam, lparam);
        }

        DefWindowProcW(window, message, wparam, lparam)
    }
}
