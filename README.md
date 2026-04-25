# Kwant

Kwant is a candle-driven technical indicators library for Rust, built around the workflow used in a Hyperliquid trading terminal. It is designed for two update modes:

- `update_before_close(price)` for provisional, in-candle values on every tick
- `update_after_close(price)` for the final committed candle close

The crate exposes a shared `Indicator` trait, a shared `Price` input type, and a `Value` enum that carries each indicator's output shape.

## Installation

```toml
kwant = { git = "https://github.com/0xNoSystem/Kwant" }
```

## Core interface

Every indicator implements the same trait:

```rust
pub trait Indicator: Debug + Sync + Send {
    fn update_after_close(&mut self, last_price: Price);
    fn update_before_close(&mut self, last_price: Price);
    fn load(&mut self, price_data: &[Price]);
    fn is_ready(&self) -> bool;
    fn get_last(&self) -> Option<Value>;
    fn reset(&mut self);
    fn period(&self) -> u32;
}
```

### Price input

All indicators consume the same `Price` struct:

```rust
pub struct Price {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub open_time: u64,
    pub close_time: u64,
    pub vlm: f64,
}
```

In practice, most indicators only use a subset of these fields:

- close-based indicators use `close`
- range-based indicators use `high`, `low`, and often previous `close`
- volume-based indicators use `vlm`

## Indicator groups

### Momentum

#### CCI

- **Input**: `high`, `low`, `close`
- **Output**: `Value::CciValue(f64)`
- **Formula**:
  - `TP = (high + low + close) / 3`
  - `SMA_TP = mean(TP, period)`
  - `MD = mean(|TP_i - SMA_TP|, period)`
  - `CCI = (TP - SMA_TP) / (0.015 * MD)`

#### MACD

- **Input**: `close`
- **Output**: `Value::MacdValue { macd, signal, histogram }`
- **Formula**:
  - `EMA_fast = EMA(close, fast)`
  - `EMA_slow = EMA(close, slow)`
  - `macd = EMA_fast - EMA_slow`
  - `signal = EMA(macd, signal_period)`
  - `histogram = macd - signal`

#### ROC

- **Input**: `close`
- **Output**: `Value::RocValue(f64)`
- **Formula**:
  - `ROC = ((close_t / close_{t-period}) - 1) * 100`

#### RSI

- **Input**: `close`
- **Output**: `Value::RsiValue(f64)`
- **Formula**:
  - `change = close_t - close_{t-1}`
  - `gain = max(change, 0)`
  - `loss = max(-change, 0)`
  - Wilder smoothing:
    - `avg_gain = ((prev_avg_gain * (period - 1)) + gain) / period`
    - `avg_loss = ((prev_avg_loss * (period - 1)) + loss) / period`
  - `RSI = 100 - (100 / (1 + avg_gain / avg_loss))`
  - if `avg_loss == 0`, RSI is `100`

#### SMA on RSI

- **Input**: `close`
- **Output**: `Value::SmaRsiValue(f64)`
- **Formula**:
  - first compute RSI
  - then compute a simple moving average over the last `smoothing_length` RSI values

#### Stochastic RSI

- **Input**: `close`
- **Output**: `Value::StochRsiValue { k, d }`
- **Formula**:
  - first compute RSI
  - `raw_k = (RSI - min(RSI, period)) / (max(RSI, period) - min(RSI, period))`
  - `%K = SMA(raw_k, k_smoothing) * 100`
  - `%D = SMA(%K, d_smoothing) * 100`

### Trend

#### ADX

- **Input**: `high`, `low`, `close`
- **Output**: `Value::AdxValue(f64)`
- **Formula**:
  - `TR = max(high - low, |high - prev_close|, |low - prev_close|)`
  - `+DM = high - prev_high` when it exceeds down move and is positive, otherwise `0`
  - `-DM = prev_low - low` when it exceeds up move and is positive, otherwise `0`
  - smooth `TR`, `+DM`, and `-DM` with Wilder smoothing over `di_length`
  - `+DI = 100 * (+DM_smooth / TR_smooth)`
  - `-DI = 100 * (-DM_smooth / TR_smooth)`
  - `DX = 100 * |+DI - -DI| / (+DI + -DI)`
  - `ADX` is the Wilder average of `DX` over `period`

#### DEMA

- **Input**: `close`
- **Output**: `Value::DemaValue(f64)`
- **Formula**:
  - `EMA1 = EMA(close, period)`
  - `EMA2 = EMA(EMA1, period)`
  - `DEMA = 2 * EMA1 - EMA2`

#### EMA

- **Input**: `close`
- **Output**: `Value::EmaValue(f64)`
- **Formula**:
  - seed with `SMA(close, period)`
  - `alpha = 2 / (period + 1)`
  - `EMA_t = alpha * close_t + (1 - alpha) * EMA_{t-1}`

`Ema` also exposes `get_slope()`, which reports the percentage change from the last confirmed EMA to the current EMA value.

#### EMA Cross

- **Input**: `close`
- **Output**: `Value::EmaCrossValue { short, long, trend }`
- **Formula**:
  - compute a short EMA and a long EMA
  - `trend = short >= long`

