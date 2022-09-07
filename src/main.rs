use libm::floorf;
use nannou::event::ElementState;
use nannou::prelude::*;

use pad::PadStr;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

use coreaudio::audio_unit::render_callback::{self, data};
use coreaudio::audio_unit::{AudioUnit, IOType, SampleFormat};
use std::f64::consts::PI;

struct SineWaveGenerator {
    time: f64,
    /// generated frequency in Hz
    freq: f64,
    /// magnitude of generated signal
    volume: f64,
}

impl SineWaveGenerator {
    fn new(freq: f64, volume: f64) -> Self {
        SineWaveGenerator {
            time: 0.,
            freq,
            volume,
        }
    }

    fn set_freq(&mut self, freq: f64) {
        self.freq = freq;
    }

    fn set_volume(&mut self, volume: f64) {
        self.volume = volume;
    }
}

impl Iterator for SineWaveGenerator {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        self.time += 1. / 48_000.;
        let output = ((self.freq * self.time * PI * 2.).sin() * self.volume) as f32;
        Some(output)
    }
}

fn main() {
    nannou::app(model).event(event).update(update).run();
}

struct Model {
    sender: Sender<f32>,
    _window: window::Id,
    allowed_segments: [(i32, i32); 16],
    id: u16,
}

fn start_audio(rx: Receiver<f32>) -> Result<(), coreaudio::Error> {
    let frequency_hz = 440.;
    let volume = 0.15;
    let mut samples = SineWaveGenerator::new(frequency_hz, volume);

    // Construct an Output audio unit that delivers audio to the default output device.
    let mut audio_unit = AudioUnit::new(IOType::DefaultOutput)?;

    // Read the input format. This is counterintuitive, but it's the format used when sending
    // audio data to the AudioUnit representing the output device. This is separate from the
    // format the AudioUnit later uses to send the data to the hardware device.
    let stream_format = audio_unit.input_stream_format()?;
    println!("{:#?}", &stream_format);

    // For this example, our sine wave expects `f32` data.
    assert!(SampleFormat::F32 == stream_format.sample_format);

        match msg {
            Ok(freq) => samples.set_volume(freq as f64),
            Err(_) => (),
        }

    type Args = render_callback::Args<data::NonInterleaved<f32>>;
    audio_unit.set_render_callback(move |args| {
        let Args {
            num_frames,
            mut data,
            ..
        } = args;
        let msg = rx.recv();
                
        for i in 0..num_frames {
            let sample = samples.next().unwrap();
            for channel in data.channels_mut() {
                channel[i] = sample;
            }
        }
        Ok(())
    })?;
    audio_unit.start()?;

    loop {
        thread::sleep(Duration::MAX);
    }

    // Ok(())
}

fn model(app: &App) -> Model {
    let (sender, receiver): (Sender<f32>, Receiver<f32>) = channel();

    thread::spawn(|| {
        let audio_result = start_audio(receiver);

        match audio_result {
            Ok(()) => (),
            Err(error) => panic!("Problem starting audio: {:?}", error),
        };
    });

    let _window = app.new_window().size(600, 600).view(view).build().unwrap();
    Model {
        sender,
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

fn event(app: &App, model: &mut Model, event: Event) {
    let sender = model.sender.clone();
    match event {
        Event::DeviceEvent(_id, event) => match event {
            nannou::winit::event::DeviceEvent::Button { button, state } => {
                if button == 0 && state == ElementState::Released {
                    model.id = random_range(0, 65536) as u16;
                }
            }
            nannou::winit::event::DeviceEvent::MouseMotion { delta: _ } => {
                let res = sender.send((app.mouse.y + 300.) / 600.);
                match res {
                    Ok(_) => (),
                    Err(_) => (),
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
