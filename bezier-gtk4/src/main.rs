use std::cell::RefCell;
use std::rc::Rc;

use geometry::Point;
use gtk::gdk::ModifierType;
use gtk::{prelude::*, DrawingArea};
use gtk::{Application, ApplicationWindow};

use geometry::bezier::Bezier;

const HANDLE_RADIUS: f32 = 5.0;
const HANDLE_LINE_WIDTH: f64 = 1.0;
const HANDLE_GRAY: f64 = 0.25;
const HANDLE_SELECT_RED: f64 = 0.8;

trait Draw {
    fn draw(&self, context: &cairo::Context);
    fn draw_mut(&mut self, context: &cairo::Context);
}

#[derive(Debug, Clone)]
struct BezierRender {
    bezier: Bezier,
    selected_ctrl_pt: Option<usize>,
}

impl Draw for BezierRender {
    fn draw(&self, _context: &cairo::Context) {
        unimplemented!();
    }

    fn draw_mut(&mut self, context: &cairo::Context) {
        let curve = self.bezier.curve();
        context.set_source_rgb(0.0, 0.0, 0.0);
        context.move_to(curve[0].x as f64, curve[0].y as f64);
        for p in curve.iter().skip(1) {
            context.line_to(p.x as f64, p.y as f64);
        }
        context.stroke().expect("Unable to draw");
        context.set_line_width(HANDLE_LINE_WIDTH);
        context.set_source_rgb(HANDLE_GRAY, HANDLE_GRAY, HANDLE_GRAY);
        for (i, p) in self.bezier.ctrl_points().iter().enumerate() {
            if let Some(s) = self.selected_ctrl_pt {
                if s == i {
                    context.set_source_rgb(HANDLE_SELECT_RED, 0.0, 0.0);
                    context.arc(
                        p.x as f64,
                        p.y as f64,
                        HANDLE_RADIUS as f64,
                        0.0,
                        std::f64::consts::TAU,
                    );
                    context.fill().expect("unable to draw to context");
                    context.set_source_rgb(HANDLE_GRAY, HANDLE_GRAY, HANDLE_GRAY);
                }
            }
            context.arc(
                p.x as f64,
                p.y as f64,
                HANDLE_RADIUS as f64,
                0.0,
                std::f64::consts::TAU,
            );
            context.stroke().expect("unable to draw to context");
        }
        context.set_dash(&[2.0, 1.0], 0.0);
        let p = self.bezier.ctrl_points();
        context.move_to(p[0].x as f64, p[0].y as f64);
        context.line_to(p[1].x as f64, p[1].y as f64);
        context.move_to(p[2].x as f64, p[2].y as f64);
        context.line_to(p[3].x as f64, p[3].y as f64);
        context.stroke().expect("unable to draw to context");
    }
}

fn main() {
    let app = Application::builder()
        .application_id("org.bytetrail.Bezier")
        .build();

    app.connect_activate(|app| {
        let mut b = Bezier::new(0.025);
        b.set_ctrl_point(Point { x: 50.0, y: 0.0 }, 1);
        b.set_ctrl_point(Point { x: 100.0, y: 100.0 }, 2);
        b.set_ctrl_point(Point { x: 150.0, y: 100.0 }, 3);
        let render_context = BezierRender {
            bezier: b,
            selected_ctrl_pt: None,
        };
        let context = Rc::new(RefCell::new(render_context));

        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(600)
            .title("Bézier Curves")
            .build();

        let view = DrawingArea::builder()
            .can_focus(true)
            .hexpand(true)
            .vexpand(true)
            .build();
        let bezier_draw = context.clone();
        view.set_draw_func(move |_area, ctx, _width, _height| {
            let mut b = bezier_draw.borrow_mut();
            b.draw_mut(ctx);
        });
        // connect the mouse button gesture
        let g = gtk::GestureClick::new();
        view.add_controller(&g);
        let bezier_pressed = context.clone();
        g.connect_pressed(move |_g, _i, x, y| {
            let mut b = bezier_pressed.borrow_mut();
            for (i, p) in b.bezier.ctrl_points().iter().enumerate() {
                if p.distance(&Point {
                    x: x as f32,
                    y: y as f32,
                }) < HANDLE_RADIUS
                {
                    b.selected_ctrl_pt = Some(i);
                    break;
                }
            }
        });
        window.set_child(Some(&view));

        let m = gtk::EventControllerMotion::new();
        view.add_controller(&m);
        let view_guard = RefCell::new(view);
        let view_drag_update = view_guard.clone();
        let bezier_drag = context.clone();

        m.connect_motion(move |controller, x, y| {
            let event = controller.current_event().unwrap();
            let modifiers = event.modifier_state();
            let mut render_context = bezier_drag.borrow_mut();
            if modifiers.bits() & ModifierType::BUTTON1_MASK.bits() != 0 {
                if let Some(selected) = render_context.selected_ctrl_pt {
                    // move the control point
                    render_context.bezier.set_ctrl_point(
                        Point {
                            x: x as f32,
                            y: y as f32,
                        },
                        selected,
                    );
                    view_drag_update.borrow_mut().queue_draw();
                }
            } else {
                let prev_selected: Option<usize> = render_context.selected_ctrl_pt;
                render_context.selected_ctrl_pt = None;
                for (i, p) in render_context.bezier.ctrl_points().iter().enumerate() {
                    if p.distance(&Point {
                        x: x as f32,
                        y: y as f32,
                    }) < HANDLE_RADIUS
                    {
                        render_context.selected_ctrl_pt = Some(i);
                        break;
                    }
                }
                if prev_selected != render_context.selected_ctrl_pt {
                    view_drag_update.borrow_mut().queue_draw();
                }
            }
        });
        window.show();
    });

    app.run();
}
