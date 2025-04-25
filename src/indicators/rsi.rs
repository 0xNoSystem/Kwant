use std::collections::VecDeque;
use crate::indicators::Price;
use crate::indicators::Indicator;
use crate::indicators::StochRsi;

#[derive(Clone, Debug)]
pub struct Rsi{
    periods: usize,
    buff: RsiBuffer,
    last_price: Option<f32>,
    value: Option<f32>,
    sma: Option<SmaOnRsi>,
    stoch: StochRsi,
}

#[derive(Clone, Debug)]
struct RsiBuffer{
    changes_buffer: VecDeque<f32>, 
    sum_gain: f32,
    sum_loss: f32,
    last_avg_gain: Option<f32>,
    last_avg_loss: Option<f32>,
    in_candle: bool,
}

#[derive(Clone, Debug)]
pub struct SmaOnRsi{
    buff: VecDeque<f32>,
    length: usize, 
    current_sum: f32,
}


impl SmaOnRsi{
    fn new(smoothing_length: usize) -> Self{
       
        assert!(smoothing_length > 1, "length field must be a positive integer > 1, ({})", smoothing_length);


        SmaOnRsi{
            buff: VecDeque::with_capacity(smoothing_length), 
            length: smoothing_length,
            current_sum: 0.0,
        }
    }

    fn push(&mut self, new_rsi: f32){

        if self.is_full(){
            let expired_rsi = self.buff.pop_front().unwrap();
            self.current_sum -= expired_rsi;
        }
        self.buff.push_back(new_rsi);
        self.current_sum += new_rsi;

    }

    fn get(&self) -> Option<f32>{

        if self.is_full(){
            Some(self.current_sum / (self.length) as f32)
        }else{
            None
        }


    }

    fn is_full(&self) -> bool{

        self.buff.len() == self.length
    }
}




impl Rsi{

    pub fn new(periods: usize,stoch_length: usize,smoothing_length: Option<usize>) -> Self{

        assert!(periods > 1, "Periods field must be a positive integer > 1, ({})", periods);
        
        let sma = smoothing_length.map(SmaOnRsi::new);

        Rsi{
            periods: periods,
            buff: RsiBuffer::new(periods - 1),
            last_price: None,
            value: None,
            sma: sma,
            stoch: StochRsi::new(stoch_length, 3, 3),
        }
    }

     fn calc_rsi(&mut self, change: f32, last_avg_gain: f32, last_avg_loss: f32, after: bool) -> Option<f32>{

        let change_loss = (-change).max(0.0);
        let change_gain = (change).max(0.0);

        let avg_gain = (last_avg_gain*(self.periods as f32 - 1.0) + change_gain) / self.periods as f32;
        let avg_loss = (last_avg_loss*(self.periods as f32 - 1.0)+ change_loss ) /self.periods as f32;

        let rsi = if avg_loss == 0.0{
            100.0
        }else{
            100.0 - (100.0 / (1.0 + (avg_gain / avg_loss)))
        };
        
        if after{
            self.buff.last_avg_gain = Some(avg_gain);
            self.buff.last_avg_loss = Some(avg_loss); 
            self.stoch.update_after_close(rsi);
            if let Some(sma) = &mut self.sma{
                sma.push(rsi);
            }
        }else {
            self.stoch.update_before_close(rsi);
        };

        self.value = Some(rsi);

        Some(rsi)
    }
    
    pub fn get_sma_rsi(&self) -> Option<f32>{
        if let Some(sma) = &self.sma{
            sma.get()
        }else{
            None
        }
    }

    pub fn get_stoch_rsi(&self) -> Option<f32> {
        self.stoch.get_k()
    }

    pub fn get_stoch_signal(&self) -> Option<f32> {
        self.stoch.get_d()
    }
}



impl Indicator for Rsi{

