use crate::indicators::{Price,Value, Indicator};
use crate::Mean;

#[derive(Clone, Debug)]
pub struct VolumeMa{
    periods: u32,
    mean: Mean, 
}

impl VolumeMa{

    pub fn new(periods: u32) -> Self{

        assert!(periods > 1, "VolMa  periods field must a positive integer n > 1, {} ", periods);
        VolumeMa{
            periods,
            mean: Mean::new(periods), 
        }
    }
}

impl Indicator for VolumeMa {
    fn update_after_close(&mut self, price: Price) {
        self.mean.update_after_close(price.vlm);
    }

    fn update_before_close(&mut self, price: Price) {
        self.mean.update_before_close(price.vlm);
    }

    fn load(&mut self, price_data: &[Price]) {
        for p in price_data{
            self.mean.update_after_close(p.vlm);
        }
    }

    fn is_ready(&self) -> bool {
        self.mean.is_ready()
    }

    fn get_last(&self) -> Option<Value> {
        self.mean
            .get_last()
            .map(Value::VolumeMaValue)
    }

    fn reset(&mut self) {
        self.mean.reset();
    }

    fn period(&self) -> u32 {
        self.periods
    }
}



impl Default for VolumeMa{

    fn default() -> Self{
        VolumeMa{
            periods: 14,
            mean: Mean::new(14),             
        }
    }
}




