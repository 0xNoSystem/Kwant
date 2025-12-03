use crate::indicators::{Value,Price, Indicator};


#[derive(Clone, Debug)]
pub struct Atr {
    periods: u32,
    value: Option<f64>,
    prev_value: Option<f64>,
    prev_close: Option<f64>,
    warmup_trs: Vec<f64>,
}


impl Atr{

    pub fn new(periods: u32) -> Self{

        assert!(periods > 0, "Atr periods must be a periods > 0, ({})", periods);
        Atr{
            periods,
            prev_close: None,
            warmup_trs: Vec::with_capacity(periods as usize),
            prev_value: None,
            value: None,
        }
    }

    pub fn normalized(&self, price: f64) -> Option<Value> {
    if price.abs() < f64::EPSILON {
        return None;
    }
    self.value.map(|value| Value::AtrValue((value / price) * 100.0))
    }
}



impl Indicator for Atr{

    fn period(&self) -> u32{
        self.periods
    }

    fn update_after_close(&mut self, price: Price) {
    let high = price.high;
    let low = price.low;
    let close = price.close;

    let tr = if let Some(prev_close) = self.prev_close {
        calc_tr(high, low, prev_close)
    } else {
        high - low
    };

    if self.value.is_none() {
        self.warmup_trs.push(tr);
        if self.warmup_trs.len() == self.periods as usize {
            let sum: f64 = self.warmup_trs.iter().sum();
            let initial_atr = sum / self.periods as f64;
            self.value = Some(initial_atr);
            self.prev_value = Some(initial_atr);
        }
    } else if let Some(prev_atr) = self.value {
        let new_atr = (prev_atr * (self.periods as f64 - 1.0) + tr) / self.periods as f64;
        self.value = Some(new_atr);
        self.prev_value = Some(new_atr);
    }

    self.prev_close = Some(close);
}



    fn update_before_close(&mut self, price: Price) {
    if let (Some(prev_close), Some(prev_atr)) = (self.prev_close, self.prev_value) {
        let tr = calc_tr(price.high, price.low, prev_close);
        let provisional_atr = (prev_atr * (self.periods as f64 - 1.0) + tr) / self.periods as f64;
        self.value = Some(provisional_atr);
    }
}

    fn get_last(&self) -> Option<Value>{
        self.value.map(|value| Value::AtrValue(value))
    }
    fn load(&mut self, price_data: &[Price]){
        for p in price_data{
            self.update_after_close(*p);
        }
    }
 
    fn reset(&mut self) {
        self.value = None;
        self.prev_close = None;
        self.prev_value = None;
        self.warmup_trs.clear();
    }

    fn is_ready(&self) -> bool {
        self.value.is_some()
    }

    }


fn calc_tr(high: f64, low: f64, prev_close: f64) -> f64{

        f64::max(high - low, f64::max((high - prev_close).abs(), (low - prev_close).abs()))
    }


impl Default for Atr {
    fn default() -> Self {
        Atr::new(14)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::{Atr, Price, Value};

    fn p(h: f64, l: f64, c: f64) -> Price {
        Price { high: h, low: l, close: c, open: l }
    }

    #[test]
    fn test_atr_warmup_and_initial_value() {
        let mut atr = Atr::new(3);

        atr.update_after_close(p(10.0, 5.0, 8.0));
        assert!(!atr.is_ready());

        atr.update_after_close(p(12.0, 6.0, 10.0));
        assert!(!atr.is_ready());

        atr.update_after_close(p(14.0, 7.0, 11.0));
        assert!(atr.is_ready());
        assert_eq!(atr.get_last(), Some(Value::AtrValue(6.0)));
    }

    #[test]
    fn test_atr_updates_after_warmup() {
        let mut atr = Atr::new(3);

        atr.update_after_close(p(10.0, 5.0, 8.0));
        atr.update_after_close(p(12.0, 6.0, 10.0));
        atr.update_after_close(p(14.0, 7.0, 11.0));

        atr.update_after_close(p(16.0, 8.0, 15.0));
        let tr = calc_tr(16.0, 8.0, 11.0);
        let expected = (6.0 * 2.0 + tr) / 3.0;

        assert_eq!(atr.get_last(), Some(Value::AtrValue(expected)));
    }

    #[test]
    fn test_update_before_close_provisional() {
        let mut atr = Atr::new(3);

        atr.update_after_close(p(10.0, 5.0, 8.0));
        atr.update_after_close(p(12.0, 6.0, 10.0));
        atr.update_after_close(p(14.0, 7.0, 11.0));

        atr.update_before_close(p(15.0, 13.0, 14.0));

        if let Some(Value::AtrValue(v)) = atr.get_last() {
            assert!(v > 0.0);
        } else {
            panic!("ATR not set");
        }
    }

    #[test]
    fn test_reset() {
        let mut atr = Atr::new(3);

        atr.update_after_close(p(10.0, 5.0, 8.0));
        atr.update_after_close(p(12.0, 6.0, 10.0));
        atr.update_after_close(p(14.0, 7.0, 11.0));

        assert!(atr.is_ready());

        atr.reset();

        assert!(!atr.is_ready());
        assert_eq!(atr.get_last(), None);
        assert_eq!(atr.prev_close, None);
        assert_eq!(atr.prev_value, None);
        assert!(atr.warmup_trs.is_empty());
    }
}

