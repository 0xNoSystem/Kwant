use std::collections::VecDeque;
use crate::indicators::Price;
use crate::indicators::Indicator;

pub struct Atr{            
    pub periods: usize,
    buff: VecDeque<f32>,
    last_price: Option<f32>,
    tr_sum: f32,
    value: Option<f32>,
}


impl Atr{

    pub fn new(periods: usize) -> Self{

        assert!(periods > 0, "Atr periods must be a periods > 0, ({})", periods);
        Atr{
            periods,
            buff: VecDeque::with_capacity(periods),
            last_price: None,
            tr_sum: 0.0,
            value: None,
        }
    }

    

}



impl Indicator for Atr{

    fn update_after_close(&mut self, last_price: Price){
        

        let h_l = last_price.high - last_price.low;
        if self.last_price.is_none(){
            self.buff.push_back(h_l);
            self.tr_sum += h_l;
            self.last_price = Some(last_price.close);
            return;
        }   

        let prev_close = self.last_price.unwrap();
        let tr = calc_tr(last_price.high, last_price.low, prev_close);

        if self.is_ready(){
            let expired_tr = self.buff.pop_front().unwrap();
            self.tr_sum -= expired_tr;
        }
            
        self.buff.push_back(tr);
        self.tr_sum += tr;
        self.last_price = Some(last_price.close);
        
        if self.is_ready(){
            self.value = Some(self.tr_sum/self.periods as f32); 
        }
       
    }


    fn update_before_close(&mut self, last_price: Price){
    
        if self.is_ready(){
            let prev_close = self.last_price.unwrap();
            if prev_close == last_price.open{
                return;
            }
            
            let last_tr = self.buff.pop_back().unwrap();
            self.tr_sum -= last_tr;

            let tr = calc_tr(last_price.high, last_price.low, prev_close);
            self.buff.push_back(tr);
            self.tr_sum += tr;

            self.value = Some(self.tr_sum/self.periods as f32);
        }
    }

    fn get_last(&self) -> Option<f32>{
        self.value
    }

    fn is_ready(&self) -> bool{

        self.buff.len() == self.buff.capacity()
    }

    
    fn load(&mut self, price_data: &Vec<Price>){

        if price_data.len() > 1 {
            
            for p in price_data{
                self.update_after_close(*p);
            }
        };
       }
    }


fn calc_tr(high: f32, low: f32, prev_close: f32) -> f32{

        f32::max(high - low, f32::max((high - prev_close).abs(), (low - prev_close).abs()))
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
