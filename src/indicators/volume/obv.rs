use crate::indicators::{Indicator, Price, Value};

#[derive(Clone, Debug)]
pub struct Obv {
    prev_close: Option<f64>,
    confirmed_value: Option<f64>,
    value: Option<f64>,
}

impl Obv {
    pub fn new() -> Self {
        Self {
            prev_close: None,
            confirmed_value: None,
            value: None,
        }
    }

    fn next_value(base: f64, prev_close: f64, price: Price) -> f64 {
        if price.close > prev_close {
            base + price.vlm
        } else if price.close < prev_close {
            base - price.vlm
        } else {
            base
        }
    }
}

impl Indicator for Obv {
    fn update_after_close(&mut self, price: Price) {
        match (self.prev_close, self.confirmed_value) {
            (Some(prev_close), Some(base)) => {
                let next = Self::next_value(base, prev_close, price);
                self.confirmed_value = Some(next);
                self.value = Some(next);
            }
            _ => {
                self.confirmed_value = Some(0.0);
                self.value = Some(0.0);
            }
        }

        self.prev_close = Some(price.close);
    }

    fn update_before_close(&mut self, price: Price) {
        if let (Some(prev_close), Some(base)) = (self.prev_close, self.confirmed_value) {
            self.value = Some(Self::next_value(base, prev_close, price));
        }
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
        self.value.map(Value::ObvValue)
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.confirmed_value = None;
        self.value = None;
    }

    fn period(&self) -> u32 {
        1
    }
}

impl Default for Obv {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(close: f64, volume: f64) -> Price {
        Price {
            open: close,
            high: close,
            low: close,
            close,
            open_time: 0,
            close_time: 0,
            vlm: volume,
        }
    }

    #[test]
    fn obv_initializes_to_zero_then_accumulates() {
        let mut obv = Obv::new();

        obv.update_after_close(p(100.0, 5.0));
        assert_eq!(obv.get_last(), Some(Value::ObvValue(0.0)));

        obv.update_after_close(p(101.0, 7.0));
        assert_eq!(obv.get_last(), Some(Value::ObvValue(7.0)));

        obv.update_after_close(p(99.0, 4.0));
        assert_eq!(obv.get_last(), Some(Value::ObvValue(3.0)));
    }

    #[test]
    fn obv_before_close_is_provisional() {
        let mut obv = Obv::new();

        obv.update_after_close(p(100.0, 5.0));
        obv.update_after_close(p(101.0, 7.0));

        let after_close = obv.get_last();
        obv.update_before_close(p(99.0, 10.0));

        assert_ne!(after_close, obv.get_last());
        assert_eq!(obv.get_last(), Some(Value::ObvValue(-3.0)));
    }

    #[test]
    fn obv_reset_clears_state() {
        let mut obv = Obv::new();

        obv.update_after_close(p(100.0, 5.0));
        obv.update_after_close(p(101.0, 7.0));
        assert!(obv.is_ready());

        obv.reset();

        assert!(!obv.is_ready());
        assert_eq!(obv.get_last(), None);
    }
}