    fn update_before_close(&mut self, price: Price){
        let price = price.close;

        let change = match self.last_price {
            Some(prev_price) => price - prev_price,
            None => {
                self.last_price = Some(price);
                return;
            }
        };

        self.buff.push_before_close(change);

        if self.buff.is_full(){
           match (self.buff.last_avg_gain, self.buff.last_avg_loss) {
            
            (Some(last_avg_gain), Some(last_avg_loss)) =>{
                self.calc_rsi(change,last_avg_gain, last_avg_loss, false);

                } 

            _ => {
                return;
            }}
        }

    }


    fn update_after_close(&mut self, price: Price){
        let price = price.close;
        let change = match self.last_price {
        Some(prev_price) => price - prev_price,
        None => {
            self.last_price = Some(price);
            return;
        }
        };
        
        self.buff.push(change);
        self.last_price = Some(price);

        if self.buff.is_full(){
    
            match (self.buff.last_avg_gain, self.buff.last_avg_loss) {
            
            (Some(last_avg_gain), Some(last_avg_loss)) =>{

                self.calc_rsi(change, last_avg_gain,last_avg_loss, true);
            }

            _ =>  { 
                    return;
                }

            }
                  }
    }
    fn get_last(&self) -> Option<f32>{

            self.value
        }  

   

    fn is_ready(&self) -> bool{

        self.buff.is_full() && self.value.is_some() 
    }

    fn load<'a,I: IntoIterator<Item=&'a Price>>(&mut self, price_data: I){

        for p in price_data{
            self.update_after_close(*p);
        }

    }

    fn reset(&mut self) {
    self.buff = RsiBuffer::new(self.periods - 1);
    self.last_price = None;
    self.value = None;
    if let Some(sma) = &mut self.sma {
        *sma = SmaOnRsi::new(sma.length);
    }
    self.stoch.reset();
}

    fn period(&self) -> usize{
        self.periods
    }
}





impl RsiBuffer{

    fn new(capacity: usize) -> Self{
        RsiBuffer{
            changes_buffer: VecDeque::with_capacity(capacity), 
            sum_gain: 0_f32,
            sum_loss: 0_f32,
            last_avg_gain: None,
            last_avg_loss: None,
            in_candle: true,   
        }
    }

    fn push(&mut self, change: f32){
        
        if self.is_full(){
            self.init_last_avg();
            let expired_change = self.changes_buffer.pop_front().unwrap();

            if expired_change > 0.0{
                self.sum_gain -= expired_change;
            }else{
                self.sum_loss -= expired_change.abs();
            }  
        }

        if change > 0.0{
            self.sum_gain += change;
        }else{
            self.sum_loss += change.abs();
        }

        self.changes_buffer.push_back(change);   
        self.in_candle = true;

    }
    fn push_before_close(&mut self, change: f32){

        if !self.is_full(){return;}
        let expired_change: f32;
        if !self.in_candle{
            expired_change = self.changes_buffer.pop_back().unwrap();
        }else{
            expired_change = self.changes_buffer.pop_front().unwrap();
            self.in_candle = false;
        }
        if expired_change > 0.0{
            self.sum_gain -= expired_change;
        }else{
            self.sum_loss -= expired_change.abs();
        }  

            if change > 0.0{
        self.sum_gain += change;
        }else{
        self.sum_loss += change.abs();
        }

        self.changes_buffer.push_back(change);

    }
    


    fn is_full(&self) -> bool{

        self.changes_buffer.len() == self.changes_buffer.capacity() - 1
    }

    fn init_last_avg(&mut self){
        if self.last_avg_gain.is_none(){
                self.last_avg_gain = Some(self.sum_gain / (self.changes_buffer.capacity()) as f32);
            }

        if self.last_avg_loss.is_none(){
            self.last_avg_loss = Some(self.sum_loss / (self.changes_buffer.capacity()) as f32);
        }
    }

    

}

impl Default for Rsi{

    fn default() -> Self{

        Rsi{
            periods: 14,
            buff: RsiBuffer::new(14-1),
            last_price: None,
            value: None,
            sma: Some(SmaOnRsi::new(10)),
            stoch: StochRsi::new(14, 3, 3),
        }
    }
}
