use std::sync::Once;

use geometry::{bezier::Bezier, Point};
use windows::{Win32::{
    Foundation::RECT,
    Graphics::{Direct2D::{
        Common::{D2D1_COLOR_F, D2D_POINT_2F, D2D_SIZE_U},
        ID2D1HwndRenderTarget, D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_PRESENT_OPTIONS,
        D2D1_RENDER_TARGET_PROPERTIES, D2D1_ELLIPSE,
    }, Gdi::InvalidateRect},
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        GetClientRect, GetWindowLongPtrA, SetWindowLongPtrA, CREATESTRUCTA, GWLP_USERDATA,
        WM_CREATE, WM_LBUTTONDOWN, WM_LBUTTONUP,
    },
}, core::HRESULT};
use windows::{
    core::{Result, HSTRING},
    w,
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::{
            Direct2D::{ID2D1Factory1, ID2D1SolidColorBrush, ID2D1StrokeStyle},
            Gdi::{BeginPaint, CreateSolidBrush, EndPaint, PAINTSTRUCT},
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, LoadCursorW, PostQuitMessage, RegisterClassW,
            ShowWindow, COLOR_WINDOW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, HMENU, IDC_ARROW,
            MK_LBUTTON, SW_SHOW, WINDOW_EX_STYLE, WM_DESTROY, WM_MOUSEMOVE, WM_PAINT, WM_SIZE, WNDCLASSW,
            WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        },
    },
};

use crate::direct2d::{create_brush, create_factory, create_style};

const RENDER_CTRL_HANDLE_RADIUS: f32 = 5.0;

static REGISTER_WINDOW_CLASS: Once = Once::new();
static WINDOW_CLASS_NAME: &HSTRING = w!("bytetrail.window.bezier");

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
                    Point { x: 220.0, y: 220.0 },
                    Point { x: 320.0, y: 20.0 },
                ],
                0.025,
            ),
            hover: None,
            selected: None,
        }
    }

    fn in_control_point(&self, x: f32, y: f32) -> Option<usize> {
        for (idx, ctrl) in self.bezier.ctrl_points().iter().enumerate() {
            if ctrl.dist_to_xy(x, y) <= RENDER_CTRL_HANDLE_RADIUS {
                return Some(idx)
            }
        }
        None
    }
}

pub(crate) struct Window {
    handle: HWND,
    factory: ID2D1Factory1,
    line_style: ID2D1StrokeStyle,
    target: Option<ID2D1HwndRenderTarget>,
    line_brush: Option<ID2D1SolidColorBrush>,
    selected_brush: Option<ID2D1SolidColorBrush>,
    control_brush: Option<ID2D1SolidColorBrush>,
    render_state: RenderState,
    dpi: f32,
}

impl Window {
    pub(crate) fn new(title: &str) -> Result<Box<Self>> {
        let factory = create_factory()?;
        let line_style = create_style(&factory, None)?;
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

    pub(crate) fn create_render_target(&mut self) -> Result<()> {
        unsafe {
            let mut rect: RECT = RECT::default();
            GetClientRect(self.handle, &mut rect);
            let props = D2D1_RENDER_TARGET_PROPERTIES::default();
            let hwnd_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd: self.handle,
                pixelSize: windows::Win32::Graphics::Direct2D::Common::D2D_SIZE_U {
                    width: (rect.right - rect.left) as u32,
                    height: (rect.bottom - rect.top) as u32,
                },
                presentOptions: D2D1_PRESENT_OPTIONS::default(),
            };
            let target = self.factory.CreateHwndRenderTarget(&props, &hwnd_props)?;
            self.target = Some(target);
        }
        Ok(())
    }
    fn release_device(&mut self) {
        self.target = None;
        self.release_device_resources();
    }

    fn release_device_resources(&mut self) {
        self.line_brush = None;
        self.control_brush = None;
        self.selected_brush = None;
    }

