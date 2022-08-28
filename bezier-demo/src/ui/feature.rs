use std::sync::Once;

use geometry::bezier::Bezier;
use geometry::Point;
use windows::core::{Result, HRESULT, HSTRING};
use windows::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_COLOR_F, D2D1_FIGURE_BEGIN_FILLED, D2D1_FIGURE_END_CLOSED, D2D_POINT_2F, D2D_SIZE_U,
};
use windows::Win32::Graphics::Direct2D::{
    ID2D1Factory1, ID2D1HwndRenderTarget, ID2D1PathGeometry, ID2D1SolidColorBrush,
    ID2D1StrokeStyle, D2D1_ELLIPSE, D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_PRESENT_OPTIONS,
    D2D1_RENDER_TARGET_PROPERTIES,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, EndPaint, InvalidateRect, PAINTSTRUCT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetClientRect, GetWindowLongPtrA, LoadCursorW, RegisterClassW,
    SetWindowLongPtrA, ShowWindow, CREATESTRUCTA, CS_HREDRAW, CS_VREDRAW, GWLP_USERDATA, HICON,
    HMENU, IDC_ARROW, MK_LBUTTON, SW_SHOW, WINDOW_EX_STYLE, WM_CREATE, WM_DESTROY, WM_PAINT,
    WM_SIZE, WNDCLASSW, WS_CHILDWINDOW, WS_TABSTOP, WS_VISIBLE,
};

use crate::feature::road::Road;

use super::direct2d::{create_brush, create_factory, create_style};

static REGISTER_FEATURE_WINDOW_CLASS: Once = Once::new();
static FEATURE_CLASS_NAME: &HSTRING = w!("bytetrail.window.feature_view");

trait FeatureVisual {
    fn draw(&mut self, target: &ID2D1HwndRenderTarget) -> Result<()>;
}

impl core::fmt::Debug for dyn FeatureVisual {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug)]
struct RoadVisual {
    road: Road,
    surface: Option<ID2D1PathGeometry>,
    surface_brush: Option<ID2D1SolidColorBrush>,
    centerline_brush: Option<ID2D1SolidColorBrush>,
    edgeline_brush: Option<ID2D1SolidColorBrush>,
    selected: Option<usize>,
    hover: Option<usize>,
}

impl<'a> RoadVisual {
    fn new(road_width: f32, centerline: bool, edgeline: bool) -> Self {
        let road = Road::new_with_attributes(road_width, None, false);
        RoadVisual {
            road,
            surface: None,
            surface_brush: None,
            centerline_brush: None,
            edgeline_brush: None,
            selected: None,
            hover: None,
        }
    }

    fn create_path(&mut self, factory: &ID2D1Factory1) -> Result<()> {
        let path_geometry = unsafe { factory.CreatePathGeometry() }?;
        let sink = unsafe { path_geometry.Open() }?;

        let surface = self.road.surface();
        println!("CREATING PATH");
        unsafe {
            sink.BeginFigure((*surface[0]).into(), D2D1_FIGURE_BEGIN_FILLED);
            for point in surface.iter().skip(1) {
                sink.AddLine((**point).into());
            }
            sink.AddLine((*surface[0]).into());
            sink.EndFigure(D2D1_FIGURE_END_CLOSED);
            sink.Close()?;
        }
        self.surface = Some(path_geometry);
        Ok(())
    }

    fn set_surface_brush(&mut self, brush: Option<ID2D1SolidColorBrush>) {
        self.surface_brush = brush;
    }

    fn release_device_resources(&mut self) {
        self.surface_brush = None;
        self.centerline_brush = None;
        self.edgeline_brush = None;
    }
}

impl FeatureVisual for RoadVisual {
    fn draw(&mut self, target: &ID2D1HwndRenderTarget) -> Result<()> {
        println!("DRAW");
        unsafe { target.BeginDraw() };
        println!("BeginDraw()");
        unsafe {
            target.Clear(Some(&D2D1_COLOR_F {
                r: 0.9,
                g: 0.9,
                b: 0.9,
                a: 0.5,
            }));
        }
        if let Some(surface) = &self.surface {
            println!("have a surface geometry");
            let brush = self.surface_brush.as_ref().unwrap();
            unsafe { target.FillGeometry(surface, brush, None) };
        }
        unsafe { target.EndDraw(None, None)? };
        Ok(())
    }
}

const RENDER_CTRL_HANDLE_RADIUS: f32 = 5.0;

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
                return Some(idx);
            }
        }
        None
    }
}

pub(crate) struct FeatureWindow {
    pub(crate) handle: HWND,
    factory: ID2D1Factory1,
    line_style: ID2D1StrokeStyle,
    ctrl_style: ID2D1StrokeStyle,
    target: Option<ID2D1HwndRenderTarget>,
    line_brush: Option<ID2D1SolidColorBrush>,
    selected_brush: Option<ID2D1SolidColorBrush>,
    control_brush: Option<ID2D1SolidColorBrush>,
    render_state: RenderState,
    dpi: f32,
}

