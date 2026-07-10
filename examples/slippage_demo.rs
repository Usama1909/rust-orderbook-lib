use rust_orderbook_lib::order::{Order, OrderResult, Side};
use rust_orderbook_lib::orderbook::OrderBook;

fn main() {
    let mut book = OrderBook::new("BTCUSD");

    // Build resting ask-side liquidity at three proce levels.
    // Thin liquidity at the top, more further away
    book.submit(Order::new_limit(1, "BTCUSD", Side::Sell, 50_000, 2))
        .unwrap();
    book.submit(Order::new_limit(2, "BTCUSD", Side::Sell, 50_010, 3))
        .unwrap();
    book.submit(Order::new_limit(3, "BTCUSD", Side::Sell, 50_025, 10))
        .unwrap();

    println!("Book built: 2 @ 50000, 3 @ 50010, 10 @ 50025 (asks)\n");

    // A market but for 8 lots has to walk through mutliple price levels
    // to fill, slice the best level only has 2 lots availble.
    let expected_price = 50_000; // price a naive trader would expect
    let order = Order::new_market(4, "BTCUSD", Side::Buy, 8);

    let result = book.submit(order).unwrap();

    let trades = match result {
        OrderResult::Filled(trades) => trades,
        OrderResult::PartialFill(trades, remaining) => {
            println!("Warning: order partially filled. {remaining} lots unfilled");
            trades
        }
        OrderResult::Unfilled => {
            println!("Order unfilled - no liquidity");
            return;
        }
        OrderResult::Resting => {
            println!("Order resting = should not happen for aa market order");
            return;
        }
    };

    println!("Expected {} trades:", trades.len());
    for t in &trades {
        println!(" {} lots @ {}", t.quantity, t.price);
    }

    match OrderBook::analyze_slippage(&trades, expected_price) {
        Some(report) => {
            println!("\nSlippage report:");
            println!("  Expected price:   {}", report.expected_price);
            println!("  Actual avg price: {}", report.actual_avg_price);
            println!("  Slippage:         {} ticks", report.slippage_ticks);
            println!("  Slippage:         {:.3}%", report.slippage_pct());
        }
        None => println!("No trades to analyze"),
    }
}
