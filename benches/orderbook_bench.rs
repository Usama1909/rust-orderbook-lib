use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_orderbook_lib::order::{Order, Side};
use rust_orderbook_lib::orderbook::OrderBook;

fn bench_submit_resting(c: &mut Criterion) {
    c.bench_function("submit resting order", |b| {
        b.iter(|| {
            let mut book = OrderBook::new("NVDA");
            book.submit(black_box(Order::new(1, "NVDA", Side::Buy, 100, 10)));
        })
    });
}

fn bench_submit_matching(c: &mut Criterion) {
    c.bench_function("submit matching order", |b| {
        b.iter(|| {
            let mut book = OrderBook::new("NVDA");
            book.submit(Order::new(1, "NVDA", Side::Sell, 100, 10));
            book.submit(black_box(Order::new(2, "NVDA", Side::Buy, 100, 10)));
        })
    });
}

fn bench_cancel(c: &mut Criterion) {
    c.bench_function("cancel order", |b| {
        b.iter(|| {
            let mut book = OrderBook::new("NVDA");
            book.submit(Order::new(1, "NVDA", Side::Buy, 100, 10));
            book.cancel_order(black_box(1));
        })
    });
}

criterion_group!(
    benches,
    bench_submit_resting,
    bench_submit_matching,
    bench_cancel
);
criterion_main!(benches);
