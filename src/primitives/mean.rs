use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Mean {
    periods: u32,
    buff: VecDeque<f64>,
    sum: f64,
    sum_sq: f64,
    value: Option<f64>,
    in_candle: bool,
}

impl Mean {
    pub fn new(periods: u32) -> Self {
        assert!(periods > 0);
        Self {
            periods,
            buff: VecDeque::with_capacity(periods as usize),
            sum: 0.0,
            sum_sq: 0.0,
            value: None,
            in_candle: true,
        }
    }
    #[inline]
    pub fn sum(&self) -> f64 {
        self.sum
    }

    #[inline]
    pub fn sum_sq(&self) -> f64 {
        self.sum_sq
    }
    
    #[inline]
    pub fn len(&self) -> usize {
        self.buff.len()
    }

    pub fn update_after_close(&mut self, x: f64) {
        if self.is_ready() {
            let expired = self.buff.pop_front().unwrap();
            self.sum -= expired;
            self.sum_sq -= expired * expired;
        }

        self.buff.push_back(x);
        self.sum += x;
        self.sum_sq += x * x;

        if self.is_ready() {
            self.value = Some(self.sum / self.periods as f64);
        }

        self.in_candle = true;
    }

    pub fn update_before_close(&mut self, x: f64) {
        if !self.is_ready() {
            return;
        }

        let replaced = if self.in_candle {
            self.in_candle = false;
            self.buff.pop_front().unwrap()
        } else {
            self.buff.pop_back().unwrap()
        };

        self.sum -= replaced;
        self.sum_sq -= replaced * replaced;

        self.buff.push_back(x);
        self.sum += x;
        self.sum_sq += x * x;

        self.value = Some(self.sum / self.periods as f64);
    }

    pub fn load(&mut self, price_data: &[f64]) {
        for p in price_data {
            self.update_after_close(*p);
        }
    }
    
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.buff.len() == self.buff.capacity()
    }

    #[inline]
    pub fn get_last(&self) -> Option<f64> {
        self.value
    }

    pub fn reset(&mut self) {
        self.buff.clear();
        self.sum = 0.0;
        self.sum_sq = 0.0;
        self.value = None;
        self.in_candle = true;
    }

    #[inline]
    pub fn period(&self) -> u32 {
        self.periods
    }
}
