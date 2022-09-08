use dsp::Walker;
use dsp::{FromSample, Graph, Node, Sample};

/// Our type for which we will implement the `Dsp` trait.
#[derive(Debug)]
pub enum DspNode {
    /// Synth will be our demonstration of a master GraphNode.
    Synth,
    /// Oscillator will be our generator type of node, meaning that we will override
    /// the way it provides audio via its `audio_requested` method.
    Oscillator(Phase, Frequency, Volume),
}

impl Node<[Output; CHANNELS]> for DspNode {
    /// Here we'll override the audio_requested method and generate a sine wave.
    fn audio_requested(&mut self, buffer: &mut [[Output; CHANNELS]], sample_hz: f64) {
        match *self {
            DspNode::Synth => (),
            DspNode::Oscillator(ref mut phase, frequency, volume) => {
                dsp::slice::map_in_place(buffer, |_| {
                    let val = sine_wave(*phase, volume);
                    *phase += frequency / sample_hz;
                    dsp::Frame::from_fn(|_| val)
                });
            }
        }
    }
}

/// Return a sine wave for the given phase.
fn sine_wave<S: Sample>(phase: Phase, volume: Volume) -> S
where
    S: Sample + FromSample<f32>,
{
    use std::f64::consts::PI;
    ((phase * PI * 2.0).sin() as f32 * volume).to_sample::<S>()
}

/// SoundStream is currently generic over i8, i32 and f32. Feel free to change it!
type Output = f32;

type Phase = f64;
type Frequency = f64;
type Volume = f32;

const CHANNELS: usize = 2;
const FRAMES: u32 = 64;
const SAMPLE_HZ: f64 = 44_100.0;

const A5_HZ: Frequency = 440.0;
const D5_HZ: Frequency = 587.33;
const F5_HZ: Frequency = 698.46;

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
    let mut graph = Graph::new();

    // Construct our fancy Synth and add it to the graph!
    let synth = graph.add_node(DspNode::Synth);

    let mut osc1 = DspNode::Oscillator(0.0, A5_HZ, 0.2);
    let _osc2 = DspNode::Oscillator(0.0, D5_HZ, 0.1);
    let _osc3 = DspNode::Oscillator(0.0, F5_HZ, 0.15);

    // Connect a few oscillators to the synth.
    graph.add_input(osc1, synth);
    graph.add_input(_osc2, synth);
    graph.add_input(_osc3, synth);

    // Set the synth as the master node for the graph.
    graph.set_master(Some(synth));

    let (tx, rx): (Sender<f32>, Receiver<f32>) = channel();

    let mut request = SampleRequestOptions {
        sample_rate: 0.,
        nchannels: 0,
        rx,
        graph,
        synth,
    };

    thread::spawn(|| -> anyhow::Result<()> {
        let stream = stream_setup_for(request)?;
        stream.play()?;
        thread::sleep(Duration::MAX);
        Ok(())
    });
    tx
}

pub struct SampleRequestOptions {
    sample_rate: f32,
    nchannels: usize,
    rx: Receiver<f32>,
    graph: Graph<[f32; 2], DspNode>,
    synth: dsp::NodeIndex,
}

pub fn stream_setup_for(mut request: SampleRequestOptions) -> Result<cpal::Stream, anyhow::Error> {
    let (_host, device, config) = host_device_setup()?;

    let sample_rate = config.sample_rate().0 as f32;
    let nchannels = config.channels() as usize;

    request.sample_rate = sample_rate;
    request.nchannels = nchannels;

    match config.sample_format() {
        cpal::SampleFormat::F32 => stream_make::<f32>(&device, &config.into(), request),
        cpal::SampleFormat::I16 => stream_make::<i16>(&device, &config.into(), request),
        cpal::SampleFormat::U16 => stream_make::<u16>(&device, &config.into(), request),
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

pub fn stream_make<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut request: SampleRequestOptions,
) -> Result<cpal::Stream, anyhow::Error>
where
    T: cpal::Sample,
{
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            let buffer = &mut [[0.0_f32, 0.0_f32]; 512];

            loop {
                if let Ok(v) = request.rx.try_recv() {
                    let mut inputs = request.graph.inputs(request.synth);
                    while let Some(input_idx) = inputs.next_node(&request.graph) {
                        if let DspNode::Oscillator(_, ref mut pitch, _) = request.graph[input_idx] {
                            *pitch = v as f64;
                        }
                    }
                } else {
                    break;
                };
            }

            request.graph.audio_requested(buffer, SAMPLE_HZ);
            let mut index = 0;
            for frame in output.chunks_mut(request.nchannels) {
                let channels = match buffer.get(index) {
                    Some(v) => v,
                    None => &[0.0_f32, 0.0_f32],
                };
                let l: T = cpal::Sample::from::<f32>(&channels[0]);
                let r: T = cpal::Sample::from::<f32>(&channels[1]);
                frame[0] = l;
                frame[1] = r;
                index += 1;
            }
        },
        err_fn,
    )?;

    Ok(stream)
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
