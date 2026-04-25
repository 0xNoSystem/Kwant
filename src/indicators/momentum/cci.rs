use crate::indicators::{Indicator, Price, Value};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Cci {
    periods: u32,
    typical_prices: VecDeque<f64>,
    sum: f64,
    value: Option<f64>,
    in_candle: bool,
}

impl Cci {
    const SCALE: f64 = 0.015;

    pub fn new(periods: u32) -> Self {
        assert!(periods > 1, "CCI period must be > 1, got {}", periods);
        Self {
            periods,
            typical_prices: VecDeque::with_capacity(periods as usize),
            sum: 0.0,
            value: None,
            in_candle: true,
        }
    }

    #[inline]
    fn typical_price(price: Price) -> f64 {
        (price.high + price.low + price.close) / 3.0
    }

    fn remove_value(&mut self, value: f64) {
        self.sum -= value;
    }

    fn add_value(&mut self, value: f64) {
        self.sum += value;
    }

    fn compute(&mut self) {
        if self.typical_prices.len() != self.periods as usize {
            self.value = None;
            return;
        }

        let period = self.periods as f64;
        let sma = self.sum / period;
        let mean_deviation = self
            .typical_prices
            .iter()
            .map(|value| (value - sma).abs())
            .sum::<f64>()
            / period;

        let latest = self.typical_prices.back().copied().unwrap();
        self.value = if mean_deviation <= f64::EPSILON {
            Some(0.0)
        } else {
            Some((latest - sma) / (Self::SCALE * mean_deviation))
        };
    }
}

impl Indicator for Cci {
    fn update_after_close(&mut self, price: Price) {
        let typical_price = Self::typical_price(price);

        if self.typical_prices.len() == self.periods as usize {
            let expired = if self.in_candle {
                self.typical_prices.pop_front().unwrap()
            } else {
                self.typical_prices.pop_back().unwrap()
            };
            self.remove_value(expired);
        }

        self.typical_prices.push_back(typical_price);
        self.add_value(typical_price);
        self.compute();
        self.in_candle = true;
    }

    fn update_before_close(&mut self, price: Price) {
        if self.typical_prices.len() != self.periods as usize {
            return;
        }

        let expired = if self.in_candle {
            self.in_candle = false;
            self.typical_prices.pop_front().unwrap()
        } else {
            self.typical_prices.pop_back().unwrap()
        };
        self.remove_value(expired);

        let typical_price = Self::typical_price(price);
        self.typical_prices.push_back(typical_price);
        self.add_value(typical_price);
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
        self.value.map(Value::CciValue)
    }

    fn reset(&mut self) {
        self.typical_prices.clear();
        self.sum = 0.0;
        self.value = None;
        self.in_candle = true;
    }

    fn period(&self) -> u32 {
        self.periods
    }
}

impl Default for Cci {
    fn default() -> Self {
        Self::new(20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(high: f64, low: f64, close: f64) -> Price {
        Price {
            open: close,
            high,
            low,
            close,
            open_time: 0,
            close_time: 0,
            vlm: 0.0,
        }
    }

    fn approx_eq(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "left={a}, right={b}");
    }

    #[test]
    fn cci_warms_up_and_computes_expected_value() {
        let mut cci = Cci::new(3);

        cci.update_after_close(p(10.0, 10.0, 10.0));
        cci.update_after_close(p(12.0, 12.0, 12.0));
        assert!(!cci.is_ready());

        cci.update_after_close(p(14.0, 14.0, 14.0));

        match cci.get_last() {
            Some(Value::CciValue(value)) => approx_eq(value, 100.0),
            _ => panic!("missing cci"),
        }
    }

    #[test]
    fn cci_updates_after_warmup() {
        let mut cci = Cci::new(3);

        cci.load(&[
            p(10.0, 10.0, 10.0),
            p(12.0, 12.0, 12.0),
            p(14.0, 14.0, 14.0),
        ]);

        let first = match cci.get_last() {
            Some(Value::CciValue(value)) => value,
            _ => panic!("missing cci"),
        };

        cci.update_after_close(p(15.0, 15.0, 15.0));

        let second = match cci.get_last() {
            Some(Value::CciValue(value)) => value,
            _ => panic!("missing updated cci"),
        };

        assert_ne!(first, second);
    }

    #[test]
    fn cci_before_close_is_provisional() {
        let mut cci = Cci::new(3);

        cci.load(&[
            p(10.0, 10.0, 10.0),
            p(12.0, 12.0, 12.0),
            p(14.0, 14.0, 14.0),
            p(15.0, 15.0, 15.0),
        ]);

        let after_close = match cci.get_last() {
            Some(Value::CciValue(value)) => value,
            _ => panic!("missing cci"),
        };

        cci.update_before_close(p(18.0, 18.0, 18.0));

        let provisional = match cci.get_last() {
            Some(Value::CciValue(value)) => value,
            _ => panic!("missing provisional cci"),
        };

        assert_ne!(after_close, provisional);
    }

    #[test]
    fn cci_reset_clears_state() {
        let mut cci = Cci::new(3);

        cci.load(&[
            p(10.0, 10.0, 10.0),
            p(12.0, 12.0, 12.0),
            p(14.0, 14.0, 14.0),
        ]);
        assert!(cci.is_ready());

        cci.reset();

        assert!(!cci.is_ready());
        assert_eq!(cci.get_last(), None);
        assert!(cci.typical_prices.is_empty());
    }
}
