use crate::indicators::{Indicator, Price, Value};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Roc {
    periods: u32,
    closes: VecDeque<f64>,
    value: Option<f64>,
    in_candle: bool,
}

impl Roc {
    pub fn new(periods: u32) -> Self {
        assert!(periods > 0, "ROC period must be > 0, got {}", periods);
        Self {
            periods,
            closes: VecDeque::with_capacity(periods as usize + 1),
            value: None,
            in_candle: true,
        }
    }

    fn compute(&mut self) {
        if self.closes.len() != self.periods as usize + 1 {
            self.value = None;
            return;
        }

        let first = self.closes.front().copied().unwrap();
        let last = self.closes.back().copied().unwrap();

        self.value = if first.abs() <= f64::EPSILON {
            None
        } else {
            Some(((last / first) - 1.0) * 100.0)
        };
    }
}

impl Indicator for Roc {
    fn update_after_close(&mut self, price: Price) {
        if self.closes.len() == self.periods as usize + 1 {
            if self.in_candle {
                self.closes.pop_front();
            } else {
                self.closes.pop_back();
            }
        }

        self.closes.push_back(price.close);
        self.compute();
        self.in_candle = true;
    }

    fn update_before_close(&mut self, price: Price) {
        if self.closes.len() != self.periods as usize + 1 {
            return;
        }

        if self.in_candle {
            self.closes.pop_front();
            self.in_candle = false;
        } else {
            self.closes.pop_back();
        }

        self.closes.push_back(price.close);
        self.compute();
    }

    fn load(&mut self, price_data: &[Price]) {
        for price in price_data {
            self.update_after_close(*price);
        }
    }

    fn is_ready(&self) -> bool {
        self.value.is_some()
    }

    fn get_last(&self) -> Option<Value> {
        self.value.map(Value::RocValue)
    }

    fn reset(&mut self) {
        self.closes.clear();
        self.value = None;
        self.in_candle = true;
    }

    fn period(&self) -> u32 {
        self.periods
    }
}

impl Default for Roc {
    fn default() -> Self {
        Self::new(12)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(close: f64) -> Price {
        Price {
            open: close,
            high: close,
            low: close,
            close,
            open_time: 0,
            close_time: 0,
            vlm: 0.0,
        }
    }

    #[test]
    fn roc_needs_period_plus_one_closes() {
        let mut roc = Roc::new(3);

        roc.update_after_close(p(100.0));
        roc.update_after_close(p(100.0));
        roc.update_after_close(p(100.0));

        assert!(!roc.is_ready());

        roc.update_after_close(p(110.0));

        assert_eq!(roc.get_last(), Some(Value::RocValue(10.000000000000009)));
    }

    #[test]
    fn roc_updates_after_warmup() {
        let mut roc = Roc::new(3);

        for close in [100.0, 100.0, 100.0, 110.0] {
            roc.update_after_close(p(close));
        }

        let first = match roc.get_last() {
            Some(Value::RocValue(value)) => value,
            _ => panic!("missing roc"),
        };

        roc.update_after_close(p(121.0));

        let second = match roc.get_last() {
            Some(Value::RocValue(value)) => value,
            _ => panic!("missing updated roc"),
        };

        assert_ne!(first, second);
    }

    #[test]
    fn roc_before_close_is_provisional() {
        let mut roc = Roc::new(3);

        for close in [100.0, 100.0, 100.0, 110.0, 121.0] {
            roc.update_after_close(p(close));
        }

        let after_close = match roc.get_last() {
            Some(Value::RocValue(value)) => value,
            _ => panic!("missing roc"),
        };

        roc.update_before_close(p(130.0));

        let provisional = match roc.get_last() {
            Some(Value::RocValue(value)) => value,
            _ => panic!("missing provisional roc"),
        };

        assert_ne!(after_close, provisional);
    }

    #[test]
    fn roc_reset_clears_state() {
        let mut roc = Roc::new(3);

        for close in [100.0, 100.0, 100.0, 110.0] {
            roc.update_after_close(p(close));
        }

        assert!(roc.is_ready());

        roc.reset();

        assert!(!roc.is_ready());
        assert_eq!(roc.get_last(), None);
        assert!(roc.closes.is_empty());
    }
}
