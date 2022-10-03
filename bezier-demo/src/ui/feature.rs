use crate::feature::{road::Road, BezierFeature, BezierFeatureType};
use geometry::Point;

use std::sync::Once;
use windows::{
    core::HRESULT,
    Win32::{
        Foundation::{RECT, COLORREF},
        Graphics::{
            Direct2D::{
                Common::{D2D1_COLOR_F, D2D_POINT_2F, D2D_SIZE_U},
                ID2D1HwndRenderTarget, D2D1_ELLIPSE, D2D1_HWND_RENDER_TARGET_PROPERTIES,
                D2D1_PRESENT_OPTIONS, D2D1_RENDER_TARGET_PROPERTIES,
            },
            Gdi::{InvalidateRect},
        },
        System::{LibraryLoader::GetModuleHandleW, SystemServices::MK_LBUTTON},
        UI::WindowsAndMessaging::{
            GetClientRect, GetWindowLongPtrA, SetWindowLongPtrA, CREATESTRUCTA, GWLP_USERDATA,
            WM_CREATE, WM_LBUTTONDOWN, WM_LBUTTONUP, WS_CHILDWINDOW, WS_CLIPSIBLINGS, WS_HSCROLL,
            WS_VSCROLL,
        },
    },
};
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
            CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, HMENU, IDC_ARROW, 
            WINDOW_EX_STYLE, WM_DESTROY, WM_MOUSEMOVE, WM_PAINT, WM_SIZE, WNDCLASSW, WS_VISIBLE,
        },
    },
};

use crate::ui::direct2d::{create_brush, create_style};

use super::direct2d;

const RENDER_CTRL_HANDLE_RADIUS: f32 = 5.0;

static REGISTER_FEATURE_WINDOW_CLASS: Once = Once::new();
static FEATURE_WINDOW_CLASS_NAME: &HSTRING = w!("bytetrail.window.bezier-demo");

#[derive(Debug)]
pub(crate) struct RenderState<'a> {
    pub hover: Option<usize>,
    pub selected: Option<usize>,
    pub feature: BezierFeatureType,
    pub road_visual: Road<'a>,
}

impl<'a> RenderState<'a> {
    pub(crate) fn new(factory: &'a ID2D1Factory1) -> Self {
        let mut road = BezierFeature::new_with_attributes(30.0, false);
        road.set_ctrl_point(0, Point { x: 10.0, y: 10.0 });
        road.set_ctrl_point(1, Point { x: 100.0, y: 10.0 });
        road.set_ctrl_point(2, Point { x: 100.0, y: 200.0 });
        road.set_ctrl_point(3, Point { x: 200.0, y: 200.0 });
        road.add_segment(Point { x: 300.0, y: 300.0 }, Point { x: 300.0, y: 400.0 });

        let mut road_visual = Road::new(factory);
        road_visual.set_feature(road);

        RenderState {
            hover: None,
            selected: None,
            feature: BezierFeatureType::Road,
            road_visual,
        }
    }

    fn in_control_point(&self, x: f32, y: f32) -> Option<usize> {
        for (idx, ctrl) in self.road_visual.feature().unwrap().into_iter().enumerate() {
            if ctrl.dist_to_xy(x, y) <= RENDER_CTRL_HANDLE_RADIUS {
                return Some(idx);
            }
        }
        None
    }
}

pub(crate) struct FeatureWindow<'a> {
    pub(crate) handle: HWND,
    factory: &'a ID2D1Factory1,
    line_style: ID2D1StrokeStyle,
    ctrl_style: ID2D1StrokeStyle,
    target: Option<ID2D1HwndRenderTarget>,
    line_brush: Option<ID2D1SolidColorBrush>,
    selected_brush: Option<ID2D1SolidColorBrush>,
    control_brush: Option<ID2D1SolidColorBrush>,
    water_brush: Option<ID2D1SolidColorBrush>,
    render_state: RenderState<'a>,
    dpi: f32,
}

