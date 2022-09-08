pub trait SignalProcessor {
    fn process(&mut self, sample_rate: f32);
}

pub trait Parametric<T> {
    fn set_parameter(&mut self, id: T, value: f32);
    fn parameter(&mut self, id: T) -> f32;
}

pub trait SignalReceiver {
    fn set_input(&mut self, value: f32);
}

pub trait SignalGenerator {
    fn output(&mut self) -> f32;
}
