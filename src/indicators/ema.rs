use crate::indicators::{Price, Indicator, Sma};


#[derive(Clone, Debug)]
pub struct Ema{
    periods: usize,
    alpha: f32,
    buff: Sma,
    prev_value: Option<f32>,
    value: Option<f32>,
    slope: Option<f32>,
}


pub struct EmaCross{
    pub short: Ema,
    pub long: Ema,
    uptrend: Option<bool>,
}



impl EmaCross{

    pub fn new(period_short: usize, period_long: usize) -> Self{
        
        
        EmaCross{
            short: Ema::new(period_short.min(period_long)),
            long: Ema::new(period_short.max(period_long)),
            uptrend: None,
        }
    }

    fn check_for_cross(&mut self) -> Option<bool> {

        if let (Some(short_value), Some(long_value)) = (self.short.get_last(), self.long.get_last()) {
            
            let uptrend = short_value > long_value;
            if self.uptrend != Some(uptrend){
                self.uptrend = Some(uptrend);
                Some(uptrend)
            }else{
                None
            }
        }else{
            None
        }
    }

    pub fn update(&mut self,price: Price ,after_close: bool) -> Option<bool>{

        if after_close{
            self.update_after_close(price);
        }else{
            self.update_before_close(price);
        }

        self.check_for_cross()
    }

    fn update_after_close(&mut self, price: Price){

        self.short.update_after_close(price);
        self.long.update_after_close(price);

    }

    fn update_before_close(&mut self, price: Price){

        self.short.update_before_close(price);
        self.long.update_before_close(price);
    }

    pub fn is_ready(&self) -> bool{

        self.short.is_ready() && self.long.is_ready()
    }

    pub fn get_trend(&self) -> Option<bool>{
        self.uptrend
    }

    pub fn periods(&self) -> (usize,usize){

        (self.short.period(), self.long.period())
    }

    pub fn load(&mut self, price_data: &Vec<Price>){

        for p in price_data{
            self.update(*p, true);
        }
    }

}







impl Ema{

    pub fn new(periods: usize) -> Self{

        assert!(periods > 1, "Ema  periods field must a positive integer n > 1, {} ", periods);

        Ema{
            periods,
            buff: Sma::new(periods),
            alpha: 2.0/(periods as f32 + 1.0),
            prev_value: None,
            value: None,
            slope: None,
        }
    }

    pub fn get_sma(&self) -> Option<f32>{
        self.buff.get_last()
    }

    pub fn get_slope(&self) -> Option<f32>{
        self.slope
    }
}


impl Indicator for Ema{

    fn update_after_close(&mut self, price: Price){
        let close = price.close;
        self.buff.update_after_close(price);
        
        if let Some(last_ema)  = self.value{
            let ema = (self.alpha*close) + (1.0 - self.alpha)*last_ema;
            self.slope = Some(((ema - last_ema) / ema)*100.0);
            self.prev_value = Some(last_ema);
            self.value = Some(ema);
        }else{
            if self.buff.is_ready(){
            self.value = self.buff.get_last();
        }

        
        }
    }

    fn update_before_close(&mut self, price: Price){

        
        if let Some(last_ema) = self.prev_value{
            let close = price.close;
            let ema = (self.alpha*close) + (1.0 - self.alpha)*last_ema;
            self.slope = Some(((ema - last_ema)/ema)*100.0);
            self.value = Some(ema);
        }

        if self.buff.is_ready(){
            self.buff.update_before_close(price);
        }
        
    }

    fn get_last(&self) -> Option<f32>{
        self.value
    }

    fn is_ready(&self) -> bool{

        self.value.is_some()
    }

    fn load(&mut self, price_data: &Vec<Price>){

        for p in price_data{
            self.update_after_close(*p);
        }
    }

    fn reset(&mut self){
        self.buff.reset();
        self.value = None;
        self.slope = None;
    }

    fn period(&self) -> usize{
        self.periods
    }
}

impl Default for Ema{
    fn default() -> Self{

        Ema{
            periods: 9,
            buff: Sma::new(9),
            alpha: 2.0/(9.0 + 1.0),
            prev_value: None,
            value: None,
            slope: None,
        }
    }
}












