use crate::Mean;
use crate::indicators::{Indicator, Price, Value};

#[derive(Clone, Debug)]
pub struct BollingerBands {
    periods: u32,
    std_multiplier: f64,
    mean: Mean,
}

impl BollingerBands {
    pub fn new(periods: u32, std_multiplier: f64) -> Self {
        assert!(
            periods > 1,
            "Bollinger periods must be > 1, got {}",
            periods
        );
        assert!(
            std_multiplier.is_finite() && std_multiplier > 0.0,
            "Bollinger std multiplier must be finite and > 0, got {}",
            std_multiplier
        );

        Self {
            periods,
            std_multiplier,
            mean: Mean::new(periods),
        }
    }

    fn width(upper: f64, lower: f64, mid: f64) -> f64 {
        if mid.abs() <= f64::EPSILON {
            0.0
        } else {
            ((upper - lower) / mid.abs()) * 100.0
        }
    }

    fn get_bands(&self) -> Option<(f64, f64, f64, f64)> {
        let mid = self.mean.get_last()?;
        let n = self.periods as f64;
        let sum = self.mean.sum();
        let sum_sq = self.mean.sum_sq();
        let variance = (sum_sq / n) - (sum / n).powi(2);
        let stddev = variance.max(0.0).sqrt();
        let offset = stddev * self.std_multiplier;
        let upper = mid + offset;
        let lower = mid - offset;
        let width = Self::width(upper, lower, mid);

        Some((upper, mid, lower, width))
    }
}

impl Indicator for BollingerBands {
    fn update_after_close(&mut self, price: Price) {
        self.mean.update_after_close(price.close);
    }

    fn update_before_close(&mut self, price: Price) {
        self.mean.update_before_close(price.close);
    }

    fn load(&mut self, price_data: &[Price]) {
        for price in price_data {
            self.update_after_close(*price);
        }
    }

    fn is_ready(&self) -> bool {
        self.mean.is_ready()
    }

    fn get_last(&self) -> Option<Value> {
        self.get_bands()
            .map(|(upper, mid, lower, width)| Value::BollingerValue {
                upper,
                mid,
                lower,
                width,
            })
    }

    fn reset(&mut self) {
        self.mean.reset();
    }

    fn period(&self) -> u32 {
        self.periods
    }
}

impl Default for BollingerBands {
    fn default() -> Self {
        Self::new(20, 2.0)
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
    fn bollinger_computes_expected_bands() {
        let mut bands = BollingerBands::new(3, 2.0);

        bands.update_after_close(p(10.0));
        bands.update_after_close(p(12.0));
        assert!(!bands.is_ready());

        bands.update_after_close(p(14.0));

        let (upper, mid, lower, width) = match bands.get_last() {
            Some(Value::BollingerValue {
                upper,
                mid,
                lower,
                width,
            }) => (upper, mid, lower, width),
            _ => panic!("missing bollinger bands"),
        };

        let expected_stddev = (8.0_f64 / 3.0).sqrt();
        let expected_offset = expected_stddev * 2.0;

        approx_eq(upper, 12.0 + expected_offset);
        approx_eq(mid, 12.0);
        approx_eq(lower, 12.0 - expected_offset);
        approx_eq(width, (expected_offset * 2.0 / 12.0) * 100.0);
    }

    #[test]
    fn bollinger_updates_after_warmup() {
        let mut bands = BollingerBands::new(3, 2.0);

        for close in [10.0, 12.0, 14.0] {
            bands.update_after_close(p(close));
        }

        let first = match bands.get_last() {
            Some(Value::BollingerValue { width, .. }) => width,
            _ => panic!("missing bands"),
        };

        bands.update_after_close(p(16.0));

        let second = match bands.get_last() {
            Some(Value::BollingerValue { width, .. }) => width,
            _ => panic!("missing updated bands"),
        };

        assert_ne!(first, second);
    }

    #[test]
    fn bollinger_before_close_is_provisional() {
        let mut bands = BollingerBands::new(3, 2.0);

        for close in [10.0, 12.0, 14.0, 16.0] {
            bands.update_after_close(p(close));
        }

        let after_close = match bands.get_last() {
            Some(Value::BollingerValue { upper, .. }) => upper,
            _ => panic!("missing bands"),
        };

        bands.update_before_close(p(20.0));

        let provisional = match bands.get_last() {
            Some(Value::BollingerValue { upper, .. }) => upper,
            _ => panic!("missing provisional bands"),
        };

        assert_ne!(after_close, provisional);
    }

    #[test]
    fn bollinger_reset_clears_state() {
        let mut bands = BollingerBands::new(3, 2.0);

        for close in [10.0, 12.0, 14.0] {
            bands.update_after_close(p(close));
        }

        assert!(bands.is_ready());

        bands.reset();

        assert!(!bands.is_ready());
        assert_eq!(bands.get_last(), None);
    }
}
