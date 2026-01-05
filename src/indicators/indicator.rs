use crate::indicators::Price;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};

pub trait Indicator: Debug + Sync + Send {
    fn update_after_close(&mut self, last_price: Price);
    fn update_before_close(&mut self, last_price: Price);
    fn load(&mut self, price_data: &[Price]);
    fn is_ready(&self) -> bool;
    fn get_last(&self) -> Option<Value>;
    fn reset(&mut self);
    fn period(&self) -> u32;
}

#[derive(PartialEq, PartialOrd, Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Value {
    RsiValue(f64),
    StochRsiValue { k: f64, d: f64 },
    EmaValue(f64),
    EmaCrossValue { short: f64, long: f64, trend: bool },
    SmaValue(f64),
    SmaRsiValue(f64),
    AdxValue(f64),
    AtrValue(f64),
    VolumeMaValue(f64),
    StdDevValue(f64),
}
