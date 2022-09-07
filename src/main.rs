use nannou::prelude::*;

use libm::floorf;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {}

fn model(app: &App) -> Model {
    app.new_window().size(600, 600).view(view).build().unwrap();
    Model {}
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn get_dot_coord(n: i32) -> Vec2 {
    return pt2(n as f32 % 3.0, floorf(n as f32 / 3.0));
}

fn draw_dot(coord: Vec2, rect: Rect, draw: &Draw) {
    let draw2 = draw.xy(rect.wh() * pt2(-0.5, 0.5));
    let xy = rect.xy() + coord * rect.wh() / pt2(2.0, -2.0);
    draw2.ellipse().w_h(10.0, 10.0).xy(xy);
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    // draw.background().color(PLUM);

    let win = app.window_rect();
    let win_p = win.pad(25.0);

    draw.rect().xy(win.xy()).wh(win.wh()).color(PLUM);

    // let start_point = win_p.xy();
    // let end_point = win_p.xy() + win_p.wh() / pt2(2.0,2.0);

    // draw.line()
    //     .start(start_point)
    //     .end(end_point)
    //     .weight(4.0)
    //     .color(STEELBLUE);

    for i in 0..9 {
        let coord = get_dot_coord(i);
        draw_dot(coord, win_p, &draw);
    }

    draw.to_frame(app, &frame).unwrap();
}
