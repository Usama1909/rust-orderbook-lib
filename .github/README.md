# rust-orderbook-lib

A production-grade order book library in Rust with price-time priority matching and execution quality analysis.

![CI](https://github.com/Usama1909/rust-orderbook-lib/actions/workflows/ci.yml/badge.svg)

## Features
- Integer prices and quantities — no floating point
- Price-time priority matching (BTreeMap + VecDeque)
- Limit orders and market orders
- Partial fills
- Cancel orders by ID
- Trade events on every match
- Slippage analysis — weighted fill price vs expected price
- Thread-safe order ID generation
- 20 passing tests

## Performance
| Operation | Time |
|-----------|------|
| Submit resting order | ~900ns |
| Submit matching order | ~1.3µs |
| Cancel order | ~870ns |

## Complexity
| Operation | Complexity |
|-----------|------------|
| Add order | O(log n) |
| Cancel order | O(log n) |
| Match order | O(log n) |

## Usage
```rust
use rust_orderbook_lib::order::{Order, Side};
use rust_orderbook_lib::orderbook::OrderBook;

let mut book = OrderBook::new("NVDA");
book.submit(Order::new_limit(1, "NVDA", Side::Buy, 500, 10));
book.submit(Order::new_limit(2, "NVDA", Side::Sell, 490, 10));
```

## Slippage Analysis
```rust
let report = OrderBook::analyze_slippage(&trades, 500).unwrap();
println!("Slippage: {} ticks ({:.2}%)", 
    report.slippage_ticks, 
    report.slippage_pct());
```

## License
MIT OR Apache-2.0
