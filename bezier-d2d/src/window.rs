use std::{
    sync::Once, ptr,
};

use geometry::{bezier::Bezier, Point};
use windows::Win32::{
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        GetWindowLongPtrA, SetWindowLongPtrA, CREATESTRUCTA, GWLP_USERDATA, WM_CREATE,
    }, Graphics::Direct2D::Common::D2D1_COLOR_F,
};
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

use crate::direct2d::{create_device, create_factory, create_render_target, create_style, create_swapchain, create_swapchain_bitmap, create_brush};

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
    dpi: f32,
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

        let mut dpix = 0.0;
        let mut dpiy = 0.0;
        unsafe { factory.GetDesktopDpi(&mut dpix, &mut dpiy) };

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
            dpi: dpix,
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
            let device = create_device()?;
            let target = create_render_target(&self.factory, &device)?;
            unsafe { target.SetDpi(self.dpi, self.dpi) };

            // setup the swap chain
            let swapchain = create_swapchain(&device, self.handle)?;
            create_swapchain_bitmap(&swapchain, &target)?;
            self.control_brush = create_brush(&target, 0.2, 0.2, 0.2, 1.0).ok();
            self.line_brush = create_brush(&target, 0.0, 0.0, 0.0, 1.0).ok();
            self.target = Some(target);
            self.swapchain = Some(swapchain);
        }
        // draw 
        //let target = self.target.as_ref().unwrap();
        unsafe { self.target.as_ref().unwrap().BeginDraw() };
        self.draw()?;
        unsafe {
            let null: *mut u64 = ptr::null_mut(); 
            self.target.as_ref().unwrap().EndDraw(null, null)?;
        }

        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        let target = self.target.as_ref().unwrap();
        unsafe {
            target.Clear(&D2D1_COLOR_F{ r: 0.9, g: 0.9, b: 0.9, a: 0.5});
        }
        Ok(())
    }

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                unsafe {
                    BeginPaint(self.handle, &mut ps);
                    self.render().expect("unable to render");
                    EndPaint(self.handle, &mut ps);
                }
                LRESULT(0)
            }
            WM_MOUSEMOVE => {
                if wparam.0 == MK_LBUTTON as usize {}
                LRESULT(0)
            }
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(self.handle, message, wparam, lparam) },
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
        } else {
            let this = GetWindowLongPtrA(window, GWLP_USERDATA) as *mut Self;

            if !this.is_null() {
                return (*this).message_handler(message, wparam, lparam);
            }
        }
        DefWindowProcW(window, message, wparam, lparam)
    }
}
