use crate::indicators::Indicator;
use crate::indicators::Price;


#[derive(Clone, Debug)]
pub struct Atr {
    periods: u32,
    value: Option<f32>,
    prev_value: Option<f32>,
    prev_close: Option<f32>,
    warmup_trs: Vec<f32>,
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

    pub fn normalized(&self, price: f32) -> Option<f32> {
    if price.abs() < f32::EPSILON {
        return None;
    }
    self.value.map(|value| (value / price) * 100.0)
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
            let sum: f32 = self.warmup_trs.iter().sum();
            let initial_atr = sum / self.periods as f32;
            self.value = Some(initial_atr);
            self.prev_value = Some(initial_atr);
        }
    } else if let Some(prev_atr) = self.value {
        let new_atr = (prev_atr * (self.periods as f32 - 1.0) + tr) / self.periods as f32;
        self.value = Some(new_atr);
        self.prev_value = Some(new_atr);
    }

    self.prev_close = Some(close);
}



    fn update_before_close(&mut self, price: Price) {
    if let (Some(prev_close), Some(prev_atr)) = (self.prev_close, self.prev_value) {
        let tr = calc_tr(price.high, price.low, prev_close);
        let provisional_atr = (prev_atr * (self.periods as f32 - 1.0) + tr) / self.periods as f32;
        self.value = Some(provisional_atr);
    }
}

    fn get_last(&self) -> Option<f32>{
        self.value
    }
    fn load<'a,I: IntoIterator<Item=&'a Price>>(&mut self, price_data: I){
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


fn calc_tr(high: f32, low: f32, prev_close: f32) -> f32{

        f32::max(high - low, f32::max((high - prev_close).abs(), (low - prev_close).abs()))
    }


impl Default for Atr {
    fn default() -> Self {
        Atr::new(14)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::{Atr, Price};

    #[test]
    fn test_atr_update_after_close() {
        let mut atr = Atr::new(3);

        let prices = vec![
            Price { high: 10.0, low: 5.0, close: 8.0, open: 5.0 },
            Price { high: 12.0, low: 6.0, close: 10.0, open: 8.0 },
            Price { high: 14.0, low: 7.0, close: 11.0, open: 10.0 },
        ];

        for price in prices {
            atr.update_after_close(price);
        }   

        assert!(atr.is_ready());
        assert_eq!(atr.get_last(), Some(6.0));

        atr.update_before_close(Price{high: 11.1, low: 10.04, close: 10.9, open:11.0});
        
     }
    }



