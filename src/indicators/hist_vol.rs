use crate::StdDev;
use crate::indicators::{Indicator, Price, Value};

#[derive(Clone, Debug)]
pub struct HistVolatility {
    periods: u32,
    prev_close: Option<f64>,
    stddev: StdDev,
    value: Option<f64>,
}

impl HistVolatility {
    const ANNUALIZATION_DAYS: f64 = 365.0;

    pub fn new(periods: u32) -> Self {
        assert!(periods > 1, "HV periods must be > 1");
        Self {
            periods,
            prev_close: None,
            stddev: StdDev::new(periods),
            value: None,
        }
    }

    fn update_value(&mut self) {
        self.value = self
            .stddev
            .get_last_value()
            .map(|v| v * Self::ANNUALIZATION_DAYS.sqrt() * 100.0);
    }
}

impl Indicator for HistVolatility {
    fn update_after_close(&mut self, price: Price) {
        if let Some(prev) = self.prev_close {
            let r = (price.close / prev).ln();
            self.stddev.update_after_close_value(r);
            self.update_value();
        }
        self.prev_close = Some(price.close);
    }

    fn update_before_close(&mut self, price: Price) {
        if let Some(prev) = self.prev_close {
            if self.stddev.is_ready() {
                let provisional = (price.close / prev).ln();
                self.stddev.update_before_close_value(provisional);
                self.update_value();
            }
        }
    }

    fn get_last(&self) -> Option<Value> {
        self.value.map(Value::HistVolatilityValue)
    }

    fn is_ready(&self) -> bool {
        self.stddev.is_ready()
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.stddev.reset();
        self.value = None;
    }

    fn load(&mut self, price_data: &[Price]) {
        for p in price_data {
            self.update_after_close(*p);
        }
    }

    fn period(&self) -> u32 {
        self.periods
    }
}

impl Default for HistVolatility {
    fn default() -> Self {
        Self::new(20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::{Indicator, Price, Value};

    fn p(c: f64) -> Price {
        Price {
            open: c,
            high: c,
            low: c,
            close: c,
            open_time: 0,
            close_time: 0,
            vlm: 0.0,
        }
    }

    #[test]
    fn hv_not_ready_until_window_full() {
        let mut hv = HistVolatility::new(3);

        hv.update_after_close(p(100.0));
        assert!(!hv.is_ready());

        hv.update_after_close(p(101.0));
        assert!(!hv.is_ready());

        hv.update_after_close(p(102.0));
        assert!(!hv.is_ready());

        hv.update_after_close(p(103.0));
        assert!(hv.is_ready());
    }

    #[test]
    fn hv_becomes_ready_when_window_full() {
        let mut hv = HistVolatility::new(3);

        hv.update_after_close(p(100.0));
        hv.update_after_close(p(101.0));
        hv.update_after_close(p(102.0));
        hv.update_after_close(p(103.0));

        assert!(hv.is_ready());

        match hv.get_last() {
            Some(Value::HistVolatilityValue(v)) => assert!(v > 0.0),
            _ => panic!("HV not computed"),
        }
    }

    #[test]
    fn hv_updates_on_new_close() {
        let mut hv = HistVolatility::new(3);

        hv.update_after_close(p(100.0));
        hv.update_after_close(p(101.0));
        hv.update_after_close(p(102.0));
        hv.update_after_close(p(103.0));

        let first = match hv.get_last() {
            Some(Value::HistVolatilityValue(v)) => v,
            _ => panic!("missing hv"),
        };

        hv.update_after_close(p(104.0));

        let second = match hv.get_last() {
            Some(Value::HistVolatilityValue(v)) => v,
            _ => panic!("missing hv"),
        };

        assert_ne!(first, second);
    }

    #[test]
    fn hv_before_close_is_provisional() {
        let mut hv = HistVolatility::new(3);

        hv.update_after_close(p(100.0));
        hv.update_after_close(p(101.0));
        hv.update_after_close(p(102.0));
        hv.update_after_close(p(103.0));

        let after_close = match hv.get_last() {
            Some(Value::HistVolatilityValue(v)) => v,
            _ => panic!("missing hv"),
        };

        hv.update_before_close(p(110.0));

        let provisional = match hv.get_last() {
            Some(Value::HistVolatilityValue(v)) => v,
            _ => panic!("missing provisional hv"),
        };

        assert_ne!(after_close, provisional);
    }

    #[test]
    fn hv_reset_clears_state() {
        let mut hv = HistVolatility::new(3);

        hv.update_after_close(p(100.0));
        hv.update_after_close(p(101.0));
        hv.update_after_close(p(102.0));
        hv.update_after_close(p(103.0));

        assert!(hv.is_ready());

        hv.reset();

        assert!(!hv.is_ready());
        assert_eq!(hv.get_last(), None);
        assert_eq!(hv.prev_close, None);
    }
}