#### Ichimoku

- **Input**: `high`, `low`, `close`
- **Output**: `Value::IchimokuValue { tenkan, kijun, span_a, span_b, chikou }`
- **Formula**:
  - `tenkan = (highest_high(tenkan) + lowest_low(tenkan)) / 2`
  - `kijun = (highest_high(kijun) + lowest_low(kijun)) / 2`
  - `span_a = (tenkan + kijun) / 2`
  - `span_b = (highest_high(senkou_b) + lowest_low(senkou_b)) / 2`
  - `chikou = current close`

The crate returns the raw line values. Plotting offsets for senkou spans and chikou are left to the consumer.

#### SMA

- **Input**: `close`
- **Output**: `Value::SmaValue(f64)`
- **Formula**:
  - `SMA = mean(close, period)`

#### TEMA

- **Input**: `close`
- **Output**: `Value::TemaValue(f64)`
- **Formula**:
  - `EMA1 = EMA(close, period)`
  - `EMA2 = EMA(EMA1, period)`
  - `EMA3 = EMA(EMA2, period)`
  - `TEMA = 3 * EMA1 - 3 * EMA2 + EMA3`

### Volatility

#### ATR

- **Input**: `high`, `low`, `close`
- **Output**: `Value::AtrValue(f64)`
- **Formula**:
  - `TR = max(high - low, |high - prev_close|, |low - prev_close|)`
  - initial ATR is the mean of the warmup true ranges
  - afterward:
    - `ATR_t = ((ATR_{t-1} * (period - 1)) + TR_t) / period`

`Atr` also exposes `normalized(price)`, which returns ATR as a percentage of a supplied reference price.

#### Bollinger Bands

- **Input**: `close`
- **Output**: `Value::BollingerValue { upper, mid, lower, width }`
- **Formula**:
  - `mid = SMA(close, period)`
  - `stddev = sqrt(E[x^2] - E[x]^2)` using population variance over the rolling window
  - `upper = mid + std_multiplier * stddev`
  - `lower = mid - std_multiplier * stddev`
  - `width = ((upper - lower) / |mid|) * 100`

#### Historical Volatility

- **Input**: `close`
- **Output**: `Value::HistVolatilityValue(f64)`
- **Formula**:
  - `r_t = ln(close_t / close_{t-1})`
  - compute rolling sample standard deviation of `r_t`
  - annualize with:
    - `HV = stddev(log_returns, period) * sqrt(365) * 100`

### Volume

#### OBV

- **Input**: `close`, `vlm`
- **Output**: `Value::ObvValue(f64)`
- **Formula**:
  - if `close_t > close_{t-1}`: `OBV_t = OBV_{t-1} + volume_t`
  - if `close_t < close_{t-1}`: `OBV_t = OBV_{t-1} - volume_t`
  - otherwise: `OBV_t = OBV_{t-1}`
  - the implementation initializes from `0`

#### Volume MA

- **Input**: `vlm`
- **Output**: `Value::VolumeMaValue(f64)`
- **Formula**:
  - `VolumeMA = mean(volume, period)`

#### VWAP Deviation

- **Input**: `close`, `vlm`
- **Output**: `Value::VwapDeviationValue(f64)`
- **Formula**:
  - `VWAP = sum(price * volume) / sum(volume)` over the rolling window
  - `variance = sum(price^2 * volume) / sum(volume) - VWAP^2`
  - `stddev = sqrt(variance)`
  - `deviation = (close - VWAP) / stddev`

## Notes on update semantics

- `update_before_close` is for live, in-candle recalculation and may be called many times for the same candle
- `update_after_close` commits the candle and advances the rolling state
- `load(&[Price])` is equivalent to replaying a historical series through `update_after_close`

## Example

```rust
use kwant::indicators::{Ema, Indicator, Price, Value};

fn main() {
    let mut ema = Ema::new(3);

    let candles = vec![
        Price {
            open: 100.0,
            high: 102.0,
            low: 99.0,
            close: 101.0,
            open_time: 0,
            close_time: 0,
            vlm: 10.0,
        },
        Price {
            open: 101.0,
            high: 103.0,
            low: 100.0,
            close: 102.0,
            open_time: 1,
            close_time: 1,
            vlm: 11.0,
        },
        Price {
            open: 102.0,
            high: 104.0,
            low: 101.0,
            close: 103.0,
            open_time: 2,
            close_time: 2,
            vlm: 12.0,
        },
    ];

    ema.load(&candles);

    let live_price = Price {
        open: 103.0,
        high: 105.0,
        low: 102.0,
        close: 104.0,
        open_time: 3,
        close_time: 3,
        vlm: 13.0,
    };

    ema.update_before_close(live_price);

    if let Some(Value::EmaValue(value)) = ema.get_last() {
        println!("Provisional EMA: {value:.2}");
    }

    ema.update_after_close(live_price);

    if let Some(Value::EmaValue(value)) = ema.get_last() {
        println!("Confirmed EMA: {value:.2}");
    }
}
```

## Contributing

PRs are welcome. If you use Kwant in a live trading system and want to add indicators or tighten parity with an external charting platform, open an issue or send a patch.

## License
MIT
