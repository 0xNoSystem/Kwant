use std::collections::VecDeque;

fn is_same(a: f32, b: f32) -> bool {
    (a - b).abs() < f32::EPSILON
}

#[derive(Clone, Debug)]
pub struct StochRsi {
    buffer: VecDeque<f32>,
    length: usize,
    value: Option<f32>,
    current_min: f32,
    current_max: f32,
    signal_buffer: VecDeque<f32>,
    signal_value: Option<f32>,
}

impl StochRsi {
    pub fn new(length: usize) -> Self {
        assert!(length > 1, "Stoch length field must be a positive integer > 1, ({})", length);
        Self {
            buffer: VecDeque::with_capacity(length),
            length,
            value: None,
            current_min: f32::INFINITY,
            current_max: f32::NEG_INFINITY,
            signal_buffer: VecDeque::with_capacity(3),
            signal_value: None,
        }
    }

    pub fn update_after_close(&mut self, rsi: f32) {
        if self.buffer.len() == self.length {
            let expired = self.buffer.pop_front().unwrap();
            if is_same(expired, self.current_min) || is_same(expired, self.current_max) {
                self.recompute_min_max();
            }
        }
        self.buffer.push_back(rsi);
        if rsi < self.current_min {
            self.current_min = rsi;
        }
        if rsi > self.current_max {
            self.current_max = rsi;
        }
        self.compute_stoch_value(rsi);
    }

    pub fn update_before_close(&mut self, rsi: f32) {
        if let Some(&old_rsi) = self.buffer.back() {
            if is_same(old_rsi, rsi) {
                return;
            }
        }
        if self.buffer.len() == self.length {
            let old_rsi = self.buffer.pop_back().unwrap();
            if is_same(old_rsi, self.current_min) || is_same(old_rsi, self.current_max) {
                self.recompute_min_max();
            }
        }
        self.buffer.push_back(rsi);
        if rsi < self.current_min {
            self.current_min = rsi;
        }
        if rsi > self.current_max {
            self.current_max = rsi;
        }
        self.compute_stoch_value(rsi);
    }

    fn compute_stoch_value(&mut self, latest_rsi: f32) {
        if self.buffer.len() == self.length && self.current_max != self.current_min {
            let stoch = (latest_rsi - self.current_min) / (self.current_max - self.current_min);
            self.value = Some(stoch);
            self.update_signal_line(stoch);
        } else {
            self.value = None;
            self.signal_value = None;
        }
    }

    fn update_signal_line(&mut self, stoch: f32) {
        if self.signal_buffer.len() == 3 {
            self.signal_buffer.pop_front();
        }
        self.signal_buffer.push_back(stoch);
        if self.signal_buffer.len() == 3 {
            let sum: f32 = self.signal_buffer.iter().sum();
            self.signal_value = Some(sum / 3.0);
        } else {
            self.signal_value = None;
        }
    }

    pub fn get(&self) -> Option<f32> {
        self.value
    }

    pub fn get_signal(&self) -> Option<f32> {
        self.signal_value
    }

    pub fn is_ready(&self) -> bool {
        self.value.is_some()
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.value = None;
        self.current_min = f32::INFINITY;
        self.current_max = f32::NEG_INFINITY;
        self.signal_buffer.clear();
        self.signal_value = None;
    }

    fn recompute_min_max(&mut self) {
        self.current_min = f32::INFINITY;
        self.current_max = f32::NEG_INFINITY;
        for &val in &self.buffer {
            if val < self.current_min {
                self.current_min = val;
            }
            if val > self.current_max {
                self.current_max = val;
            }
        }
    }
}
