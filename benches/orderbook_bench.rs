use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_orderbook_lib::order::{Order, Side};
use rust_orderbook_lib::orderbook::OrderBook;

/// Realistic resting book: `levels` price levels per side,
/// `orders_per_level` orders queued at each price (1000 x 5 = 10,000 orders).
fn build_deep_book(levels: u64, orders_per_level: u64) -> OrderBook {
    let mut book = OrderBook::new("NVDA");
    let mut id = 1;
    for level in 0..levels {
        let bid_price = 1000 - level;
        let ask_price = 1001 + level;
        for _ in 0..orders_per_level {
            book.submit(Order::new_limit(id, "NVDA", Side::Buy, bid_price, 10))
                .unwrap();
            id += 1;
            book.submit(Order::new_limit(id, "NVDA", Side::Sell, ask_price, 10))
                .unwrap();
            id += 1;
        }
    }
    book
}

// Book built ONCE outside the timed loop. Each iteration is net-neutral, so we
// measure the operation on a steady 1000-level / 10k-order book, not its construction.

fn bench_submit_cancel(c: &mut Criterion) {
    let mut book = build_deep_book(1000, 5);
    let mut id = 1_000_000u64;
    c.bench_function(
        "submit + cancel resting order (1000-level, 10k-order book)",
        |b| {
            b.iter(|| {
                book.submit(black_box(Order::new_limit(id, "NVDA", Side::Buy, 500, 10)))
                    .unwrap();
                book.cancel_order(black_box(id));
                id += 1;
            })
        },
    );
}

fn bench_match(c: &mut Criterion) {
    let mut book = build_deep_book(1000, 5);
    let mut id = 2_000_000u64;
    c.bench_function(
        "match at top of book + reseed (1000-level, 10k-order book)",
        |b| {
            b.iter(|| {
                book.submit(black_box(Order::new_limit(id, "NVDA", Side::Buy, 1001, 10)))
                    .unwrap();
                book.submit(black_box(Order::new_limit(
                    id + 1,
                    "NVDA",
                    Side::Sell,
                    1001,
                    10,
                )))
                .unwrap();
                id += 2;
            })
        },
    );
}

criterion_group!(benches, bench_submit_cancel, bench_match);
criterion_main!(benches);
