# 📊 Kwant

**Kwant** is a lightweight, extensible technical indicators library for Rust—designed to work seamlessly with real-time price streams. It's built for high-performance trading bots and aims to match TradingView-calculated values with precision.

This library powers signal generation for the [`hyperliquid_rust_bot`](https://github.com/0xNoSystem/hyperliquid_rust_bot) and is designed for both live and backtest environments.

---

## ✨ Features

- 🔁 Live and backtest-compatible updates (`update_before_close` & `update_after_close`)
- ⚡ Efficient buffer-based design with `VecDeque`
- 📀 Indicators:
  - EMA
  - EMA Cross
  - RSI
  - ATR
  - SMA
  - SMA on RSI
  - Stochastic RSI
  - ADX (experimental)
- ✅ Same values as TradingView (verified manually)

---

## 📦 Installation

```toml
# Add to your Cargo.toml
depkwant = { git = "https://github.com/0xNoSystem/Kwant" }
```

---

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
```

---

### 🔸 EMA Crossover Strategy

```rust
use kwant::indicators::{Price, EmaCross};

fn main() {
    let mut cross = EmaCross::new(9, 21); // 9/21 EMA crossover

    let price = Price {
        open: 100.0,
        high: 105.0,
        low: 99.0,
        close: 102.0,
    };

    // Feed price (live or from history)
    let maybe_signal = cross.update_and_check_for_cross(price, true);

    if let Some(signal) = maybe_signal {
        if signal {
            println!("Bullish crossover detected ✅");
        } else {
            println!("Bearish crossover detected ❌");
        }
    }

    // Check current trend direction
    if let Some(uptrend) = cross.get_trend() {
        println!("Currently in {} trend", if uptrend { "up" } else { "down" });
    }
}
```

---

### 🔁 Load Historical Data (Backtest-style)

```rust
use kwant::indicators::{Price, Indicator, Ema};

fn main() {
    let price_history = vec![
        Price { open: 100.0, high: 102.0, low: 99.0, close: 101.0 },
        Price { open: 101.0, high: 103.0, low: 100.0, close: 102.0 },
        Price { open: 102.0, high: 104.0, low: 101.0, close: 103.0 },
        // ... more candles
    ];

    let mut ema = Ema::new(14);

    // Load all prices (after close) to initialize the indicator
    ema.load(&price_history);

    if let Some(ema_value) = ema.get_last() {
        println!("Loaded EMA: {:.2}", ema_value);
    }
}
```

---

## 🧠 Design Principles

- `Price` struct: `{ open, high, low, close }`
- Consistent trait `Indicator` with:
  - `update_before_close(price)`
  - `update_after_close(price)`
  - `get_last()` for the latest value
  - `load(&Vec<Price>)` for bulk updates

---

## 📚 Roadmap

- [ ] MACD
- [ ] Bollinger Bands
- [ ] OBV / CCI / VWAP
- [ ] Benchmarks & CI

---

## 👥 Contributing

PRs are welcome! If you're using Kwant in a trading bot and want to add indicators or improvements, feel free to open an issue or pull request.

---

## 📄 License

MIT



