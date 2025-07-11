use std::collections::VecDeque;
use crate::indicators::{Value,Price, Indicator};

#[derive(Clone, Debug)]
pub struct Adx {
    periods: u32,
    buff: AdxBuffer,
    prev_close: Option<f64>,
    prev_value: Option<f64>,
    value: Option<f64>,
}

#[derive(Clone, Debug)]
struct AdxBuffer {
    di_length: u32,
    prev_high: Option<f64>,
    prev_low: Option<f64>,
    prev_dm_pos: Option<f64>,
    prev_dm_neg: Option<f64>,
    prev_tr: Option<f64>,
    dx_buffer: VecDeque<f64>,
    dx: Option<f64>,
}

impl Adx {
    pub fn new(periods: u32, di_length: u32) -> Self {
        assert!(periods > 0, "Adx periods must be > 0, got {}", periods);
        Adx {
            periods,
            buff: AdxBuffer::new(di_length),
            prev_close: None,
            prev_value: None,
            value: None,
        }
    }

    fn calc_adx(&mut self, dx: f64, after: bool) {
    let length = self.periods;

    if self.prev_value.is_none(){
        if after{
            self.buff.dx_buffer.push_back(dx);
            if self.buff.dx_buffer.len() == length as usize{
                let sum: f64 = self.buff.dx_buffer.iter().sum();
                let initial_adx = sum / length as f64;
                self.prev_value = Some(initial_adx);
                self.value = Some(initial_adx);
        }}
    } else {
        let prev_adx = self.prev_value.unwrap();
        let new_adx = (prev_adx * (length as f64 - 1.0) + dx) / length as f64;
        self.value = Some(new_adx);
        if after {
            self.prev_value = Some(new_adx);
        }
    }
}
}

impl Indicator for Adx {
    fn update_after_close(&mut self, price: Price) {
        let h = price.high;
        let l = price.low;
        let close = price.close;
        let h_l = h - l;

        let tr = if let Some(prev_close) = self.prev_close {
            h_l.max((h - prev_close).abs().max((l - prev_close).abs()))
        } else {
            h_l
        };

        self.buff.update_after_close(h, l, tr);
        self.prev_close = Some(close);

        if let Some(dx) = self.buff.dx {
            self.calc_adx(dx, true);
        }
    }

    fn update_before_close(&mut self, price: Price) {
        if self.is_ready(){
            let h = price.high;
            let l = price.low;
            let h_l = h - l;

            if let Some(prev_close) = self.prev_close {
                let tr = h_l.max((h - prev_close).abs().max((l - prev_close).abs()));
                self.buff.update_before_close(h, l, tr);
            }

            if let Some(dx) = self.buff.dx {
                self.calc_adx(dx, false);
            }
        }
}

  
    fn load(&mut self, price_data: &[Price]){

        for p in price_data{
            self.update_after_close(*p);
        }
    }
    fn get_last(&self) -> Option<Value> {
        if let Some(val) = self.value{
            return Some(Value::AdxValue(val));
        }
        None
    }

    fn is_ready(&self) -> bool {
        self.value.is_some()
    }

    fn period(&self) -> u32{
        self.periods
    }

    fn reset(&mut self) {
        self.value = None;
        self.prev_close = None;
        self.prev_value = None;
        self.buff.reset();
    }
}

impl AdxBuffer {
    fn new(di_length: u32) -> Self {
        assert!(di_length > 0, "Adx di_length must be > 0, got {}", di_length);
        AdxBuffer {
            di_length,
            prev_high: None,
            prev_low: None,
            prev_dm_pos: None,
            prev_dm_neg: None,
            prev_tr: None,
            dx_buffer: VecDeque::with_capacity(di_length as usize),
            dx: None,
        }
    }

    fn update_after_close(&mut self, high: f64, low: f64, tr: f64) {
        let length = self.di_length as f64;

        if let Some(smoothed_tr) = self.prev_tr {
            let new_tr = (smoothed_tr * (length - 1.0) + tr) / length;
            self.prev_tr = Some(new_tr);
        } else {
            self.prev_tr = Some(tr);
            return;
        }

        if let (Some(prev_high), Some(prev_low)) = (self.prev_high, self.prev_low) {
            let up_move = high - prev_high;
            let down_move = prev_low - low;
            let dm_pos = if up_move > down_move && up_move > 0.0 { up_move } else { 0.0 };
            let dm_neg = if down_move > up_move && down_move > 0.0 { down_move } else { 0.0 };

            if let (Some(prev_dm_pos), Some(prev_dm_neg)) = (self.prev_dm_pos, self.prev_dm_neg) {
                let smoothed_dm_pos = (prev_dm_pos * (length - 1.0) + dm_pos) / length;
                let smoothed_dm_neg = (prev_dm_neg * (length - 1.0) + dm_neg) / length;
                self.prev_dm_pos = Some(smoothed_dm_pos);
                self.prev_dm_neg = Some(smoothed_dm_neg);
                self.calc_dx(smoothed_dm_pos, smoothed_dm_neg, self.prev_tr.unwrap());
            } else {
                self.prev_dm_pos = Some(dm_pos);
                self.prev_dm_neg = Some(dm_neg);
                return;
            }
        }

        self.prev_high = Some(high);
        self.prev_low = Some(low);
    }

    fn update_before_close(&mut self, high: f64, low: f64, tr: f64) {
        self.dx = None;
        let length = self.di_length as f64;

        let provisional_tr = if let Some(smoothed_tr) = self.prev_tr {
            (smoothed_tr * (length - 1.0) + tr) / length
        } else {
            return;
        };

        if let (Some(prev_high), Some(prev_low)) = (self.prev_high, self.prev_low) {
            let up_move = high - prev_high;
            let down_move = prev_low - low;
            let dm_pos = if up_move > down_move && up_move > 0.0 { up_move } else { 0.0 };
            let dm_neg = if down_move > up_move && down_move > 0.0 { down_move } else { 0.0 };

            if let (Some(prev_dm_pos), Some(prev_dm_neg)) = (self.prev_dm_pos, self.prev_dm_neg) {
                let provisional_dm_pos = (prev_dm_pos * (length - 1.0) + dm_pos) / length;
                let provisional_dm_neg = (prev_dm_neg * (length - 1.0) + dm_neg) / length;
                self.calc_dx(provisional_dm_pos, provisional_dm_neg, provisional_tr);
            }
        }
    }

    fn calc_dx(&mut self, dm_pos: f64, dm_neg: f64, tr: f64) {
        if tr <= f64::EPSILON {
            self.dx = Some(0.0);
            return;
        }

        let di_pos = 100.0 * (dm_pos / tr);
        let di_neg = 100.0 * (dm_neg / tr);
        let diff = (di_pos - di_neg).abs();
        let sum = di_pos + di_neg;

        let dx = if sum > 0.0 {
            100.0 * (diff / sum)
        } else {
            0.0
        };

        self.dx = Some(dx);
    }

    fn reset(&mut self) {
        self.prev_high = None;
        self.prev_low = None;
        self.prev_dm_pos = None;
        self.prev_dm_neg = None;
        self.prev_tr = None;
        self.dx_buffer.clear();
        self.dx = None;
    }
}
