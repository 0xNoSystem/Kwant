use std::collections::VecDeque;



pub struct Rsi{
    periods: usize,
    buff: RsiBuffer,
    last_price: Option<f32>,
    value: Option<f32>,

}

struct RsiBuffer{
    changes_buffer: VecDeque<f32>,
    capacity: usize,
    sum_gain: f32,
    sum_loss: f32,
    last_avg_gain: Option<f32>,
    last_avg_loss: Option<f32>,
}


impl Rsi{

    pub fn new(periods: usize) -> Self{

        assert!(periods > 1, "Periods field must be a positive integer > 1, ({})", periods);

        Rsi{
            periods: periods,
            buff: RsiBuffer::new(periods-1),
            last_price: None,
            value: None,
        }
    }


    pub fn update_before_close(&mut self, price: f32) -> Option<f32>{
        let change = match self.last_price {
        Some(prev_price) => price - prev_price,
        None => {
            self.last_price = Some(price);
            return None;
        }
        };

        self.buff.push_before_close(change);
        self.last_price = Some(price);

        if self.buff.is_full(){
           match (self.buff.last_avg_gain, self.buff.last_avg_loss) {
            
            (Some(last_avg_gain), Some(last_avg_loss)) =>{
                self.calc_rsi(change,last_avg_gain, last_avg_loss)

                } 

            _ => {
                None
            }}
        }else{
            None
        }

    }


    pub fn update_after_close(&mut self, price: f32) -> Option<f32>{
        let change = match self.last_price {
        Some(prev_price) => price - prev_price,
        None => {
            self.last_price = Some(price);
            return None;
        }
        };
        
        self.buff.push_before_close(change);
        self.last_price = Some(price);

        if self.buff.is_full(){
    
            match (self.buff.last_avg_gain, self.buff.last_avg_loss) {
            
            (Some(last_avg_gain), Some(last_avg_loss)) =>{

                self.calc_rsi(change, last_avg_gain,last_avg_loss)
            }

            _ =>  { 
                    None
                }

        }
                  }else{ 
            None
        }
    }
    pub fn get_last(&self) -> Option<f32>{

            self.value
        }  

    fn calc_rsi(&mut self, change: f32, last_avg_gain: f32, last_avg_loss: f32) -> f32{

        let change_loss = (-change).max(0.0);
        let change_gain = (change).max(0.0);

        let avg_gain = (last_avg_gain*(self.periods as f32 - 1.0) + change_gain) / self.periods as f32;
        let avg_loss = (last_avg_loss*(self.periods as f32 - 1.0)+ change_loss ) /self.periods as f32;

        let rsi = if avg_loss == 0.0{
            100.0
        }else{
            100.0 - (100.0 / (1.0 + (avg_gain / avg_loss)))
        };
        
        self.value = Some(rsi);
        self.buff.last_avg_gain = Some(avg_gain);
        self.buff.last_avg_loss = Some(avg_loss);
        rsi
    }


}




impl RsiBuffer{

    fn new(capacity: usize) -> Self{
        RsiBuffer{
            changes_buffer: VecDeque::with_capacity(capacity), 
            capacity: capacity,
            sum_gain: 0_f32,
            sum_loss: 0_f32,
            last_avg_gain: None,
            last_avg_loss: None,
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

    }
    fn push_before_close(&mut self, change: f32){

        assert!(self.is_full(), "push_before_close() called before buffer is full");

        let expired_change = self.changes_buffer.pop_back().unwrap();

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

        self.changes_buffer.len() == self.capacity
    }

    fn init_last_avg(&mut self){
        if self.last_avg_gain.is_none(){
                self.last_avg_gain = Some(self.sum_gain / (self.capacity + 1) as f32);
            }

        if self.last_avg_loss.is_none(){
            self.last_avg_loss = Some(self.sum_loss / (self.capacity + 1) as f32);
        }
    }
}