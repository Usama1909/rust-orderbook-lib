use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_orderbook_lib::order::{Order, Side};
use rust_orderbook_lib::orderbook::OrderBook;

/// Build a realistic resting book: `levels` price levels per side,
/// `orders_per_level` orders queued at each price.
fn build_deep_book(levels: u64, orders_per_level: u64) -> OrderBook {
    let mut book = OrderBook::new("NVDA");
    let mut id = 1;

    for level in 0..levels {
        let bid_price = 1000 - level; // bids descending from 1000
        let ask_price = 1001 + level; // asks ascending from 1001
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

fn bench_submit_resting_deep_book(c: &mut Criterion) {
    c.bench_function("submit resting order (1000 levels, deep book)", |b| {
        b.iter_batched(
            || build_deep_book(1000, 5),
            |mut book| {
                book.submit(black_box(Order::new_limit(
                    999_999,
                    "NVDA",
                    Side::Buy,
                    500,
                    10,
                )))
                .unwrap();
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_submit_matching_deep_book(c: &mut Criterion) {
    c.bench_function("submit matching order (1000 levels, deep book)", |b| {
        b.iter_batched(
            || build_deep_book(1000, 5),
            |mut book| {
                book.submit(black_box(Order::new_limit(
                    999_999,
                    "NVDA",
                    Side::Buy,
                    1001,
                    10,
                )))
                .unwrap();
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_cancel_deep_book(c: &mut Criterion) {
    c.bench_function("cancel order (1000 levels, deep book)", |b| {
        b.iter_batched(
            || build_deep_book(1000, 5),
            |mut book| {
                book.cancel_order(black_box(1));
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_submit_resting_deep_book,
    bench_submit_matching_deep_book,
    bench_cancel_deep_book
);
criterion_main!(benches);