impl FeatureWindow {
    pub(crate) fn new(instance: HINSTANCE, parent: HWND) -> Result<Box<Self>> {
        let factory = create_factory()?;
        let line_style = create_style(&factory, None)?;
        let ctrl_style = create_style(&factory, Some(&[4.0, 2.0, 4.0, 2.0]))?;
        // synchronization for a one time initialization of FFI call
        REGISTER_FEATURE_WINDOW_CLASS.call_once(|| {
            // use defaults for all other fields
            let class = WNDCLASSW {
                lpfnWndProc: Some(Self::wnd_proc),
                hbrBackground: unsafe { CreateSolidBrush(0x0000FFFF) },
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
        unsafe { factory.GetDesktopDpi(&mut dpix, &mut dpiy) };

        let mut window_internal = Box::new(Self {
            handle: HWND(0),
            render_state: RenderState::new(),
            factory,
            line_style,
            ctrl_style,
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
                FEATURE_CLASS_NAME,
                &HSTRING::from("hello".to_string()),
                WS_VISIBLE | WS_CHILDWINDOW | WS_TABSTOP,
                0,
                0,
                0,
                0,
                parent,
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
            println!("CLIENT RECT: {:?}", rect);
            let mut props = D2D1_RENDER_TARGET_PROPERTIES::default();
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
            let mut ellipse = D2D1_ELLIPSE {
                radiusX: RENDER_CTRL_HANDLE_RADIUS,
                radiusY: RENDER_CTRL_HANDLE_RADIUS,
                ..Default::default()
            };
            for (idx, ctrl) in self.render_state.bezier.ctrl_points().iter().enumerate() {
                ellipse.point = D2D_POINT_2F {
                    x: ctrl.x,
                    y: ctrl.y,
                };
                if let Some(select_idx) = self.render_state.hover {
                    if select_idx == idx {
                        target.FillEllipse(&ellipse, self.selected_brush.as_ref().unwrap());
                    }
                }
                target.DrawEllipse(
                    &ellipse,
                    self.control_brush.as_ref().unwrap(),
                    1.0,
                    &self.line_style,
                );
            }
            // draw the control point lines
            let ctrl_points = self.render_state.bezier.ctrl_points();
            let ctrl_brush = self.control_brush.as_ref().unwrap();
            target.DrawLine(
                ctrl_points[0].into(),
                ctrl_points[1].into(),
                ctrl_brush,
                1.0,
                &self.ctrl_style,
            );
            target.DrawLine(
                ctrl_points[2].into(),
                ctrl_points[3].into(),
                ctrl_brush,
                1.0,
                &self.ctrl_style,
            );
        }
        Ok(())
    }

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_PAINT => {
                println!("WM_PAINT");
                let mut ps = PAINTSTRUCT::default();
                unsafe {
                    BeginPaint(self.handle, &mut ps);
                    self.render().expect("unable to render");
                    EndPaint(self.handle, &ps);
                }
                LRESULT(0)
            }
            WM_SIZE => unsafe {
                let mut hresult: HRESULT = HRESULT(0);
                if let Some(target) = self.target.as_ref() {
                    let mut rect: RECT = RECT::default();
                    GetClientRect(self.handle, &mut rect);
                    let result = target.Resize(&D2D_SIZE_U {
                        width: (rect.right - rect.left) as u32,
                        height: (rect.bottom - rect.top) as u32,
                    });
                    if let Err(e) = result {
                        hresult = e.code();
                    }
                }
                LRESULT(hresult.0 as isize)
            },
            WM_LBUTTONDOWN => {
                let (x, y) = mouse_position(lparam);
                if let Some(idx) = self.render_state.in_control_point(x, y) {
                    self.render_state.selected = Some(idx);
                }
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                // always deselct any selected control point
                self.render_state.selected = None;
                LRESULT(0)
            }
            WM_MOUSEMOVE => {
                let (x, y) = mouse_position(lparam);
                let idx = self.render_state.in_control_point(x, y);
                if wparam.0 == MK_LBUTTON as usize {
                    if let Some(selected) = self.render_state.selected {
                        let current = self.render_state.bezier.ctrl_point(selected);
                        self.render_state
                            .bezier
                            .set_ctrl_point(Point { x, y }, selected);
                        let top = (current.y.min(y) - RENDER_CTRL_HANDLE_RADIUS) as i32;
                        let bottom = (current.y.max(y) + RENDER_CTRL_HANDLE_RADIUS) as i32;
                        let left = (current.x.min(x) - RENDER_CTRL_HANDLE_RADIUS) as i32;
                        let right = (current.x.max(x) + RENDER_CTRL_HANDLE_RADIUS) as i32;
                        unsafe {
                            InvalidateRect(
                                self.handle,
                                Some(&RECT {
                                    left,
                                    top,
                                    right,
                                    bottom,
                                }),
                                false,
                            );
                        }
                    }
                }
                if let Some(idx) = idx {
                    let ctrl = &self.render_state.bezier.ctrl_points()[idx];
                    if self.render_state.hover.is_none() {
                        self.render_state.hover = Some(idx);
                        unsafe {
                            InvalidateRect(
                                self.handle,
                                Some(&RECT {
                                    left: (ctrl.x - 5.0) as i32,
                                    top: (ctrl.y - 5.0) as i32,
                                    right: (ctrl.x + 5.0) as i32,
                                    bottom: (ctrl.y + 5.0) as i32,
                                }),
                                false,
                            );
                        }
                    }
                } else {
                    // last state was hover
                    if let Some(hover) = self.render_state.hover {
                        let ctrl = &self.render_state.bezier.ctrl_points()[hover];
                        // left the active control point boundary
                        self.render_state.hover = None;
                        unsafe {
                            InvalidateRect(
                                self.handle,
                                Some(&RECT {
                                    left: (ctrl.x - 5.0) as i32,
                                    top: (ctrl.y - 5.0) as i32,
                                    right: (ctrl.x + 5.0) as i32,
                                    bottom: (ctrl.y + 5.0) as i32,
                                }),
                                false,
                            );
                        }
                    }
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                self.release_device();
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
    (
        (lparam.0 & 0x0000_FFFF) as f32,
        ((lparam.0 & 0xFFFF_0000) >> 16) as f32,
    )
}
