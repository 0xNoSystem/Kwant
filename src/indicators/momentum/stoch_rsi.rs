use super::rsi::Rsi;
use crate::indicators::{Indicator, Price, Value};
use std::collections::VecDeque;
fn is_same(a: f64, b: f64) -> bool {
    (a - b).abs() < f64::EPSILON
}

#[derive(Clone, Debug)]
pub struct StochasticRsi {
    periods: u32,
    rsi: Rsi,
}

impl StochasticRsi {
    pub fn new(periods: u32, k_smoothing: Option<u32>, d_smoothing: Option<u32>) -> Self {
        StochasticRsi {
            periods,
            rsi: Rsi::new(periods, periods, k_smoothing, d_smoothing, None),
        }
    }
}

impl Indicator for StochasticRsi {
    fn update_before_close(&mut self, price: Price) {
        self.rsi.update_before_close(price);
    }
    fn update_after_close(&mut self, price: Price) {
        self.rsi.update_after_close(price);
    }

    fn is_ready(&self) -> bool {
        self.rsi.stoch_is_ready()
    }

    fn get_last(&self) -> Option<Value> {
        if let (Some(k), Some(d)) = (self.rsi.get_stoch_rsi(), self.rsi.get_stoch_signal()) {
            return Some(Value::StochRsiValue { k, d });
        }
        None
    }

    fn reset(&mut self) {
        self.rsi.reset();
    }

    fn load(&mut self, price_data: &[Price]) {
        self.rsi.load(price_data);
    }

    fn period(&self) -> u32 {
        self.periods
    }
}

