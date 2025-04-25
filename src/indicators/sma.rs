use  std::collections::VecDeque;
use crate::indicators::{Price, Indicator};


#[derive(Clone, Debug)]
pub struct Sma{
    periods: u32,
    buff: VecDeque<f32>,
    sum: f32,
    value: Option<f32>,
    in_candle: bool,
}



impl Sma{

    pub fn new(periods: u32) -> Self{

        assert!(periods > 1, "Sma  periods field must a positive integer n > 1, {} ", periods);
        Sma{
            periods,
            buff: VecDeque::with_capacity(periods as usize),
            sum: 0.0,
            value: None,
            in_candle: true,
        }
    }
}

impl Indicator for Sma{


    fn update_after_close(&mut self, price: Price){
        let price = price.close;

        if self.is_ready(){
            let expired_price = self.buff.pop_front().unwrap();
            self.sum -= expired_price;
        }

        self.buff.push_back(price);
        self.sum += price;
        if self.is_ready(){
            self.value = Some(self.sum / self.periods as f32);
        }

        self.in_candle = false;
    }

    fn update_before_close(&mut self, price: Price){

        let price = price.close;
        if self.is_ready(){
            let last_price: f32;
            if !self.in_candle{
                last_price = self.buff.pop_front().unwrap();
            }else{
                last_price = self.buff.pop_back().unwrap();
            }
            self.sum -= last_price;
            self.buff.push_back(price);
            self.sum += price;
            
            self.value = Some(self.sum/ self.periods as f32);
            self.in_candle = true;
        }
    }

    fn get_last(&self) -> Option<f32>{

        self.value
    }

    fn is_ready(&self) -> bool{

        self.buff.len() == self.buff.capacity()
    }

    fn period(&self) -> u32{
        
        self.periods
    }

    fn reset(&mut self){

        self.buff.clear();
        self.sum = 0.0;
        self.value = None;
    }

    fn load<'a,I: IntoIterator<Item=&'a Price>>(&mut self, price_data: I){
        for p in price_data{
            self.update_after_close(*p);
        }
    }
}


impl Default for Sma{

    fn default() -> Self{
        Sma{
            periods: 9,
            buff: VecDeque::with_capacity(9),
            sum: 0.0,
            value: None,
            in_candle: true,
        }
    }
}



