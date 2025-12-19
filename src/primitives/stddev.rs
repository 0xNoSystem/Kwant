use super::Mean;
use crate::indicators::{Indicator, Price, Value};

#[derive(Clone, Debug)]
pub struct StdDev {
    periods: u32,
    mean: Mean,
    value: Option<f64>,
}

impl StdDev {
    pub fn new(periods: u32) -> Self {
        assert!(periods > 1);
        Self {
            periods,
            mean: Mean::new(periods),
            value: None,
        }
    }

    fn compute(&mut self) {
        if !self.mean.is_ready() {
            self.value = None;
            return;
        }

        let n = self.periods as f64;
        let sum = self.mean.sum();
        let sum_sq = self.mean.sum_sq();

        let var = (sum_sq - (sum * sum) / n) / (n - 1.0);
        self.value = Some(var.max(0.0).sqrt());
    }
}

impl Indicator for StdDev {
    fn update_before_close(&mut self, price: Price) {
        self.mean.update_before_close(price.close);
        self.compute();
    }

    fn update_after_close(&mut self, price: Price) {
        self.mean.update_after_close(price.close);
        self.compute();
    }

    fn load(&mut self, price_data: &[Price]) {
        for p in price_data.iter(){
            self.mean.update_after_close(p.close);
        }
        self.compute();
    }

    fn reset(&mut self) {
        self.mean.reset();
        self.value = None;
    }

    fn is_ready(&self) -> bool {
        self.mean.is_ready()
    }

    fn get_last(&self) -> Option<Value> {
        self.value.map(Value::StdDevValue)
    }

    fn period(&self) -> u32 {
        self.periods
    }
}

