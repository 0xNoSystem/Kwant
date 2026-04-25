use crate::indicators::{Indicator, Price, Value};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
struct MidpointWindow {
    period: u32,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
    max_highs: VecDeque<f64>,
    min_lows: VecDeque<f64>,
    in_candle: bool,
}

impl MidpointWindow {
    fn new(period: u32) -> Self {
        assert!(
            period > 0,
            "Ichimoku window period must be > 0, got {}",
            period
        );
        Self {
            period,
            highs: VecDeque::with_capacity(period as usize),
            lows: VecDeque::with_capacity(period as usize),
            max_highs: VecDeque::with_capacity(period as usize),
            min_lows: VecDeque::with_capacity(period as usize),
            in_candle: true,
        }
    }

    fn push_value(&mut self, high: f64, low: f64) {
        while let Some(&last) = self.max_highs.back() {
            if last < high {
                self.max_highs.pop_back();
            } else {
                break;
            }
        }
        self.max_highs.push_back(high);

        while let Some(&last) = self.min_lows.back() {
            if last > low {
                self.min_lows.pop_back();
            } else {
                break;
            }
        }
        self.min_lows.push_back(low);

        self.highs.push_back(high);
        self.lows.push_back(low);
    }

    fn remove_front_value(&mut self) {
        let expired_high = self.highs.pop_front().unwrap();
        let expired_low = self.lows.pop_front().unwrap();

        if self
            .max_highs
            .front()
            .is_some_and(|&value| value == expired_high)
        {
            self.max_highs.pop_front();
        }
        if self
            .min_lows
            .front()
            .is_some_and(|&value| value == expired_low)
        {
            self.min_lows.pop_front();
        }
    }

    fn remove_back_value(&mut self) {
        let expired_high = self.highs.pop_back().unwrap();
        let expired_low = self.lows.pop_back().unwrap();

        if self
            .max_highs
            .back()
            .is_some_and(|&value| value == expired_high)
        {
            self.max_highs.pop_back();
        }
        if self
            .min_lows
            .back()
            .is_some_and(|&value| value == expired_low)
        {
            self.min_lows.pop_back();
        }
    }

    fn update_after_close(&mut self, high: f64, low: f64) {
        if self.highs.len() == self.period as usize {
            if self.in_candle {
                self.remove_front_value();
            } else {
                self.remove_back_value();
            }
        }

        self.push_value(high, low);
        self.in_candle = true;
    }

    fn update_before_close(&mut self, high: f64, low: f64) {
        if self.highs.len() != self.period as usize {
            return;
        }

        if self.in_candle {
            self.remove_front_value();
            self.in_candle = false;
        } else {
            self.remove_back_value();
        }

        self.push_value(high, low);
    }

    fn midpoint(&self) -> Option<f64> {
        if self.highs.len() != self.period as usize {
            return None;
        }

        let high = *self.max_highs.front()?;
        let low = *self.min_lows.front()?;
        Some((high + low) / 2.0)
    }

    fn reset(&mut self) {
        self.highs.clear();
        self.lows.clear();
        self.max_highs.clear();
        self.min_lows.clear();
        self.in_candle = true;
    }
}

#[derive(Clone, Debug)]
pub struct Ichimoku {
    senkou_b_period: u32,
    tenkan_window: MidpointWindow,
    kijun_window: MidpointWindow,
    senkou_b_window: MidpointWindow,
    chikou: Option<f64>,
    value: Option<Value>,
}

impl Ichimoku {
    pub fn new(tenkan_period: u32, kijun_period: u32, senkou_b_period: u32) -> Self {
        assert!(tenkan_period > 0, "Ichimoku tenkan period must be > 0");
        assert!(kijun_period > 0, "Ichimoku kijun period must be > 0");
        assert!(senkou_b_period > 0, "Ichimoku senkou_b period must be > 0");

        Self {
            senkou_b_period,
            tenkan_window: MidpointWindow::new(tenkan_period),
            kijun_window: MidpointWindow::new(kijun_period),
            senkou_b_window: MidpointWindow::new(senkou_b_period),
            chikou: None,
            value: None,
        }
    }

