use std::{sync::{Once, Arc}, ptr};

use geometry::{bezier::Bezier, Point};
use windows::{
    Win32::{
        Foundation::{
            HINSTANCE,
            HWND, WPARAM, LPARAM, LRESULT,
        },
        UI::WindowsAndMessaging::{
            WNDCLASSW, LoadCursorW, IDC_ARROW, DefWindowProcW, RegisterClassW, CreateWindowExW, CW_USEDEFAULT, 
            HMENU, WS_OVERLAPPEDWINDOW, ShowWindow, SW_SHOW, WINDOW_EX_STYLE, CS_HREDRAW, CS_VREDRAW, PostQuitMessage, WS_VISIBLE, WM_PAINT, WM_DESTROY, COLOR_WINDOW, WM_MOUSEMOVE, MK_LBUTTON,
        }, Graphics::{Gdi::{ValidateRect, CreateSolidBrush, EndPaint, BeginPaint, PAINTSTRUCT}, Direct2D::{ID2D1Factory1, ID2D1StrokeStyle, ID2D1DeviceContext}}, 
    }, w, core::{Result, HSTRING}
};
use windows::{
    Win32::System::LibraryLoader::GetModuleHandleW,
};

use crate::direct2d::{create_factory, create_style};

static REGISTER_WINDOW_CLASS: Once = Once::new();
static WINDOW_CLASS_NAME: &HSTRING = w!("bytetrail.window.bezier");
static LINE_STYLE_HANDLE: [f32; 2] = [2.0, 1.0];

#[derive(Debug, Clone)]
pub struct RenderState {
    pub bezier: Bezier,
    pub hover: Option<usize>,
    pub selected: Option<usize>,
}

impl RenderState {
    pub(crate) fn new() -> Self {
        RenderState { bezier: Bezier::new_with_ctrl_point([
            Point{ x: 20.0, y: 20.0 },
            Point{x: 120.0, y: 20.0 },
            Point{ x: 320.0, y:220.0 },
            Point{ x: 420.0, y:22.0}
        ], 0.025), hover: None, selected:None }
    }
}


pub(crate) struct Window {
    handle: HWND,
    factory: ID2D1Factory1,
    line_style: ID2D1StrokeStyle,
    target: Option<ID2D1DeviceContext>,
    render_state: RenderState,
}

impl Window {
    pub(crate) fn new(title: &str) -> Result<Box<Self>> {
        let factory = create_factory()?;
        let line_style = create_style(&factory, &[])?;
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
            render_state: RenderState::new(),
            factory,
            line_style,
            target: None,
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
                let mut ps = PAINTSTRUCT::default();
                BeginPaint(window, &mut ps);
                ValidateRect(window, ptr::null());
                EndPaint(window, &mut ps);
                LRESULT(0)

            }
            WM_MOUSEMOVE => {
                if wparam.0 == MK_LBUTTON as usize {
                
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ =>  DefWindowProcW(window, message, wparam, lparam),
        }
    }
}