pub mod direct2d;
mod feature;

use std::sync::Once;
use windows::{
    core::HSTRING,
    w,
    Win32::{
        Foundation::{GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::{Direct2D::ID2D1Factory1, Gdi::HBRUSH},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, GetWindowLongPtrA, LoadCursorW, MoveWindow,
            PostQuitMessage, RegisterClassW, SendMessageW, SetWindowLongPtrA, ShowWindow,
            BM_SETCHECK, BS_GROUPBOX, BS_RADIOBUTTON, COLOR_WINDOW, CREATESTRUCTA, CS_HREDRAW,
            CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, HMENU, IDC_ARROW, SW_SHOW, WINDOW_EX_STYLE,
            WINDOW_STYLE, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_SIZE, WNDCLASSW, WS_CHILD,
            WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        },
    },
};

use crate::feature::BezierFeatureType;

use self::feature::FeatureWindow;

const IDC_BUTTON_ROAD: i32 = 101;
const IDC_BUTTON_RIVER: i32 = 102;
const IDC_BUTTON_RAILROAD: i32 = 103;

static REGISTER_WINDOW_CLASS: Once = Once::new();
static WINDOW_CLASS_NAME: &HSTRING = w!("bytetrail.window.bezier_demo_main");

pub(crate) struct MainWindow<'a> {
    handle: HWND,
    feature_wnd: Option<Box<FeatureWindow<'a>>>,
    road_rb: Option<HWND>,
    river_rb: Option<HWND>,
    railroad_rb: Option<HWND>,
    factory: &'a ID2D1Factory1,
}

impl<'a> MainWindow<'a> {
    pub(crate) fn new(
        title: &'static str,
        factory: &'a ID2D1Factory1,
    ) -> windows::core::Result<Box<Self>> {
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
            road_rb: None,
            river_rb: None,
            railroad_rb: None,
            factory,
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
            WM_COMMAND => {
                let ctrl_id = (wparam.0 & 0x0000_FFFF) as i32;
                if (IDC_BUTTON_ROAD..=IDC_BUTTON_RAILROAD).contains(&ctrl_id) {
                    self.set_feature(ctrl_id);
                }
                LRESULT(0)
            }
            WM_CREATE => {
                let result = unsafe { GetModuleHandleW(None) };
                match result {
                    Ok(instance) => {
                        let feature_wnd = FeatureWindow::new(self.handle, self.factory);
                        // TODO manage errors
                        self.feature_wnd = Some(feature_wnd.unwrap());

                        let _hwnd = unsafe {
                            CreateWindowExW(
                                WINDOW_EX_STYLE::default(),
                                &HSTRING::from("button"),
                                &HSTRING::from("Feature Type"),
                                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_GROUPBOX as u32),
                                8,
                                10,
                                125,
                                110,
                                self.handle,
                                HMENU(100),
                                instance,
                                std::ptr::null_mut(),
                            )
                        };
                        self.road_rb = Some(MainWindow::create_selector(
                            self.handle,
                            instance,
                            30,
                            "Road",
                            101,
                        ));
                        self.river_rb = Some(MainWindow::create_selector(
                            self.handle,
                            instance,
                            55,
                            "River",
                            102,
                        ));
                        self.railroad_rb = Some(MainWindow::create_selector(
                            self.handle,
                            instance,
                            80,
                            "Railroad",
                            103,
                        ));

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

    fn set_feature(&mut self, control_id: i32) {
        let feature_wnd = self.feature_wnd.as_mut().unwrap().as_mut();
        match control_id {
            IDC_BUTTON_ROAD => unsafe {
                SendMessageW(self.road_rb, BM_SETCHECK, WPARAM(1), LPARAM(0));
                SendMessageW(self.river_rb, BM_SETCHECK, WPARAM(0), LPARAM(0));
                SendMessageW(self.railroad_rb, BM_SETCHECK, WPARAM(0), LPARAM(0));

                feature_wnd.set_feature_type(BezierFeatureType::Road);
            },
            IDC_BUTTON_RIVER => unsafe {
                SendMessageW(self.road_rb, BM_SETCHECK, WPARAM(0), LPARAM(0));
                SendMessageW(self.river_rb, BM_SETCHECK, WPARAM(1), LPARAM(0));
                SendMessageW(self.railroad_rb, BM_SETCHECK, WPARAM(0), LPARAM(0));

                feature_wnd.set_feature_type(BezierFeatureType::River);
            },
            IDC_BUTTON_RAILROAD => unsafe {
                SendMessageW(self.road_rb, BM_SETCHECK, WPARAM(0), LPARAM(0));
                SendMessageW(self.river_rb, BM_SETCHECK, WPARAM(0), LPARAM(0));
                SendMessageW(self.railroad_rb, BM_SETCHECK, WPARAM(1), LPARAM(0));

                feature_wnd.set_feature_type(BezierFeatureType::Railroad);
            },
            _ => (),
        }
    }

    fn create_selector(
        parent: HWND,
        instance: HINSTANCE,
        y_off: i32,
        label: &str,
        id: isize,
    ) -> HWND {
        unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                &HSTRING::from("button"),
                &HSTRING::from(label),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_RADIOBUTTON as u32),
                16,
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
