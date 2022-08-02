use std::{
    ptr,
    sync::{Arc, Once},
};

use geometry::{bezier::Bezier, Point};
use windows::Win32::{System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::{WM_CREATE, CREATESTRUCTA}};
use windows::{
    core::{Result, HSTRING},
    w,
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::{
            Direct2D::{ID2D1DeviceContext, ID2D1Factory1, ID2D1SolidColorBrush, ID2D1StrokeStyle},
            Dxgi::IDXGISwapChain1,
            Gdi::{BeginPaint, CreateSolidBrush, EndPaint, ValidateRect, PAINTSTRUCT},
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, LoadCursorW, PostQuitMessage, RegisterClassW,
            ShowWindow, COLOR_WINDOW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, HMENU, IDC_ARROW,
            MK_LBUTTON, SW_SHOW, WINDOW_EX_STYLE, WM_DESTROY, WM_MOUSEMOVE, WM_PAINT, WNDCLASSW,
            WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        },
    },
};

use crate::direct2d::{create_device, create_factory, create_style};

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
        RenderState {
            bezier: Bezier::new_with_ctrl_point(
                [
                    Point { x: 20.0, y: 20.0 },
                    Point { x: 120.0, y: 20.0 },
                    Point { x: 320.0, y: 220.0 },
                    Point { x: 420.0, y: 22.0 },
                ],
                0.025,
            ),
            hover: None,
            selected: None,
        }
    }
}

pub(crate) struct Window {
    handle: HWND,
    factory: ID2D1Factory1,
    line_style: ID2D1StrokeStyle,
    target: Option<ID2D1DeviceContext>,
    swapchain: Option<IDXGISwapChain1>,
    line_brush: Option<ID2D1SolidColorBrush>,
    selected_brush: Option<ID2D1SolidColorBrush>,
    control_brush: Option<ID2D1SolidColorBrush>,
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
                hCursor: unsafe { LoadCursorW(HINSTANCE(0), IDC_ARROW).ok().unwrap() },
                lpszClassName: WINDOW_CLASS_NAME.into(),
                ..Default::default()
            };
            assert_ne!(unsafe { RegisterClassW(&class) }, 0);
        });

        let mut window_internal = Box::new(Self {
            handle: HWND(0),
            render_state: RenderState::new(),
            factory,
            line_style,
            line_brush: None,
            selected_brush: None,
            control_brush: None,
            swapchain: None,
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

    fn render(&mut self) -> Result<()> {
        // create the device specific resources
        if self.target == None {
            let device = create_device();
            println!("{:?}", device);
        }
        Ok(())
    }

    unsafe extern "system" fn wnd_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message as u32 {
            WM_CREATE => {
                let create_struct = lparam.0 as *const CREATESTRUCTA;
                let this = (*create_struct).lpCreateParams as *mut Self;
                (*this).handle = window;
                LRESULT(0)
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                BeginPaint(window, &mut ps);
                ValidateRect(window, ptr::null());
                EndPaint(window, &mut ps);
                LRESULT(0)
            }
            WM_MOUSEMOVE => {
                if wparam.0 == MK_LBUTTON as usize {}
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}
