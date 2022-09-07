use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::Duration,
};

use libm::floorf;
use nannou::event::ElementState;
use nannou::prelude::*;

use pad::PadStr;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn start_audio() -> Sender<f32> {
    let (tx, rx): (Sender<f32>, Receiver<f32>) = channel();
    thread::spawn(|| -> anyhow::Result<()> {
        let stream = stream_setup_for(sample_next, rx)?;
        stream.play()?;
        thread::sleep(Duration::MAX);
        Ok(())
    });
    tx
}

fn sample_next(o: &mut SampleRequestOptions) -> f32 {
    let mut res = o.freq;
    loop {
        match o.rx.try_recv() {
            Ok(v) => res = v,
            Err(_) => break,
        };
    }
    o.freq = res;

    o.tick()
}

pub struct SampleRequestOptions {
    pub sample_rate: f32,
    pub sample_clock: f32,
    pub nchannels: usize,
    pub rx: Receiver<f32>,
    pub freq: f32,
    pub phase: f32,
}

impl SampleRequestOptions {
    fn tick(&mut self) -> f32 {
        self.phase += self.freq * 1. / self.sample_rate;

        if self.phase >= 0.5 {
            self.phase -= 1.
        }

        (self.phase * 2.0 * std::f32::consts::PI).sin() * 0.1
    }

    pub fn set_freq(&mut self, last_freq: f32) {
        self.freq = last_freq;
    }

    pub fn freq(&self) -> f32 {
        self.freq
    }

    pub fn phase(&self) -> f32 {
        self.phase
    }

    pub fn set_phase(&mut self, phase: f32) {
        self.phase = phase;
    }
}

pub fn stream_setup_for<F>(on_sample: F, rx: Receiver<f32>) -> Result<cpal::Stream, anyhow::Error>
where
    F: FnMut(&mut SampleRequestOptions) -> f32 + std::marker::Send + 'static + Copy,
{
    let (_host, device, config) = host_device_setup()?;

    let sample_rate = config.sample_rate().0 as f32;
    let sample_clock = 0f32;
    let nchannels = config.channels() as usize;
    let request = SampleRequestOptions {
        rx,
        sample_rate,
        sample_clock,
        nchannels,
        freq: 440.,
        phase: 0.,
    };

    match config.sample_format() {
        cpal::SampleFormat::F32 => {
            stream_make::<f32, _>(&device, &config.into(), on_sample, request)
        }
        cpal::SampleFormat::I16 => {
            stream_make::<i16, _>(&device, &config.into(), on_sample, request)
        }
        cpal::SampleFormat::U16 => {
            stream_make::<u16, _>(&device, &config.into(), on_sample, request)
        }
    }
}

pub fn host_device_setup(
) -> Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig), anyhow::Error> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
    println!("Output device : {}", device.name()?);

    let config = device.default_output_config()?;
    println!("Default output config : {:?}", config);

    Ok((host, device, config))
}

pub fn stream_make<T, F>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    on_sample: F,
    mut request: SampleRequestOptions,
) -> Result<cpal::Stream, anyhow::Error>
where
    T: cpal::Sample,
    F: FnMut(&mut SampleRequestOptions) -> f32 + std::marker::Send + 'static + Copy,
{
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            on_window(output, &mut request, on_sample)
        },
        err_fn,
    )?;

    Ok(stream)
}

fn on_window<T, F>(output: &mut [T], request: &mut SampleRequestOptions, mut on_sample: F)
where
    T: cpal::Sample,
    F: FnMut(&mut SampleRequestOptions) -> f32 + std::marker::Send + 'static,
{
    for frame in output.chunks_mut(request.nchannels) {
        let value: T = cpal::Sample::from::<f32>(&on_sample(request));
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

fn main() {
    nannou::app(model).event(event).update(update).run();
}

struct Model {
    tx: Sender<f32>,
    _window: window::Id,
    allowed_segments: [(i32, i32); 16],
    id: u16,
}

fn model(app: &App) -> Model {
    let tx = start_audio();

    let _window = app.new_window().size(600, 600).view(view).build().unwrap();
    Model {
        tx,
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
    match event {
        Event::DeviceEvent(_id, event) => match event {
            nannou::winit::event::DeviceEvent::Button { button, state } => {
                if button == 0 && state == ElementState::Released {
                    model.id = random_range(0, 65536) as u16;
                }
            }
            nannou::winit::event::DeviceEvent::MouseMotion { delta: _ } => {
                model
                    .tx
                    .send((app.mouse.y + 300.) / 600. * 1000. + 1000.)
                    .unwrap();
                ()
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