    fn render(&mut self) -> Result<()> {
        // create the device specific resources
        if self.target == None {
            //let device = create_device()?;
            //let target = create_render_target(&self.factory, &device)?;
            self.create_render_target()?;
            let target = self.target.as_ref().unwrap();
            unsafe { target.SetDpi(self.dpi, self.dpi) };
            self.control_brush = create_brush(target, 0.25, 0.25, 0.25, 1.0).ok();
            self.line_brush = create_brush(target, 0.0, 0.0, 0.0, 1.0).ok();
            self.selected_brush = create_brush(target, 0.75, 0.0, 0.0, 1.0).ok();
        }
        // draw
        unsafe { self.target.as_ref().unwrap().BeginDraw() };
        self.draw()?;
        unsafe { self.target.as_ref().unwrap().EndDraw(None, None)? };
        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        let target = self.target.as_ref().unwrap();
        let curve = self.render_state.bezier.curve();
        unsafe {
            target.Clear(Some(&D2D1_COLOR_F {
                r: 0.9,
                g: 0.9,
                b: 0.9,
                a: 0.5,
            }));
            // draw the curve
            let mut p1 = &curve[0];
            for p2 in curve.iter().skip(1) {
                target.DrawLine(
                    D2D_POINT_2F { x: p1.x, y: p1.y },
                    D2D_POINT_2F { x: p2.x, y: p2.y },
                    self.control_brush.as_ref().unwrap(),
                    1.0,
                    &self.line_style,
                );
                p1 = p2;
            }
            // draw the control handles
            let mut ellipse = D2D1_ELLIPSE{
                radiusX: RENDER_CTRL_HANDLE_RADIUS,
                radiusY: RENDER_CTRL_HANDLE_RADIUS,
                ..Default::default()
            };

            for (idx, ctrl) in self.render_state.bezier.ctrl_points().iter().enumerate() {
                ellipse.point = D2D_POINT_2F{x: ctrl.x, y: ctrl.y};
                if let Some(select_idx) = self.render_state.hover {
                    if select_idx == idx {
                        target.FillEllipse(&ellipse, self.selected_brush.as_ref().unwrap());    
                    }
                }
                target.DrawEllipse(&ellipse, 
                    self.control_brush.as_ref().unwrap(),
                    1.0, 
                    &self.line_style);
            }
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
                    EndPaint(self.handle, &ps);
                }
                LRESULT(0)
            }
            WM_SIZE => {
                unsafe {
                    let mut hresult: HRESULT = HRESULT(0);
                    if let Some(target) = self.target.as_ref() {
                        let mut rect: RECT = RECT::default();
                        GetClientRect(self.handle, &mut rect);
                        let result = target.Resize(&D2D_SIZE_U {
                            width: (rect.right - rect.left) as u32,
                            height: (rect.bottom - rect.top) as u32
                        });
                        if let Err(e) = result {
                            hresult = e.code();
                        }
                    }
                    LRESULT(hresult.0 as isize)
                }
            }
            WM_LBUTTONDOWN => {
                let (x,y) = mouse_position(lparam);
                if let Some(idx) = self.render_state.in_control_point(x, y) {
                    self.render_state.selected = Some(idx);
                }
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                self.render_state.selected = None;
                LRESULT(0)
            }
            WM_MOUSEMOVE => {
                let (x,y) = mouse_position(lparam);
                self.render_state.selected = None;
                for (idx, ctrl) in self.render_state.bezier.ctrl_points().iter().enumerate() {
                    // last state was hover
                    if let Some(hover) = self.render_state.hover {
                        if idx == hover && ctrl.dist_to_xy(x, y) > RENDER_CTRL_HANDLE_RADIUS {
                            // left the active control point boundary
                            self.render_state.hover = None;
                            unsafe {
                            InvalidateRect(self.handle, 
                                Some(&RECT{
                                    left: (ctrl.x -5.0) as i32, 
                                    top: (ctrl.y-5.0) as i32, 
                                    right: (ctrl.x + 5.0) as i32, 
                                    bottom: (ctrl.y+5.0) as i32}), 
                                false);
                            }
                        }
                    }
                    if ctrl.dist_to_xy(x, y) <= RENDER_CTRL_HANDLE_RADIUS {
                        self.render_state.hover = Some(idx);
                        unsafe { 
                            InvalidateRect(self.handle, 
                                Some(&RECT{
                                    left: (ctrl.x -5.0) as i32, 
                                    top: (ctrl.y-5.0) as i32, 
                                    right: (ctrl.x + 5.0) as i32, 
                                    bottom: (ctrl.y+5.0) as i32}), 
                                true);
                        }
                        break;
                    }
                }
                if wparam.0 == MK_LBUTTON as usize {
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                self.release_device();
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


fn mouse_position(lparam: LPARAM) -> (f32, f32) {
    ((lparam.0  & 0x0000_FFFF) as f32,
    ((lparam.0  & 0xFFFF_0000) >> 16) as f32)
}