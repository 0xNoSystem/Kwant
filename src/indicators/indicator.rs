use crate::indicators::Price;
use std::fmt::Debug;

pub trait Indicator: Debug + Sync + Send{

    fn update_after_close(&mut self, last_price: Price);
    fn update_before_close(&mut self, last_price: Price);
    fn load(&mut self, price_data: &[Price]);
    fn is_ready(&self) -> bool;
    fn get_last(&self) -> Option<Value>;
    fn reset(&mut self);
    fn period(&self) -> u32;
}

#[derive(PartialEq, PartialOrd, Copy, Clone, Debug)]
pub enum Value{
    RsiValue(f32),
    StochRsiValue{k: f32, d: f32},
    EmaValue(f32),
    EmaCrossValue{short: f32, long: f32, trend: bool},
    SmaValue(f32),
    SmaRsiValue(f32),
    AdxValue(f32),
    AtrValue(f32),
}
