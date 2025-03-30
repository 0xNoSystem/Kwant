use crate::indicators::Price;


pub trait Indicator{

    fn update_after_close(&mut self, last_price: Price);
    fn update_before_close(&mut self, last_price: Price);
    fn is_ready(&self) -> bool;
    fn get_last(&self) -> Option<f32>;
}