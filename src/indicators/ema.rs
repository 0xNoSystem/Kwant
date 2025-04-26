use crate::indicators::{Price,Value, Indicator, Sma};


#[derive(Clone, Debug)]
pub struct Ema{
    periods: u32,
    alpha: f32,
    buff: Sma,
    prev_value: Option<f32>,
    pub value: Option<f32>,
    slope: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct EmaCross{
    pub short: Ema,
    pub long: Ema,
    prev_uptrend: Option<bool>,
}



impl EmaCross{

    pub fn new(period_short: u32, period_long: u32) -> Self{
        
        
        EmaCross{
            short: Ema::new(period_short.min(period_long)),
            long: Ema::new(period_short.max(period_long)),
            prev_uptrend: None,
        }
    }

    pub fn check_for_cross(&mut self) -> Option<bool> {
        if !self.is_ready(){
            None
        }else{
            let uptrend = self.get_trend().unwrap();

            if let Some(prev_uptrend) = self.prev_uptrend {
                
                if uptrend != prev_uptrend{
                    self.prev_uptrend = Some(uptrend);
                    Some(uptrend)
                }else{
                    None
                }
            }else{
                self.prev_uptrend = Some(uptrend);
                None
            }
        }
    }

    pub fn update(&mut self,price: Price ,after_close: bool){

        if after_close{
            self.update_after_close(price);
        }else{
            self.update_before_close(price);
        }

        if self.is_ready() && self.prev_uptrend.is_none(){
            self.prev_uptrend = self.get_trend();
        }
    }

    pub fn update_and_check_for_cross(&mut self,price: Price ,after_close: bool) -> Option<bool>{

        self.update(price, after_close);
        self.check_for_cross()

    }

    pub fn get_trend(&self) -> Option<bool>{
        if self.is_ready(){
            Some(self.short.get_last() >= self.long.get_last())
        }else{
            None
        }
    }
 }

impl Indicator for EmaCross{
    fn update_after_close(&mut self, price: Price){

        self.short.update_after_close(price);
        self.long.update_after_close(price);

    }

    fn update_before_close(&mut self, price: Price){

        self.short.update_before_close(price);
        self.long.update_before_close(price);
    }

    fn is_ready(&self) -> bool{

        self.short.is_ready() && self.long.is_ready()
    }


    fn period(&self) -> u32{
        //return long only
        self.long.period()
    }

   
    fn load(&mut self, price_data: &[Price]){
        for p in price_data{
            self.update_after_close(*p);
        }
    }
 
    fn reset(&mut self){
        self.short.reset();
        self.long.reset();
        self.prev_uptrend = None;
    }
    fn get_last(&self) -> Option<Value>{
        if let (Some(sh), Some(lg)) = (self.short.value, self.long.value){
            return Some(Value::EmaCrossValue{short: sh, long: lg, trend:self.get_trend().unwrap()});  
        }

        None
}

}


impl Default for EmaCross{
    fn default() -> Self{

        EmaCross::new(9, 21)
    }
}




impl Ema{

    pub fn new(periods: u32) -> Self{

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
                self.value = self.buff.value;
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

    fn get_last(&self) -> Option<Value>{
        if let Some(val) = self.value{
            return Some(Value::EmaValue(val));
        }

        None
    }

    fn is_ready(&self) -> bool{

        self.value.is_some()
    }

    fn load(&mut self, price_data: &[Price]){

        for p in price_data{
            self.update_after_close(*p);
        }
    }

    fn reset(&mut self){
        self.buff.reset();
        self.value = None;
        self.slope = None;
    }

    fn period(&self) -> u32{
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
