use nannou::prelude::*;

use libm::floorf;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    allowed_segments: [(i32, i32); 16],
}

fn model(app: &App) -> Model {
    app.new_window().size(600, 600).view(view).build().unwrap();
    Model {
        allowed_segments: [
            (0, 1),
            (1, 2),
            (0, 3),
            (0, 4),
            (1, 4),
            (4, 2),
            (2, 5),
            (3, 4),
            (4, 5),
            (3, 6),
            (6, 4),
            (4, 7),
            (4, 8),
            (5, 8),
            (6, 7),
            (7, 8),
        ],
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn get_dot_coord(n: i32) -> Vec2 {
    pt2(n as f32 % 3.0, floorf(n as f32 / 3.0))
}

fn get_dot_xy(coord: Vec2, rect: Rect) -> Vec2 {
    rect.xy() + (coord - pt2(1.0, 1.0)) * rect.wh() / pt2(2.0, -2.0)
}

fn get_segment_coords((a, b): (i32, i32)) -> (Vec2, Vec2) {
    (get_dot_coord(a), get_dot_coord(b))
}

fn get_segment_xys((a, b): (Vec2, Vec2), rect: Rect) -> (Vec2, Vec2) {
    (get_dot_xy(a, rect), get_dot_xy(b, rect))
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    // draw.background().color(PLUM);

    let win = app.window_rect();
    let win_p = win.pad(25.0);

    draw.rect().xy(win.xy()).wh(win.wh()).color(PLUM);

    // let start_point = win_p.xy();
    // let end_point = win_p.xy() + win_p.wh() / pt2(2.0,2.0);

    for i in 0..9 {
        let xy = get_dot_xy(get_dot_coord(i), win_p);
        draw.ellipse().w_h(10.0, 10.0).xy(xy);
    }

    for segment_coords in model.allowed_segments {
        let (a, b) = get_segment_xys(get_segment_coords(segment_coords), win_p);
        draw.line().start(a).end(b).weight(4.0).color(STEELBLUE);
    }

    draw.to_frame(app, &frame).unwrap();
}