impl<'a> FeatureWindow<'a> {
    pub(crate) fn new(parent: HWND, factory: &'a ID2D1Factory1) -> Result<Box<Self>> {
        let line_style = create_style(&factory, None)?;
        let ctrl_style = create_style(&factory, Some(&[4.0, 2.0, 4.0, 2.0]))?;
        let instance = unsafe { GetModuleHandleW(None)? };
        // synchronization for a one time initialization of FFI call
        REGISTER_FEATURE_WINDOW_CLASS.call_once(|| {
            // use defaults for all other fields
            let class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wnd_proc),
                hInstance: instance,
                hCursor: unsafe { LoadCursorW(HINSTANCE(0), IDC_ARROW).ok().unwrap() },
                hbrBackground: unsafe { CreateSolidBrush(COLORREF(0))},
                lpszClassName: FEATURE_WINDOW_CLASS_NAME.into(),
                ..Default::default()
            };
            assert_ne!(unsafe { RegisterClassW(&class) }, 0);
        });

        let mut dpix = 0.0;
        let mut dpiy = 0.0;
        unsafe { factory.GetDesktopDpi(&mut dpix, &mut dpiy) };

        let render_state = RenderState::new(&factory);
        let mut window_internal = Box::new(Self {
            handle: HWND(0),
            render_state,
            factory,
            line_style,
            ctrl_style,
            line_brush: None,
            selected_brush: None,
            control_brush: None,
            water_brush: None,
            target: None,
            dpi: dpix,
        });

        // create the window using Self reference
        let _window = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                FEATURE_WINDOW_CLASS_NAME,
                &HSTRING::from(""),
                WS_VISIBLE | WS_CLIPSIBLINGS | WS_CHILDWINDOW | WS_VSCROLL | WS_HSCROLL,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                400,
                300,
                parent,
                HMENU(0),
                instance,
                Some(window_internal.as_mut() as *mut _ as _),
            )
        };
        //        unsafe { ShowWindow(window, SW_SHOW) };
        Ok(window_internal)
    }

    pub(crate) fn set_feature_type(&mut self, feature_type: BezierFeatureType) {
        self.render_state.feature = feature_type;
        unsafe {
            InvalidateRect(self.handle, None, false);
        }
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
        self.render_state.road_visual.release_resources();
        self.line_brush = None;
        self.control_brush = None;
        self.selected_brush = None;
        self.water_brush = None;
    }

    fn render(&mut self) -> Result<()> {
        // create the device specific resources
        if self.target == None {
            self.create_render_target()?;
            let target = self.target.as_ref().unwrap();
            unsafe { target.SetDpi(self.dpi, self.dpi) };
            self.render_state.road_visual.create_resources(target)?;
            self.control_brush = create_brush(target, 0.25, 0.25, 0.25, 1.0).ok();
            self.line_brush = create_brush(target, 0.0, 0.0, 0.0, 1.0).ok();
            self.selected_brush = create_brush(target, 0.75, 0.0, 0.0, 1.0).ok();
            self.water_brush = create_brush(target, 0.0, 0.65, 0.93, 1.0).ok();
            // self.create_path_geom();
        }
        // draw
        unsafe { self.target.as_ref().unwrap().BeginDraw() };
        self.draw()?;
        unsafe { self.target.as_ref().unwrap().EndDraw(None, None)? };
        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        let centerline = self.render_state.road_visual.feature_mut().unwrap().curve();
        let target = self.target.as_ref().unwrap();
        unsafe {
            target.Clear(Some(&D2D1_COLOR_F {
                r: 0.98,
                g: 0.98,
                b: 0.98,
                a: 1.0,
            }));
        }
        self.render_state.road_visual.draw(target);
        direct2d::draw_line(
            target,
            &centerline,
            self.control_brush.as_ref().unwrap(),
            &self.line_style,
            1.0,
        );
        let mut ellipse = D2D1_ELLIPSE {
            radiusX: RENDER_CTRL_HANDLE_RADIUS,
            radiusY: RENDER_CTRL_HANDLE_RADIUS,
            ..Default::default()
        };
        for (idx, ctrl) in self
            .render_state
            .road_visual
            .feature()
            .unwrap()
            .into_iter()
            .enumerate()
        {
            ellipse.point = D2D_POINT_2F {
                x: ctrl.x,
                y: ctrl.y,
            };
            if let Some(select_idx) = self.render_state.hover {
                if select_idx == idx {
                    unsafe {
                        target.FillEllipse(&ellipse, self.selected_brush.as_ref().unwrap());
                    }
                }
            }
            unsafe {
                target.DrawEllipse(
                    &ellipse,
                    self.control_brush.as_ref().unwrap(),
                    1.0,
                    &self.line_style,
                );
            }
            let ctrl_brush = self.control_brush.as_ref().unwrap();
            for segment in self.render_state.road_visual.feature().unwrap().segments() {
                let ctrl_points = segment.ctrl_points();
                unsafe {
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
                if wparam.0 == MK_LBUTTON.0 as usize {
                    if let Some(selected) = self.render_state.selected {
                        let current = self
                            .render_state
                            .road_visual
                            .feature()
                            .unwrap()
                            .ctrl_point(selected)
                            .unwrap();
                        self.render_state
                            .road_visual
                            .feature_mut()
                            .unwrap()
                            .set_ctrl_point(selected, Point { x, y });
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
                    let ctrl = &self
                        .render_state
                        .road_visual
                        .feature()
                        .unwrap()
                        .ctrl_point(idx)
                        .unwrap();
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
                        let ctrl = &self
                            .render_state
                            .road_visual
                            .feature()
                            .unwrap()
                            .ctrl_point(hover)
                            .unwrap();
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
    (
        (lparam.0 & 0x0000_FFFF) as f32,
        ((lparam.0 & 0xFFFF_0000) >> 16) as f32,
    )
}
