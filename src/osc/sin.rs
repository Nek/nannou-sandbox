
use std::collections::HashMap;
use crate::defs::node::SignalProcessor;
use crate::defs::node::Parametric;
use crate::defs::node::SignalGenerator;
use crate::defs::node::SignalReceiver;
use derivative::Derivative;

#[derive(Debug, Eq, Derivative)]
#[derivative(PartialEq, Hash)]
pub enum SinOscParameters {
    Pitch,
}

pub struct SinOsc {
    pub phase: f32,
    pub output: f32,
    pub input: f32,
    pub parameters: HashMap<SinOscParameters, f32>,
}

impl SignalProcessor for SinOsc {
    fn tick(&mut self, sample_rate: f32) {
        let freq = if let Some(freq) = self.parameters.get(&SinOscParameters::Pitch) {
            *freq
        } else {
            0.
        };
        self.phase += freq * 1. / sample_rate;

        if self.phase >= 0.5 {
            self.phase -= 1.
        }

        self.output = (self.phase * 2.0 * std::f32::consts::PI).sin() * 0.1;
    }
}

impl Parametric<SinOscParameters> for SinOsc {
    fn set_parameter(&mut self, id: SinOscParameters, value: f32) {
        self.parameters.insert(id, value);
    }

    fn parameter(&mut self, id: SinOscParameters) -> f32 {
        if let Some(freq) = self.parameters.get(&id) {
            *freq
        } else {
            0.
        }
    }
}

impl SignalGenerator for SinOsc {
    fn output(&mut self) -> f32 {
        self.output
    }
}

impl SignalReceiver for SinOsc {
    fn set_input(&mut self, value: f32) {
        self.input = value;
    }
}
