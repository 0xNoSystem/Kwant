use crate::ExpMean;
use crate::indicators::{Indicator, Price, Value};

#[derive(Clone, Debug)]
pub struct Ema {
    core: ExpMean,
    pub value: Option<f64>,
    slope: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct EmaCross {
    pub short: Ema,
    pub long: Ema,
    prev_uptrend: Option<bool>,
}

impl EmaCross {
    pub fn new(period_short: u32, period_long: u32) -> Self {
        EmaCross {
            short: Ema::new(period_short.min(period_long)),
            long: Ema::new(period_short.max(period_long)),
            prev_uptrend: None,
        }
    }

    pub fn check_for_cross(&mut self) -> Option<bool> {
        if !self.is_ready() {
            None
        } else {
            let uptrend = self.get_trend().unwrap();

            if let Some(prev_uptrend) = self.prev_uptrend {
                if uptrend != prev_uptrend {
                    self.prev_uptrend = Some(uptrend);
                    Some(uptrend)
                } else {
                    None
                }
            } else {
                self.prev_uptrend = Some(uptrend);
                None
            }
        }
    }

    pub fn update(&mut self, price: Price, after_close: bool) {
        if after_close {
            self.update_after_close(price);
        } else {
            self.update_before_close(price);
        }

        if self.is_ready() && self.prev_uptrend.is_none() {
            self.prev_uptrend = self.get_trend();
        }
    }

    pub fn update_and_check_for_cross(&mut self, price: Price, after_close: bool) -> Option<bool> {
        self.update(price, after_close);
        self.check_for_cross()
    }

    pub fn get_trend(&self) -> Option<bool> {
        if self.is_ready() {
            Some(self.short.get_last() >= self.long.get_last())
        } else {
            None
        }
    }
}

impl Indicator for EmaCross {
    fn update_after_close(&mut self, price: Price) {
        self.short.update_after_close(price);
        self.long.update_after_close(price);
    }

    fn update_before_close(&mut self, price: Price) {
        self.short.update_before_close(price);
        self.long.update_before_close(price);
    }

    fn is_ready(&self) -> bool {
        self.short.is_ready() && self.long.is_ready()
    }

    fn period(&self) -> u32 {
        self.long.period()
    }

    fn load(&mut self, price_data: &[Price]) {
        for p in price_data {
            self.update_after_close(*p);
        }
    }

    fn reset(&mut self) {
        self.short.reset();
        self.long.reset();
        self.prev_uptrend = None;
    }

    fn get_last(&self) -> Option<Value> {
        if let (Some(sh), Some(lg)) = (self.short.value, self.long.value) {
            return Some(Value::EmaCrossValue {
                short: sh,
                long: lg,
                trend: self.get_trend().unwrap(),
            });
        }

        None
    }
}

impl Default for EmaCross {
    fn default() -> Self {
        EmaCross::new(9, 21)
    }
}

impl Ema {
    pub fn new(periods: u32) -> Self {
        assert!(
            periods > 1,
            "Ema periods field must a positive integer n > 1, {} ",
            periods
        );

        Self {
            core: ExpMean::new(periods),
            value: None,
            slope: None,
        }
    }

    pub fn get_slope(&self) -> Option<f64> {
        self.slope
    }

    fn sync_value(&mut self) {
        self.value = self.core.get_last();
    }

    fn update_slope(&mut self, last_confirmed: Option<f64>) {
        self.slope = match (last_confirmed, self.value) {
            (Some(previous), Some(current)) => Some(((current - previous) / previous) * 100.0),
            _ => None,
        };
    }
}

impl Indicator for Ema {
    fn update_after_close(&mut self, price: Price) {
        let last_confirmed = self.core.get_confirmed();
        self.core.update_after_close(price.close);
        self.sync_value();
        self.update_slope(last_confirmed);
    }

    fn update_before_close(&mut self, price: Price) {
        let last_confirmed = self.core.get_confirmed();
        self.core.update_before_close(price.close);
        self.sync_value();
        self.update_slope(last_confirmed);
    }

    fn get_last(&self) -> Option<Value> {
        self.value.map(Value::EmaValue)
    }

    fn is_ready(&self) -> bool {
        self.value.is_some()
    }

    fn load(&mut self, price_data: &[Price]) {
        for p in price_data {
            self.update_after_close(*p);
        }
    }

    fn reset(&mut self) {
        self.core.reset();
        self.value = None;
        self.slope = None;
    }

    fn period(&self) -> u32 {
        self.core.period()
    }
}

impl Default for Ema {
    fn default() -> Self {
        Self {
            core: ExpMean::new(9),
            value: None,
            slope: None,
        }
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
    fn ema_seeds_from_sma_then_updates() {
        let mut ema = Ema::new(3);

        ema.update_after_close(p(10.0));
        ema.update_after_close(p(11.0));
        assert!(!ema.is_ready());

        ema.update_after_close(p(12.0));
        approx_eq(ema.value.unwrap(), 11.0);

        ema.update_after_close(p(13.0));
        approx_eq(ema.value.unwrap(), 12.0);
        approx_eq(ema.get_slope().unwrap(), (1.0 / 11.0) * 100.0);
    }

    #[test]
    fn ema_after_close_uses_last_confirmed_not_provisional() {
        let mut ema = Ema::new(3);

        ema.load(&[p(10.0), p(11.0), p(12.0)]);
        approx_eq(ema.value.unwrap(), 11.0);

        ema.update_before_close(p(13.0));
        approx_eq(ema.value.unwrap(), 12.0);

        ema.update_after_close(p(14.0));
        approx_eq(ema.value.unwrap(), 12.5);
        approx_eq(ema.get_slope().unwrap(), ((12.5 - 11.0) / 11.0) * 100.0);
    }
}
