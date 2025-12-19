#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Price {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub open_time: u64,
    pub close_time: u64,
    pub vlm: f64,
}