impl Default for StochasticRsi {
    fn default() -> Self {
        Self {
            periods: 14,
            rsi: Rsi::new(14, 14, Some(3), Some(3), None),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StochBuffer {
    buffer: VecDeque<f64>,
    length: u32,
    min_buffer: VecDeque<f64>,
    max_buffer: VecDeque<f64>,

    // Buffers for double smoothing
    k_smoothing_buffer: VecDeque<f64>,
    k_sum: f64,
    k_value: Option<f64>,
    d_buffer: VecDeque<f64>,
    d_sum: f64,
    d_value: Option<f64>,
    in_candle: bool,
}

impl StochBuffer {
    /// `length` = how many RSI values to consider for raw stoch
    /// `k_smoothing` = smoothing period for %K
    /// `d_smoothing` = smoothing period for %D signal
    pub fn new(length: u32, k_smoothing: u32, d_smoothing: u32) -> Self {
        assert!(length > 1, "Stoch length must be > 1");
        assert!(k_smoothing > 0, "k_smoothing must be > 0");
        assert!(d_smoothing > 0, "d_smoothing must be > 0");

        Self {
            buffer: VecDeque::with_capacity(length as usize),
            length,
            min_buffer: VecDeque::with_capacity(length as usize),
            max_buffer: VecDeque::with_capacity(length as usize),

            k_smoothing_buffer: VecDeque::with_capacity(k_smoothing as usize),
            k_sum: 0.0,
            k_value: None,
            d_buffer: VecDeque::with_capacity(d_smoothing as usize),
            d_sum: 0.0,
            d_value: None,
            in_candle: true,
        }
    }

    fn push_value(&mut self, rsi: f64) {
        while let Some(&last) = self.min_buffer.back() {
            if last > rsi {
                self.min_buffer.pop_back();
            } else {
                break;
            }
        }
        self.min_buffer.push_back(rsi);

        while let Some(&last) = self.max_buffer.back() {
            if last < rsi {
                self.max_buffer.pop_back();
            } else {
                break;
            }
        }
        self.max_buffer.push_back(rsi);

        self.buffer.push_back(rsi);
    }

    fn rebuild_extrema(&mut self) {
        self.min_buffer.clear();
        self.max_buffer.clear();

        for &value in &self.buffer {
            while let Some(&last) = self.min_buffer.back() {
                if last > value {
                    self.min_buffer.pop_back();
                } else {
                    break;
                }
            }
            self.min_buffer.push_back(value);

            while let Some(&last) = self.max_buffer.back() {
                if last < value {
                    self.max_buffer.pop_back();
                } else {
                    break;
                }
            }
            self.max_buffer.push_back(value);
        }
    }

    fn remove_front_value(&mut self) -> f64 {
        let expired = self.buffer.pop_front().unwrap();
        self.rebuild_extrema();

        expired
    }

    fn remove_back_value(&mut self) -> f64 {
        let expired = self.buffer.pop_back().unwrap();
        self.rebuild_extrema();

        expired
    }

    pub fn update_after_close(&mut self, rsi: f64) {
        let had_provisional = !self.in_candle;

        if self.buffer.len() == self.length as usize {
            if self.in_candle {
                self.remove_front_value();
            } else {
                self.remove_back_value();
            }
        }

        self.push_value(rsi);

        self.compute_stoch_rsi(rsi, true, had_provisional);
        self.in_candle = true;
    }

    pub fn update_before_close(&mut self, rsi: f64) {
        if self.is_ready() {
            let had_provisional = !self.in_candle;
            if let Some(&old_rsi) = self.buffer.back()
                && is_same(old_rsi, rsi)
                && had_provisional
            {
                return;
            }
            if self.buffer.len() == self.length as usize {
                if !self.in_candle {
                    self.remove_back_value();
                } else {
                    self.remove_front_value();
                    self.in_candle = false;
                }
            }
            self.push_value(rsi);
            self.compute_stoch_rsi(rsi, false, had_provisional);
        }
    }

    fn compute_stoch_rsi(&mut self, latest_rsi: f64, after: bool, had_provisional: bool) {
        if self.buffer.len() == self.length as usize
            && !self.min_buffer.is_empty()
            && !self.max_buffer.is_empty()
        {
            let current_min = *self.min_buffer.front().unwrap();
            let current_max = *self.max_buffer.front().unwrap();

            if is_same(current_max, current_min) {
                self.k_value = None;
                self.d_value = None;
                return;
            }

            let raw_k = (latest_rsi - current_min) / (current_max - current_min);
            self.push_k_smoothing(raw_k, after, had_provisional);
        } else {
            self.k_value = None;
            self.d_value = None;
        }
    }

    fn pop_smoothed_value(buffer: &mut VecDeque<f64>, sum: &mut f64, had_provisional: bool) {
        let expired = if had_provisional {
            buffer.pop_back().unwrap()
        } else {
            buffer.pop_front().unwrap()
        };
        *sum -= expired;
    }

    fn push_k_smoothing(&mut self, raw_k: f64, after: bool, had_provisional: bool) {
        let k_len = self.k_smoothing_buffer.capacity();

        if self.k_smoothing_buffer.len() == k_len {
            Self::pop_smoothed_value(
                &mut self.k_smoothing_buffer,
                &mut self.k_sum,
                had_provisional,
            );
        } else {
            if !after {
                self.k_value = None;
                self.d_value = None;
                return;
            }
        }
        self.k_smoothing_buffer.push_back(raw_k);
        self.k_sum += raw_k;

        if self.k_smoothing_buffer.len() == k_len {
            let smoothed_k = self.k_sum / k_len as f64;
            self.k_value = Some(smoothed_k);
            self.push_d_smoothing(smoothed_k, after, had_provisional);
        } else {
            self.k_value = None;
            self.d_value = None;
        }
    }

    fn push_d_smoothing(&mut self, k_val: f64, after: bool, had_provisional: bool) {
        let d_len = self.d_buffer.capacity();

        if self.d_buffer.len() == d_len {
            Self::pop_smoothed_value(&mut self.d_buffer, &mut self.d_sum, had_provisional);
        } else {
            if !after {
                self.d_value = None;
                return;
            }
        }
        self.d_buffer.push_back(k_val);
        self.d_sum += k_val;

        if self.d_buffer.len() == d_len {
            self.d_value = Some(self.d_sum / d_len as f64);
        } else {
            self.d_value = None;
        }
    }

    pub fn get_k(&self) -> Option<f64> {
        self.k_value.map(|val| val * 100.0)
    }

    pub fn get_d(&self) -> Option<f64> {
        self.d_value.map(|val| val * 100.0)
    }

    pub fn is_ready(&self) -> bool {
        self.k_value.is_some() && self.d_value.is_some()
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.min_buffer.clear();
        self.max_buffer.clear();
        self.k_smoothing_buffer.clear();
        self.k_sum = 0.0;
        self.k_value = None;
        self.d_buffer.clear();
        self.d_sum = 0.0;
        self.d_value = None;
        self.in_candle = true;
    }
}

#[cfg(test)]
mod tests {
    use super::StochBuffer;

    fn approx_eq(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "left={a}, right={b}");
    }

    #[test]
    fn provisional_updates_match_close_only_after_close() {
        let mut close_only = StochBuffer::new(3, 2, 2);
        let mut live = StochBuffer::new(3, 2, 2);

        for rsi in [10.0, 20.0, 30.0, 40.0, 35.0] {
            close_only.update_after_close(rsi);
            live.update_after_close(rsi);
        }

        live.update_before_close(31.0);
        live.update_before_close(33.0);
        live.update_after_close(38.0);

        close_only.update_after_close(38.0);

        approx_eq(live.get_k().unwrap(), close_only.get_k().unwrap());
        approx_eq(live.get_d().unwrap(), close_only.get_d().unwrap());
    }
}
