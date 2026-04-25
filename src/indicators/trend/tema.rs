use crate::ExpMean;
use crate::indicators::{Indicator, Price, Value};

#[derive(Clone, Debug)]
pub struct Tema {
    periods: u32,
    ema1: ExpMean,
    ema2: ExpMean,
    ema3: ExpMean,
    value: Option<f64>,
}

impl Tema {
    pub fn new(periods: u32) -> Self {
        assert!(periods > 1, "TEMA period must be > 1, got {}", periods);
        Self {
            periods,
            ema1: ExpMean::new(periods),
            ema2: ExpMean::new(periods),
            ema3: ExpMean::new(periods),
            value: None,
        }
    }

    fn update_value(&mut self) {
        self.value = match (
            self.ema1.get_last(),
            self.ema2.get_last(),
            self.ema3.get_last(),
        ) {
            (Some(ema1), Some(ema2), Some(ema3)) => Some(3.0 * ema1 - 3.0 * ema2 + ema3),
            _ => None,
        };
    }
}

impl Indicator for Tema {
    fn update_after_close(&mut self, price: Price) {
        self.ema1.update_after_close(price.close);
        if let Some(ema1) = self.ema1.get_last() {
            self.ema2.update_after_close(ema1);
        }
        if let Some(ema2) = self.ema2.get_last() {
            self.ema3.update_after_close(ema2);
        }
        self.update_value();
    }

    fn update_before_close(&mut self, price: Price) {
        self.ema1.update_before_close(price.close);
        if let Some(ema1) = self.ema1.get_last() {
            self.ema2.update_before_close(ema1);
        }
        if let Some(ema2) = self.ema2.get_last() {
            self.ema3.update_before_close(ema2);
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
        self.value.map(Value::TemaValue)
    }

    fn reset(&mut self) {
        self.ema1.reset();
        self.ema2.reset();
        self.ema3.reset();
        self.value = None;
    }

    fn period(&self) -> u32 {
        self.periods
    }
}

impl Default for Tema {
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
    fn tema_warms_up_and_matches_expected_value() {
        let mut tema = Tema::new(3);

        for close in [1.0, 2.0, 3.0, 4.0, 5.0, 6.0] {
            tema.update_after_close(p(close));
        }
        assert!(!tema.is_ready());

        tema.update_after_close(p(7.0));

        match tema.get_last() {
            Some(Value::TemaValue(value)) => approx_eq(value, 7.0),
            _ => panic!("missing tema"),
        }
    }

    #[test]
    fn tema_before_close_is_provisional() {
        let mut tema = Tema::new(3);

        for close in [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0] {
            tema.update_after_close(p(close));
        }

        let after_close = match tema.get_last() {
            Some(Value::TemaValue(value)) => value,
            _ => panic!("missing tema"),
        };

        tema.update_before_close(p(12.0));

        let provisional = match tema.get_last() {
            Some(Value::TemaValue(value)) => value,
            _ => panic!("missing provisional tema"),
        };

        assert_ne!(after_close, provisional);
    }

    #[test]
    fn tema_reset_clears_state() {
        let mut tema = Tema::new(3);

        for close in [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0] {
            tema.update_after_close(p(close));
        }

        assert!(tema.is_ready());
        tema.reset();

        assert!(!tema.is_ready());
        assert_eq!(tema.get_last(), None);
    }
}
