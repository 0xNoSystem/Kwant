use crate::indicators::{Indicator, Price, Value};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug)]
struct WeightedPoint {
    price: f64,
    volume: f64,
}

#[derive(Clone, Debug)]
pub struct VwapDeviation {
    periods: u32,
    buffer: VecDeque<WeightedPoint>,
    sum_pv: f64,
    sum_p2v: f64,
    sum_v: f64,
    value: Option<f64>,
    in_candle: bool,
}

impl VwapDeviation {
    pub fn new(periods: u32) -> Self {
        assert!(
            periods > 1,
            "VWAP deviation period must be > 1, got {}",
            periods
        );
        Self {
            periods,
            buffer: VecDeque::with_capacity(periods as usize),
            sum_pv: 0.0,
            sum_p2v: 0.0,
            sum_v: 0.0,
            value: None,
            in_candle: true,
        }
    }

    fn remove_point(&mut self, point: WeightedPoint) {
        self.sum_pv -= point.price * point.volume;
        self.sum_p2v -= point.price * point.price * point.volume;
        self.sum_v -= point.volume;
    }

    fn add_point(&mut self, point: WeightedPoint) {
        self.sum_pv += point.price * point.volume;
        self.sum_p2v += point.price * point.price * point.volume;
        self.sum_v += point.volume;
    }

    fn compute(&mut self) {
        if self.buffer.len() != self.periods as usize || self.sum_v <= f64::EPSILON {
            self.value = None;
            return;
        }

        let vwap = self.sum_pv / self.sum_v;
        let variance = (self.sum_p2v / self.sum_v) - (vwap * vwap);
        let stddev = variance.max(0.0).sqrt();

        if stddev <= f64::EPSILON {
            self.value = Some(0.0);
            return;
        }

        let price = self.buffer.back().unwrap().price;
        self.value = Some((price - vwap) / stddev);
    }
}

impl Indicator for VwapDeviation {
    fn update_after_close(&mut self, price: Price) {
        let point = WeightedPoint {
            price: price.close,
            volume: price.vlm,
        };

        if self.buffer.len() == self.periods as usize {
            let expired = if self.in_candle {
                self.buffer.pop_front().unwrap()
            } else {
                self.buffer.pop_back().unwrap()
            };
            self.remove_point(expired);
        }

        self.buffer.push_back(point);
        self.add_point(point);
        self.compute();
        self.in_candle = true;
    }

    fn update_before_close(&mut self, price: Price) {
        if self.buffer.len() != self.periods as usize {
            return;
        }

        let expired = if self.in_candle {
            self.in_candle = false;
            self.buffer.pop_front().unwrap()
        } else {
            self.buffer.pop_back().unwrap()
        };
        self.remove_point(expired);

        let point = WeightedPoint {
            price: price.close,
            volume: price.vlm,
        };
        self.buffer.push_back(point);
        self.add_point(point);
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
        self.value.map(Value::VwapDeviationValue)
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.sum_pv = 0.0;
        self.sum_p2v = 0.0;
        self.sum_v = 0.0;
        self.value = None;
        self.in_candle = true;
    }

    fn period(&self) -> u32 {
        self.periods
    }
}

impl Default for VwapDeviation {
    fn default() -> Self {
        Self::new(20)
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

    fn approx_eq(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "left={a}, right={b}");
    }

    #[test]
    fn vwap_deviation_computes_expected_zscore() {
        let mut indicator = VwapDeviation::new(3);

        indicator.update_after_close(p(10.0, 1.0));
        indicator.update_after_close(p(12.0, 1.0));
        assert!(!indicator.is_ready());

        indicator.update_after_close(p(14.0, 2.0));

        match indicator.get_last() {
            Some(Value::VwapDeviationValue(value)) => approx_eq(value, 0.9045340337332909),
            _ => panic!("missing vwap deviation"),
        }
    }

    #[test]
    fn vwap_deviation_before_close_is_provisional() {
        let mut indicator = VwapDeviation::new(3);

        indicator.update_after_close(p(10.0, 1.0));
        indicator.update_after_close(p(12.0, 1.0));
        indicator.update_after_close(p(14.0, 2.0));

        let after_close = indicator.get_last();
        indicator.update_before_close(p(16.0, 3.0));

        assert_ne!(after_close, indicator.get_last());
    }

    #[test]
    fn vwap_deviation_reset_clears_state() {
        let mut indicator = VwapDeviation::new(3);

        indicator.load(&[p(10.0, 1.0), p(12.0, 1.0), p(14.0, 2.0)]);
        assert!(indicator.is_ready());

        indicator.reset();

        assert!(!indicator.is_ready());
        assert_eq!(indicator.get_last(), None);
    }
}
