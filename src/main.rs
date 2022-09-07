use libm::floorf;
use nannou::{event::ElementState, prelude::*};
use pad::PadStr;

fn main() {
    nannou::app(model).event(event).update(update).run();
}

struct Model {
    _window: window::Id,
    allowed_segments: [(i32, i32); 16],
    id: u16,
}

fn model(app: &App) -> Model {
    let _window = app.new_window().size(600, 600).view(view).build().unwrap();
    Model {
        _window,
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
        id: 0,
    }
}

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

fn event(_app: &App, model: &mut Model, event: Event) {
    match event {
        Event::DeviceEvent(_id, event) => match event {
            nannou::winit::event::DeviceEvent::Button { button, state } => {
                if button == 0 && state == ElementState::Released {
                    model.id = random_range(0, 65536) as u16;
                }
            }
            _ => (),
        },
        _ => (),
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    let win = app.window_rect();
    let win_p = win.pad(25.0);

    draw.rect().xy(win.xy()).wh(win.wh()).color(PLUM);

    let id: u16 = model.id;
    let id_b: String = format!("{id:b}").pad_to_width_with_char(16, '0');

    for (i, b) in id_b.chars().enumerate() {
        if b == '1' {
            let (a, b) = get_segment_xys(get_segment_coords(model.allowed_segments[i]), win_p);
            draw.line().start(a).end(b).weight(4.0).color(STEELBLUE);
        }
    }

    for i in 0..9 {
        let xy = get_dot_xy(get_dot_coord(i), win_p);
        draw.ellipse().w_h(10.0, 10.0).xy(xy);
    }

    draw.to_frame(app, &frame).unwrap();
}
