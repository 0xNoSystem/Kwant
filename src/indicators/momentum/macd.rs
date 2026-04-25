use crate::ExpMean;
use crate::indicators::{Indicator, Price, Value};

#[derive(Clone, Debug)]
pub struct Macd {
    slow_period: u32,
    fast_ema: ExpMean,
    slow_ema: ExpMean,
    signal_ema: ExpMean,
    macd: Option<f64>,
    signal: Option<f64>,
    histogram: Option<f64>,
}

impl Macd {
    pub fn new(fast_period: u32, slow_period: u32, signal_period: u32) -> Self {
        assert!(
            fast_period > 1,
            "MACD fast period must be > 1, got {}",
            fast_period
        );
        assert!(
            slow_period > 1,
            "MACD slow period must be > 1, got {}",
            slow_period
        );
        assert!(
            signal_period > 1,
            "MACD signal period must be > 1, got {}",
            signal_period
        );

        let fast = fast_period.min(slow_period);
        let slow = fast_period.max(slow_period);

        Self {
            slow_period: slow,
            fast_ema: ExpMean::new(fast),
            slow_ema: ExpMean::new(slow),
            signal_ema: ExpMean::new(signal_period),
            macd: None,
            signal: None,
            histogram: None,
        }
    }

    fn clear_value(&mut self) {
        self.macd = None;
        self.signal = None;
        self.histogram = None;
    }

    fn update_value(&mut self, macd: f64) {
        if let Some(signal) = self.signal_ema.get_last() {
            self.macd = Some(macd);
            self.signal = Some(signal);
            self.histogram = Some(macd - signal);
        } else {
            self.clear_value();
        }
    }
}

impl Indicator for Macd {
    fn update_after_close(&mut self, price: Price) {
        self.fast_ema.update_after_close(price.close);
        self.slow_ema.update_after_close(price.close);

        if let (Some(fast), Some(slow)) = (self.fast_ema.get_last(), self.slow_ema.get_last()) {
            let macd = fast - slow;
            self.signal_ema.update_after_close(macd);
            self.update_value(macd);
        } else {
            self.clear_value();
        }
    }

    fn update_before_close(&mut self, price: Price) {
        self.fast_ema.update_before_close(price.close);
        self.slow_ema.update_before_close(price.close);

        if let (Some(fast), Some(slow)) = (self.fast_ema.get_last(), self.slow_ema.get_last()) {
            let macd = fast - slow;
            self.signal_ema.update_before_close(macd);
            self.update_value(macd);
        }
    }

    fn load(&mut self, price_data: &[Price]) {
        for price in price_data {
            self.update_after_close(*price);
        }
    }

    fn is_ready(&self) -> bool {
        self.macd.is_some() && self.signal.is_some() && self.histogram.is_some()
    }

    fn get_last(&self) -> Option<Value> {
        match (self.macd, self.signal, self.histogram) {
            (Some(macd), Some(signal), Some(histogram)) => Some(Value::MacdValue {
                macd,
                signal,
                histogram,
            }),
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
        self.clear_value();
    }

    fn period(&self) -> u32 {
        self.slow_period
    }
}

impl Default for Macd {
    fn default() -> Self {
        Self::new(12, 26, 9)
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
    fn macd_not_ready_until_signal_ema_warms_up() {
        let mut macd = Macd::new(3, 5, 3);

        for close in [1.0, 2.0, 3.0, 4.0, 5.0, 6.0] {
            macd.update_after_close(p(close));
        }

        assert!(!macd.is_ready());

        macd.update_after_close(p(7.0));

        assert!(macd.is_ready());
        assert!(matches!(macd.get_last(), Some(Value::MacdValue { .. })));
    }

    #[test]
    fn macd_updates_after_warmup() {
        let mut macd = Macd::new(3, 5, 3);

        for close in [1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0] {
            macd.update_after_close(p(close));
        }

        let first = match macd.get_last() {
            Some(Value::MacdValue { macd, .. }) => macd,
            _ => panic!("missing macd"),
        };

        macd.update_after_close(p(34.0));

        let second = match macd.get_last() {
            Some(Value::MacdValue { macd, .. }) => macd,
            _ => panic!("missing updated macd"),
        };

        assert_ne!(first, second);
    }

    #[test]
    fn macd_before_close_is_provisional() {
        let mut macd = Macd::new(3, 5, 3);

        for close in [1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0, 34.0] {
            macd.update_after_close(p(close));
        }

        let after_close = match macd.get_last() {
            Some(Value::MacdValue { macd, .. }) => macd,
            _ => panic!("missing macd"),
        };

        macd.update_before_close(p(55.0));

        let provisional = match macd.get_last() {
            Some(Value::MacdValue { macd, .. }) => macd,
            _ => panic!("missing provisional macd"),
        };

        assert_ne!(after_close, provisional);
    }

    #[test]
    fn macd_reset_clears_state() {
        let mut macd = Macd::new(3, 5, 3);

        for close in [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0] {
            macd.update_after_close(p(close));
        }

        assert!(macd.is_ready());

        macd.reset();

        assert!(!macd.is_ready());
        assert_eq!(macd.get_last(), None);
        assert_eq!(macd.macd, None);
        assert_eq!(macd.signal, None);
        assert_eq!(macd.histogram, None);
    }
}
