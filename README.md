# rust-orderbook-lib

A production-grade order book library in Rust with price-time priority matching.

## Features
- Integer prices and quantities — no floating point
- Price-time priority matching (BTreeMap + VecDeque)
- Partial fills
- Cancel orders by ID
- Trade events on every match
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
book.submit(Order::new(1, "NVDA", Side::Buy, 500, 10));
book.submit(Order::new(2, "NVDA", Side::Sell, 490, 10));
```

## License
MIT OR Apache-2.0