    fn lines(&self) -> Option<(f64, f64, f64, f64, f64)> {
        let tenkan = self.tenkan_window.midpoint()?;
        let kijun = self.kijun_window.midpoint()?;
        let span_b = self.senkou_b_window.midpoint()?;
        let chikou = self.chikou?;
        let span_a = (tenkan + kijun) / 2.0;

        Some((tenkan, kijun, span_a, span_b, chikou))
    }
}

impl Indicator for Ichimoku {
    fn update_after_close(&mut self, price: Price) {
        self.tenkan_window.update_after_close(price.high, price.low);
        self.kijun_window.update_after_close(price.high, price.low);
        self.senkou_b_window
            .update_after_close(price.high, price.low);
        self.chikou = Some(price.close);
        self.value =
            self.lines().map(
                |(tenkan, kijun, span_a, span_b, chikou)| Value::IchimokuValue {
                    tenkan,
                    kijun,
                    span_a,
                    span_b,
                    chikou,
                },
            );
    }

    fn update_before_close(&mut self, price: Price) {
        self.tenkan_window
            .update_before_close(price.high, price.low);
        self.kijun_window.update_before_close(price.high, price.low);
        self.senkou_b_window
            .update_before_close(price.high, price.low);
        if self.chikou.is_some() {
            self.chikou = Some(price.close);
        }
        self.value =
            self.lines().map(
                |(tenkan, kijun, span_a, span_b, chikou)| Value::IchimokuValue {
                    tenkan,
                    kijun,
                    span_a,
                    span_b,
                    chikou,
                },
            );
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
        self.value
    }

    fn reset(&mut self) {
        self.tenkan_window.reset();
        self.kijun_window.reset();
        self.senkou_b_window.reset();
        self.chikou = None;
        self.value = None;
    }

    fn period(&self) -> u32 {
        self.senkou_b_period
    }
}

impl Default for Ichimoku {
    fn default() -> Self {
        Self::new(9, 26, 52)
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
    fn ichimoku_warms_up_and_computes_lines() {
        let mut indicator = Ichimoku::new(2, 3, 4);

        indicator.update_after_close(p(1.0, 1.0, 1.0));
        indicator.update_after_close(p(2.0, 2.0, 2.0));
        indicator.update_after_close(p(3.0, 3.0, 3.0));
        assert!(!indicator.is_ready());

        indicator.update_after_close(p(4.0, 4.0, 4.0));

        match indicator.get_last() {
            Some(Value::IchimokuValue {
                tenkan,
                kijun,
                span_a,
                span_b,
                chikou,
            }) => {
                approx_eq(tenkan, 3.5);
                approx_eq(kijun, 3.0);
                approx_eq(span_a, 3.25);
                approx_eq(span_b, 2.5);
                approx_eq(chikou, 4.0);
            }
            _ => panic!("missing ichimoku"),
        }
    }

    #[test]
    fn ichimoku_before_close_is_provisional() {
        let mut indicator = Ichimoku::new(2, 3, 4);

        indicator.load(&[
            p(1.0, 1.0, 1.0),
            p(2.0, 2.0, 2.0),
            p(3.0, 3.0, 3.0),
            p(4.0, 4.0, 4.0),
            p(5.0, 5.0, 5.0),
        ]);

        let after_close = indicator.get_last();
        indicator.update_before_close(p(10.0, 10.0, 10.0));

        assert_ne!(after_close, indicator.get_last());
    }

    #[test]
    fn ichimoku_reset_clears_state() {
        let mut indicator = Ichimoku::new(2, 3, 4);

        indicator.load(&[
            p(1.0, 1.0, 1.0),
            p(2.0, 2.0, 2.0),
            p(3.0, 3.0, 3.0),
            p(4.0, 4.0, 4.0),
        ]);
        assert!(indicator.is_ready());

        indicator.reset();

        assert!(!indicator.is_ready());
        assert_eq!(indicator.get_last(), None);
    }
}
