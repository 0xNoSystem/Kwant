use std::collections::VecDeque;
use crate::indicators::{Price, Indicator};

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
}

impl StochRsi {
    pub fn new(length: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(length),
            length,
            value: None,
            current_min: f32::INFINITY,
            current_max: f32::NEG_INFINITY,
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
        } else {
            self.value = None;
        }
    }

    pub fn get(&self) -> Option<f32> {
        self.value
    }

    pub fn is_ready(&self) -> bool {
        self.value.is_some()
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.value = None;
        self.current_min = f32::INFINITY;
        self.current_max = f32::NEG_INFINITY;
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

#[derive(Clone, Debug)]
pub struct Rsi {
    pub periods: usize,
    buff: RsiBuffer,
    last_price: Option<f32>,
    value: Option<f32>,
    stoch: StochRsi,
}

impl Rsi {
    pub fn new(periods: usize, stoch_length: usize) -> Self {
        Self {
            periods,
            buff: RsiBuffer::new(periods - 1),
            last_price: None,
            value: None,
            stoch: StochRsi::new(stoch_length),
        }
    }

    fn calc_rsi(&mut self, change: f32, last_avg_gain: f32, last_avg_loss: f32, after: bool) -> Option<f32> {
        let change_loss = (-change).max(0.0);
        let change_gain = change.max(0.0);
        let avg_gain = (last_avg_gain * (self.periods as f32 - 1.0) + change_gain) / self.periods as f32;
        let avg_loss = (last_avg_loss * (self.periods as f32 - 1.0) + change_loss) / self.periods as f32;
        let rsi = if avg_loss == 0.0 {
            100.0
        } else {
            100.0 - (100.0 / (1.0 + (avg_gain / avg_loss)))
        };
        if after {
            self.buff.last_avg_gain = Some(avg_gain);
            self.buff.last_avg_loss = Some(avg_loss);
            self.stoch.update_after_close(rsi);
        } else {
            if self.stoch.is_ready() {
                self.stoch.update_before_close(rsi);
            }
        }
        self.value = Some(rsi);
        Some(rsi)
    }

    pub fn get_rsi(&self) -> Option<f32> {
        self.value
    }

    pub fn get_stoch_rsi(&self) -> Option<f32> {
        self.stoch.get()
    }

    pub fn reset(&mut self) {
        self.buff = RsiBuffer::new(self.periods - 1);
        self.last_price = None;
        self.value = None;
        self.stoch.reset();
    }
}

impl Indicator for Rsi {
    fn update_before_close(&mut self, price: Price) {
        let close = price.close;
        let change = match self.last_price {
            Some(p) => close - p,
            None => {
                self.last_price = Some(close);
                return;
            }
        };
        self.buff.push_before_close(change);
        if self.buff.is_full() {
            if let (Some(g), Some(l)) = (self.buff.last_avg_gain, self.buff.last_avg_loss) {
                self.calc_rsi(change, g, l, false);
            }
        }
    }

    fn update_after_close(&mut self, price: Price) {
        let close = price.close;
        let change = match self.last_price {
            Some(p) => close - p,
            None => {
                self.last_price = Some(close);
                return;
            }
        };
        self.buff.push(change);
        self.last_price = Some(close);
        if self.buff.is_full() {
            if let (Some(g), Some(l)) = (self.buff.last_avg_gain, self.buff.last_avg_loss) {
                self.calc_rsi(change, g, l, true);
            }
        }
    }

    fn get_last(&self) -> Option<f32> {
        self.value
    }

    fn is_ready(&self) -> bool {
        self.buff.is_full() && self.value.is_some()
    }

    fn load(&mut self, price_data: &Vec<Price>) {
        if price_data.len() > 1 {
            for p in price_data {
                self.update_after_close(*p);
            }
        }
    }

    fn reset(&mut self) {
        self.reset();
    }
}
impl Default for Rsi {
    fn default() -> Self {
        Rsi {
            periods: 14,
            buff: RsiBuffer::new(13),
            last_price: None,
            value: None,
            stoch: StochRsi::new(14),
        }
    }
}




#[derive(Clone, Debug)]
struct RsiBuffer {
    changes_buffer: VecDeque<f32>,
    sum_gain: f32,
    sum_loss: f32,
    last_avg_gain: Option<f32>,
    last_avg_loss: Option<f32>,
}

impl RsiBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            changes_buffer: VecDeque::with_capacity(capacity),
            sum_gain: 0.0,
            sum_loss: 0.0,
            last_avg_gain: None,
            last_avg_loss: None,
        }
    }

    fn push(&mut self, change: f32) {
        if self.is_full() {
            self.init_last_avg();
            let expired = self.changes_buffer.pop_front().unwrap();
            if expired > 0.0 {
                self.sum_gain -= expired;
            } else {
                self.sum_loss -= expired.abs();
            }
        }
        if change > 0.0 {
            self.sum_gain += change;
        } else {
            self.sum_loss += change.abs();
        }
        self.changes_buffer.push_back(change);
    }

    fn push_before_close(&mut self, change: f32) {
        if self.is_full() {
            let expired = self.changes_buffer.pop_back().unwrap();
            if expired > 0.0 {
                self.sum_gain -= expired;
            } else {
                self.sum_loss -= expired.abs();
            }
            if change > 0.0 {
                self.sum_gain += change;
            } else {
                self.sum_loss += change.abs();
            }
            self.changes_buffer.push_back(change);
        }
    }

    fn is_full(&self) -> bool {
        self.changes_buffer.len() == self.changes_buffer.capacity()
    }

    fn init_last_avg(&mut self) {
        if self.last_avg_gain.is_none() {
            self.last_avg_gain = Some(self.sum_gain / self.changes_buffer.capacity() as f32);
        }
        if self.last_avg_loss.is_none() {
            self.last_avg_loss = Some(self.sum_loss / self.changes_buffer.capacity() as f32);
        }
    }
}
