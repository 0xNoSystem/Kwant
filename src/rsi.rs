use std::collections::VecDeque;


pub struct Rsi{
    periods: usize,
    last_price: Option<f32>,
    buff: RsiBuffer,
    value: Option<f32>,
    rsi_buffer: VecDeque<f32>,
    smoothing_length: usize,
}

struct RsiBuffer{
    buff: VecDeque<f32>,
    capacity: usize,
    sum_w: f32,
    sum_l: f32,
}

impl RsiBuffer{

    fn new(capacity: usize)-> Self{
        
        RsiBuffer{
            buff: VecDeque::with_capacity(capacity),
            capacity: capacity,
            sum_w: 0_f32,
            sum_l: 0_f32,
        }
    }

    fn is_full(&self) -> bool{

        self.buff.len() == self.capacity
    }

    fn push(&mut self, change: f32){
        
        if self.is_full(){
            let expired_change = self.buff.pop_front().unwrap();

                if expired_change > 0.0{
                    self.sum_w -= expired_change;
                }else{
                    self.sum_l -= expired_change.abs();
                }   

        }

        if change > 0.0{
            self.sum_w += change;
        }else{
            self.sum_l += change.abs();
        }

        self.buff.push_back(change);   
    }

    fn push_current_close(&mut self, change: f32) {

        let expired_change = self.buff.pop_back().unwrap();

            if expired_change > 0.0{
                self.sum_w -= expired_change;
            }else{
                self.sum_l -= expired_change.abs();
            }   

        if change > 0.0{
            self.sum_w += change;
        }else{
            self.sum_l += change.abs();
        }

        self.buff.push_back(change);   
    }
}

    



impl Rsi{
    pub fn new(periods: usize, smoothing_length: usize) -> Self{

        Rsi{
            periods: periods,
            last_price: None,
            buff: RsiBuffer::new(periods - 1),
            value: None,
            rsi_buffer: VecDeque::with_capacity(smoothing_length),
            smoothing_length: smoothing_length,
        }
    }

    pub fn update(&mut self, price: f32) -> Option<f32>{

        if let Some(last_price) = self.last_price{
            let diff = price - last_price;
            self.buff.push(diff);
        }

        self.last_price = Some(price);

        if self.buff.is_full(){
            let avg_gain = self.buff.sum_w / self.periods as f32;
            let avg_loss = self.buff.sum_l / self.periods as f32;

            let rsi = if avg_loss == 0.0 {
                100.0
            } else {
                100.0 - (100.0 / (1.0 + (avg_gain / avg_loss)))
            };
            self.value = Some(rsi);

            if let Some(smoothed_rsi) = self.get_smooth(rsi, true){
                return Some(smoothed_rsi);
            }

            self.value
        }else{
            None
        }

    }

    pub fn update_current_close(&mut self, price: f32) -> Option<f32> {

        if self.buff.is_full(){

            if let Some(last_price) = self.last_price{
            let diff = price - last_price;
            self.buff.push_current_close(diff);
            }

            let avg_gain = self.buff.sum_w / self.periods as f32;
            let avg_loss = self.buff.sum_l / self.periods as f32;

            let rsi = if avg_loss == 0.0 {
                100.0
            } else {
                100.0 - (100.0 / (1.0 + (avg_gain / avg_loss)))
            };
            self.value = Some(rsi);

            if let Some(smoothed_rsi) = self.get_smooth(rsi, false){
                return Some(smoothed_rsi);
            }
            self.value
        }else{
            None
        }

    }

    pub fn get_raw(&self) -> Option<f32>{
        
        self.value
    }

    pub fn get_smooth(&mut self, rsi: f32, new: bool) -> Option<f32>{
        let is_ready = {self.rsi_buffer.len() == self.smoothing_length};

        if is_ready{
            match new{
                true=> {self.rsi_buffer.pop_front();},
                false=>{self.rsi_buffer.pop_back();}
            }
        }
        self.rsi_buffer.push_back(rsi);

        if is_ready{
            let sum: f32 = self.rsi_buffer.iter().sum();
            Some(sum / (self.smoothing_length as f32))
        }else{
            None
        }
    }

}


#[cfg(test)]

mod test{
    use super::*;

    #[test]
    fn push(){
        let mut rsi = Rsi::new(9);

        assert_eq!(rsi.update(121.2), None);
        assert_eq!(rsi.update(121.2), None);

        rsi.update(121.0);
        rsi.update(121.2);
        rsi.update(121.3);
        rsi.update(120.9);
        rsi.update(121.3);
        rsi.update(121.7);
        
        rsi.update(121.9);

        let val = rsi.get();
        assert_ne!(rsi.get(), None);

        println!("{:?}", val);
        
    }
}
