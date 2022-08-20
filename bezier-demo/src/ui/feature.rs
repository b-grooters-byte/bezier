use std::sync::Once;

use windows::core::{Result, HSTRING};
use windows::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Direct2D::{
    ID2D1Factory1, ID2D1HwndRenderTarget, ID2D1SolidColorBrush, D2D1_HWND_RENDER_TARGET_PROPERTIES,
    D2D1_PRESENT_OPTIONS, D2D1_RENDER_TARGET_PROPERTIES,
};
use windows::Win32::Graphics::Gdi::CreateSolidBrush;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetClientRect, GetWindowLongPtrA, LoadCursorW, RegisterClassW,
    SetWindowLongPtrA, CREATESTRUCTA, CS_HREDRAW, CS_VREDRAW, GWLP_USERDATA, HICON, HMENU,
    IDC_ARROW, WINDOW_EX_STYLE, WM_CREATE, WNDCLASSW, WS_CHILDWINDOW, WS_TABSTOP, WS_VISIBLE,
};

use crate::feature::road::Road;

use super::direct2d::create_factory;

static REGISTER_FEATURE_WINDOW_CLASS: Once = Once::new();
static FEATURE_CLASS_NAME: &HSTRING = w!("bytetrail.window.feature_view");

trait FeatureVisual {
    fn draw(&mut self);
}

impl core::fmt::Debug for dyn FeatureVisual {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug)]
struct RoadVisual {
    road: Road,
    surface_brush: Option<ID2D1SolidColorBrush>,
    centerline_brush: Option<ID2D1SolidColorBrush>,
    edgeline_brush: Option<ID2D1SolidColorBrush>,
    selected: Option<usize>,
    hover: Option<usize>,
}

impl RoadVisual {
    fn new(road_width: f32, centerline: bool, edgeline: bool) -> Self {
        let road = Road::new();
        RoadVisual {
            road,
            surface_brush: None,
            centerline_brush: None,
            edgeline_brush: None,
            selected: None,
            hover: None,
        }
    }

    fn release_device_resources(&mut self) {
        self.surface_brush = None;
        self.centerline_brush = None;
        self.edgeline_brush = None;
    }
}

impl FeatureVisual for RoadVisual {
    fn draw(&mut self) {
        let surface = self.road.surface();
    }
}

#[derive(Debug)]
pub(crate) struct FeatureWindow {
    pub(crate) handle: HWND,
    feature: Option<Box<dyn FeatureVisual>>,
    factory: ID2D1Factory1,
    target: Option<ID2D1HwndRenderTarget>,
    dpi: f32,
}

impl FeatureWindow {
    pub(crate) fn new(instance: HINSTANCE, parent: HWND) -> Result<Box<Self>> {
        let factory = create_factory()?;

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

        let mut dpix = 0.0;
        let mut dpiy = 0.0;
        unsafe { factory.GetDesktopDpi(&mut dpix, &mut dpiy) };
        let mut view = Box::new(Self {
            handle: HWND(0),
            feature: None,
            factory,
            target: None,
            dpi: dpix,
        });

        unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                FEATURE_CLASS_NAME,
                None,
                WS_VISIBLE | WS_CHILDWINDOW | WS_TABSTOP,
                0,
                0,
                0,
                0,
                parent,
                HMENU(0),
                instance,
                view.as_mut() as *mut _ as _,
            );
        };
        Ok(view)
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
        // TODO release device resources from current visual
    }

    fn message_loop(
        &mut self,
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message {
            _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
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
        } else {
            let this = GetWindowLongPtrA(window, GWLP_USERDATA) as *mut Self;

            if !this.is_null() {
                return (*this).message_loop(window, message, wparam, lparam);
            }
        }

        DefWindowProcW(window, message, wparam, lparam)
    }
}
