# 📊 Kwant

**Kwant** is a lightweight, extensible technical indicators library for Rust—designed to work seamlessly with real-time candle streams and built for high-performance trading bots, including support for live and backtest use cases.

This library powers signal generation for the [`hyperliquid_rust_bot`](https://github.com/0xNoSystem/hyperliquid_rust_bot) and aims to match TradingView-calculated values for precise parity in production.

---

## ✨ Features

- 🔁 Supports both `update_before_close` (live intrabar) and `update_after_close` (on close) logic.
- ⏱ Real-time streaming compatible: designed around `VecDeque` buffers and async-ready updates.
- 📐 Includes commonly used indicators:
  - RSI (with optional smoothing)
  - ATR
  - EMA
  - EMA Cross
  - SMA
  - SMA on RSI
  - Stochastic RSI
  - ADX (in progress / experimental)
- ✅ Unit-tested with comparisons against TradingView results.

---

## 📦 Installation

```toml
# Add to your Cargo.toml
kwant = { git = "https://github.com/0xNoSystem/Kwant" }


## 🚀 Usage

### 🔹 Exponential Moving Average (EMA)

```rust
use kwant::indicators::{Price, Indicator, Ema};

fn main() {
    let mut ema = Ema::new(20); // 20-period EMA

    let price = Price {
        open: 100.0,
        high: 105.0,
        low: 99.0,
        close: 102.0,
    };

    // Optional: update mid-candle (e.g. during live streaming)
    ema.update_before_close(price);

    // Finalize candle at close
    ema.update_after_close(price);

    if let Some(value) = ema.get_last() {
        println!("Current EMA: {:.2}", value);
    }

    if let Some(slope) = ema.get_slope() {
        println!("Current EMA Slope: {:.2}%", slope);
    }
}


