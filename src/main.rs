mod geometry;

use std::sync::{Mutex, Arc};

use geometry::Point;
use gtk::{prelude::*, DrawingArea};
use gtk::{Application, ApplicationWindow};

use crate::geometry::bezier::Bezier;

const HANDLE_RADIUS: f32 = 5.0;
const HANDLE_LINE_WIDTH: f64 = 1.0;

trait Draw {
    fn draw(&self, context: &cairo::Context);
    fn draw_mut(&mut self, context: &cairo::Context);
}




impl Draw for Bezier {
    fn draw(&self, _context: &cairo::Context) {
        unimplemented!();
    }

    fn draw_mut(&mut self, context: &cairo::Context) {
        let curve  = self.curve();
        context.set_source_rgb(0.0, 0.0, 0.0);
        context.move_to(curve[0].x as f64, curve[0].y as f64);
        for p in curve.iter().skip(1) {
            context.line_to(p.x as f64, p.y as f64);
        }
        context.stroke().expect("Unable to draw");
        context.set_source_rgb(0.0, 0.75, 0.0);
        context.set_line_width(HANDLE_LINE_WIDTH);
        for p in self.ctrl_points() {
            context.arc(p.x as f64, p.y as f64, HANDLE_RADIUS as f64, 0.0, 6.28);
            context.stroke().expect("unable to draw to context");
        }
        context.set_dash(&[2.0, 1.0], 0.0);
        let p = self.ctrl_points();
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

        let mut drag_start = Arc::new(Mutex::new(Point{x: 0.0, y: 0.0}));

        let mut b = Bezier::new(0.05);
        b.set_ctrl_point(Point{x: 50.0, y: 0.0}, 1);
        b.set_ctrl_point(Point{x: 100.0, y: 100.0}, 2);
        b.set_ctrl_point(Point{x: 150.0, y: 100.0}, 3);

        let mut selected: Arc<Mutex<Option<usize>>> = Arc::new(Mutex::new(None));
        let bezier = Arc::new(Mutex::new(b));

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
            let bezier_draw = bezier.clone();
            view.set_draw_func(move |_area,ctx, _width, _height| {
                let mut b = bezier_draw.lock().unwrap();
                b.draw_mut(ctx);
            });
        // connect the mouse button gesture
        let g = gtk::GestureClick::new();
        view.add_controller(&g);
        let pressed_selected = selected.clone();
        let bezier_pressed = bezier.clone();
        g.connect_pressed(move |p, i, x, y| {
            let b = bezier_pressed.lock().unwrap();
            for (i, p) in b.ctrl_points().iter().enumerate() {
                if p.distance(&Point{x: x as f32, y: y as f32}) < HANDLE_RADIUS {
                    if let Ok(mut s) = pressed_selected.lock() {
                        *s = Some(i);
                    }                    
                    break;
                }
            }
        });
        
        let d = gtk::GestureDrag::new();
        view.add_controller(&d);      
        window.set_child(Some(&view));
        let view_guard = Arc::new(Mutex::new(view));
        let view_drag_update = view_guard.clone();
        let bezier_drag = bezier.clone();
        let drag_selected = selected.clone();  
        let drag_initial = drag_start.clone();
        d.connect_drag_update(move |a,cx ,cy| {
            if let Ok(selected) = drag_selected.lock() {
                if let Some(selected) = *selected {
                    // move the control point                   
                    let mut b = bezier_drag.lock().unwrap();
                    let p = drag_initial.lock().unwrap();
                    b.set_ctrl_point(Point{ x: p.x + cx as f32, y: p.y + cy as f32}, selected);
                    view_drag_update.lock().unwrap().queue_draw();
                }
            }
        });
        // set the initial offset
        let drag_begin = drag_start.clone();
        d.connect_drag_begin(move | _g, x, y| {
            let mut p = drag_begin.lock().unwrap();
            p.x = x as f32;
            p.y = y as f32;
        });
        // repaint on drag end and set selected to none
        d.connect_drag_end(move | _g, x, y| {
            println!("[{:?}, {:?}]", x, y);
        });
        window.show();
    });

    app.run();
}
