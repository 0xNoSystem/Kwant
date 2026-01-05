use crate::Mean;
use crate::indicators::{Indicator, Price, Value};

#[derive(Clone, Debug)]
pub struct Sma {
    periods: u32,
    mean: Mean,
}

impl Sma {
    pub fn new(periods: u32) -> Self {
        assert!(
            periods > 1,
            "Sma  periods field must a positive integer n > 1, {} ",
            periods
        );
        Sma {
            periods,
            mean: Mean::new(periods),
        }
    }
}

impl Indicator for Sma {
    fn update_after_close(&mut self, price: Price) {
        self.mean.update_after_close(price.close);
    }

    fn update_before_close(&mut self, price: Price) {
        self.mean.update_before_close(price.close);
    }

    fn load(&mut self, price_data: &[Price]) {
        for p in price_data {
            self.mean.update_after_close(p.close);
        }
    }

    fn is_ready(&self) -> bool {
        self.mean.is_ready()
    }

    fn get_last(&self) -> Option<Value> {
        self.mean.get_last().map(Value::SmaValue)
    }

    fn reset(&mut self) {
        self.mean.reset();
    }

    fn period(&self) -> u32 {
        self.periods
    }
}

impl Default for Sma {
    fn default() -> Self {
        Sma {
            periods: 9,
            mean: Mean::new(9),
        }
    }
}
