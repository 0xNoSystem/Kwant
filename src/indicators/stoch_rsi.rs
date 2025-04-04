use std::collections::VecDeque;

fn is_same(a: f32, b: f32) -> bool {
    (a - b).abs() < f32::EPSILON
}

#[derive(Clone, Debug)]
pub struct StochRsi {
    buffer: VecDeque<f32>,
    length: usize,
    current_min: f32,
    current_max: f32,

    // Buffers for double smoothing
    k_smoothing_buffer: VecDeque<f32>,
    k_value: Option<f32>,
    d_buffer: VecDeque<f32>,
    d_value: Option<f32>,
    in_candle: bool,
}

impl StochRsi {
    /// `length` = how many RSI values to consider for raw stoch
    /// `k_smoothing` = smoothing period for %K
    /// `d_smoothing` = smoothing period for %D signal
    pub fn new(length: usize, k_smoothing: usize, d_smoothing: usize) -> Self {
        assert!(length > 1, "Stoch length must be > 1");
        assert!(k_smoothing > 0, "k_smoothing must be > 0");
        assert!(d_smoothing > 0, "d_smoothing must be > 0");

        Self {
            buffer: VecDeque::with_capacity(length),
            length,
            current_min: f32::INFINITY,
            current_max: f32::NEG_INFINITY,

            k_smoothing_buffer: VecDeque::with_capacity(k_smoothing),
            k_value: None,
            d_buffer: VecDeque::with_capacity(d_smoothing),
            d_value: None,
            in_candle: false,
        }
    }

    pub fn periods(&self) -> usize {
        self.length
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
        self.in_candle = false;
        self.compute_stoch_rsi(rsi, true);
        
    }

    pub fn update_before_close(&mut self, rsi: f32) {
        if let Some(&old_rsi) = self.buffer.back() {
            if is_same(old_rsi, rsi) {
                return;
            }
        }
        if self.buffer.len() == self.length {
            let expired: f32;
            if !self.in_candle{
                expired = self.buffer.pop_front().unwrap();
            }else{
                expired = self.buffer.pop_back().unwrap();
                }
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

        self.compute_stoch_rsi(rsi,false);
        self.in_candle = true;
    }

    fn compute_stoch_rsi(&mut self, latest_rsi: f32, after: bool) {
        if self.buffer.len() == self.length && self.current_max != self.current_min {
            let raw_k = (latest_rsi - self.current_min) / (self.current_max - self.current_min);
            self.push_k_smoothing(raw_k, after);
        } else {
            self.k_value = None;
            self.d_value = None;
        }
    }

    fn push_k_smoothing(&mut self, raw_k: f32, after: bool) {

        let k_len = self.k_smoothing_buffer.capacity();
        
        if self.k_smoothing_buffer.len() == k_len{
            if after{
                self.k_smoothing_buffer.pop_front();
                self.k_smoothing_buffer.push_back(raw_k);
            }else{
                if !self.in_candle{
                    self.k_smoothing_buffer.pop_back();
                }else{
                    self.k_smoothing_buffer.pop_front();
                }
                self.k_smoothing_buffer.push_back(raw_k);
            }
        }else{
            if after{
                self.k_smoothing_buffer.push_back(raw_k);
            }
        }
        
        if self.k_smoothing_buffer.len() == k_len {
            let sum_k: f32 = self.k_smoothing_buffer.iter().sum();
            let smoothed_k = sum_k / k_len as f32;
            self.k_value = Some(smoothed_k);
            self.push_d_smoothing(smoothed_k, after);
        } else {
            self.k_value = None;
            self.d_value = None;
        }
    }

    fn push_d_smoothing(&mut self, k_val: f32, after: bool) {
        let d_len = self.d_buffer.capacity();
        
        if self.d_buffer.len() == d_len{
            if after{
                self.d_buffer.pop_front();
                self.d_buffer.push_back(k_val);
            }else{
                if !self.in_candle{
                    self.d_buffer.pop_back();
                }else{
                    self.d_buffer.pop_front();
                }
                self.d_buffer.push_back(k_val);
            }
            }else if after{  
                self.d_buffer.push_back(k_val);
            }
        
        // If the buffer is not full, we don't compute the average
        if self.d_buffer.len() == d_len {
            let sum_d: f32 = self.d_buffer.iter().sum();
            self.d_value = Some(sum_d / d_len as f32);
        } else {
            self.d_value = None;
        }
    }

    pub fn get_k(&self) -> Option<f32> {
        self.k_value.map(|val| val * 100.0)
    }

    pub fn get_d(&self) -> Option<f32> {
        self.d_value.map(|val| val * 100.0)
    }

    pub fn is_ready(&self) -> bool {
        self.k_value.is_some() && self.d_value.is_some()
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.current_min = f32::INFINITY;
        self.current_max = f32::NEG_INFINITY;
        self.k_smoothing_buffer.clear();
        self.k_value = None;
        self.d_buffer.clear();
        self.d_value = None;
        self.in_candle = false;
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


