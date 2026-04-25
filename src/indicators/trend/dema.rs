use crate::ExpMean;
use crate::indicators::{Indicator, Price, Value};

#[derive(Clone, Debug)]
pub struct Dema {
    periods: u32,
    ema1: ExpMean,
    ema2: ExpMean,
    value: Option<f64>,
}

impl Dema {
    pub fn new(periods: u32) -> Self {
        assert!(periods > 1, "DEMA period must be > 1, got {}", periods);
        Self {
            periods,
            ema1: ExpMean::new(periods),
            ema2: ExpMean::new(periods),
            value: None,
        }
    }

    fn update_value(&mut self) {
        self.value = match (self.ema1.get_last(), self.ema2.get_last()) {
            (Some(ema1), Some(ema2)) => Some(2.0 * ema1 - ema2),
            _ => None,
        };
    }
}

impl Indicator for Dema {
    fn update_after_close(&mut self, price: Price) {
        self.ema1.update_after_close(price.close);
        if let Some(ema1) = self.ema1.get_last() {
            self.ema2.update_after_close(ema1);
        }
        self.update_value();
    }

    fn update_before_close(&mut self, price: Price) {
        self.ema1.update_before_close(price.close);
        if let Some(ema1) = self.ema1.get_last() {
            self.ema2.update_before_close(ema1);
        }
        self.update_value();
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
        self.value.map(Value::DemaValue)
    }

    fn reset(&mut self) {
        self.ema1.reset();
        self.ema2.reset();
        self.value = None;
    }

    fn period(&self) -> u32 {
        self.periods
    }
}

impl Default for Dema {
    fn default() -> Self {
        Self::new(20)
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

    fn approx_eq(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "left={a}, right={b}");
    }

    #[test]
    fn dema_warms_up_and_matches_expected_value() {
        let mut dema = Dema::new(3);

        for close in [1.0, 2.0, 3.0, 4.0] {
            dema.update_after_close(p(close));
        }
        assert!(!dema.is_ready());

        dema.update_after_close(p(5.0));

        match dema.get_last() {
            Some(Value::DemaValue(value)) => approx_eq(value, 5.0),
            _ => panic!("missing dema"),
        }
    }

    #[test]
    fn dema_before_close_is_provisional() {
        let mut dema = Dema::new(3);

        for close in [1.0, 2.0, 3.0, 4.0, 5.0, 6.0] {
            dema.update_after_close(p(close));
        }

        let after_close = match dema.get_last() {
            Some(Value::DemaValue(value)) => value,
            _ => panic!("missing dema"),
        };

        dema.update_before_close(p(10.0));

        let provisional = match dema.get_last() {
            Some(Value::DemaValue(value)) => value,
            _ => panic!("missing provisional dema"),
        };

        assert_ne!(after_close, provisional);
    }

    #[test]
    fn dema_reset_clears_state() {
        let mut dema = Dema::new(3);

        for close in [1.0, 2.0, 3.0, 4.0, 5.0] {
            dema.update_after_close(p(close));
        }

        assert!(dema.is_ready());
        dema.reset();

        assert!(!dema.is_ready());
        assert_eq!(dema.get_last(), None);
    }
}
