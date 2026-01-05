use super::Mean;

#[derive(Clone, Debug)]
pub struct ExpMean {
    periods: u32,
    alpha: f64,
    buff: Mean,
    prev_value: Option<f64>,
    value: Option<f64>,
}

impl ExpMean {
    pub fn new(periods: u32) -> Self {
        assert!(periods > 1);
        Self {
            periods,
            alpha: 2.0 / (periods as f64 + 1.0),
            buff: Mean::new(periods),
            prev_value: None,
            value: None,
        }
    }

    pub fn update_after_close(&mut self, x: f64) {
        self.buff.update_after_close(x);

        if let Some(last_ema) = self.value {
            let ema = (self.alpha * x) + (1.0 - self.alpha) * last_ema;
            self.prev_value = Some(last_ema);
            self.value = Some(ema);
        } else if self.buff.is_ready() {
            self.value = self.buff.get_last();
        }
    }

    pub fn update_before_close(&mut self, x: f64) {
        if let Some(last_ema) = self.prev_value {
            let ema = (self.alpha * x) + (1.0 - self.alpha) * last_ema;
            self.value = Some(ema);
        }

        if self.buff.is_ready() {
            self.buff.update_before_close(x);
        }
    }

    pub fn load(&mut self, data: &[f64]) {
        for x in data {
            self.update_after_close(*x);
        }
    }

    #[inline]
    pub fn is_ready(&self) -> bool {
        self.value.is_some()
    }

    #[inline]
    pub fn get_last(&self) -> Option<f64> {
        self.value
    }

    pub fn reset(&mut self) {
        self.buff.reset();
        self.prev_value = None;
        self.value = None;
    }

    #[inline]
    pub fn period(&self) -> u32 {
        self.periods
    }
}
