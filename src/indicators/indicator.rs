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
    StochRsiValue {
        k: f64,
        d: f64,
    },
    EmaValue(f64),
    DemaValue(f64),
    TemaValue(f64),
    ObvValue(f64),
    VwapDeviationValue(f64),
    CciValue(f64),
    IchimokuValue {
        tenkan: f64,
        kijun: f64,
        span_a: f64,
        span_b: f64,
        chikou: f64,
    },
    EmaCrossValue {
        short: f64,
        long: f64,
        trend: bool,
    },
    MacdValue {
        macd: f64,
        signal: f64,
        histogram: f64,
    },
    SmaValue(f64),
    SmaRsiValue(f64),
    RocValue(f64),
    BollingerValue {
        upper: f64,
        mid: f64,
        lower: f64,
        width: f64,
    },
    AdxValue(f64),
    AtrValue(f64),
    VolumeMaValue(f64),
    StdDevValue(f64),
    HistVolatilityValue(f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum IndicatorKind {
    Rsi(u32),
    SmaOnRsi {
        periods: u32,
        smoothing_length: u32,
    },
    StochRsi {
        periods: u32,
        k_smoothing: Option<u32>,
        d_smoothing: Option<u32>,
    },
    Adx {
        periods: u32,
        di_length: u32,
    },
    Atr(u32),
    Ema(u32),
    Dema(u32),
    Tema(u32),
    Obv,
    VwapDeviation(u32),
    Cci(u32),
    Ichimoku {
        tenkan: u32,
        kijun: u32,
        senkou_b: u32,
    },
    EmaCross {
        short: u32,
        long: u32,
    },
    Macd {
        fast: u32,
        slow: u32,
        signal: u32,
    },
    Sma(u32),
    Roc(u32),
    BollingerBands {
        periods: u32,
        std_multiplier_x100: u32,
    },
    VolMa(u32),
    HistVolatility(u32),
}

impl IndicatorKind {
    pub fn key(&self) -> String {
        match self {
            IndicatorKind::Rsi(p) => format!("rsi_{}", p),
            IndicatorKind::Atr(p) => format!("atr_{}", p),
            IndicatorKind::Ema(p) => format!("ema_{}", p),
            IndicatorKind::Dema(p) => format!("dema_{}", p),
            IndicatorKind::Tema(p) => format!("tema_{}", p),
            IndicatorKind::Obv => "obv".to_string(),
            IndicatorKind::VwapDeviation(p) => format!("vwapDeviation_{}", p),
            IndicatorKind::Cci(p) => format!("cci_{}", p),
            IndicatorKind::Sma(p) => format!("sma_{}", p),
            IndicatorKind::Roc(p) => format!("roc_{}", p),
            IndicatorKind::VolMa(p) => format!("volMa_{}", p),
            IndicatorKind::HistVolatility(p) => format!("histVol_{}", p),
            IndicatorKind::SmaOnRsi {
                periods,
                smoothing_length,
            } => format!("smaRsi_{}_{}", periods, smoothing_length),
            IndicatorKind::StochRsi {
                periods,
                k_smoothing,
                d_smoothing,
            } => format!(
                "stochRsi_{}_{}_{}",
                periods,
                k_smoothing.unwrap_or(3),
                d_smoothing.unwrap_or(3)
            ),
            IndicatorKind::Adx { periods, di_length } => format!("adx_{}_{}", periods, di_length),
            IndicatorKind::EmaCross { short, long } => format!("emaCross_{}_{}", short, long),
            IndicatorKind::Macd { fast, slow, signal } => {
                format!("macd_{}_{}_{}", fast, slow, signal)
            }
            IndicatorKind::Ichimoku {
                tenkan,
                kijun,
                senkou_b,
            } => format!("ichimoku_{}_{}_{}", tenkan, kijun, senkou_b),
            IndicatorKind::BollingerBands {
                periods,
                std_multiplier_x100,
            } => format!(
                "bollinger_{}_{}",
                periods,
                format_multiplier_x100(*std_multiplier_x100)
            ),
        }
    }
}

fn format_multiplier_x100(std_multiplier_x100: u32) -> String {
    let whole = std_multiplier_x100 / 100;
    let fraction = std_multiplier_x100 % 100;

    if fraction == 0 {
        whole.to_string()
    } else if fraction.is_multiple_of(10) {
        format!("{}.{}", whole, fraction / 10)
    } else {
        format!("{}.{:02}", whole, fraction)
    }
}
