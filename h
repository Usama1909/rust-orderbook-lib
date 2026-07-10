warning: in the working copy of 'examples/slippage_demo.rs', LF will be replaced by CRLF the next time Git touches it
[1mdiff --git a/examples/slippage_demo.rs b/examples/slippage_demo.rs[m
[1mindex ae9f978..37b25e5 100644[m
[1m--- a/examples/slippage_demo.rs[m
[1m+++ b/examples/slippage_demo.rs[m
[36m@@ -1,24 +1,27 @@[m
 use rust_orderbook_lib::order::{Order, OrderResult, Side};[m
[31m-use rust_orderbook_lib::orderbook::{OrderBook};[m
[32m+[m[32muse rust_orderbook_lib::orderbook::OrderBook;[m
 [m
 fn main() {[m
     let mut book = OrderBook::new("BTCUSD");[m
[31m-    [m
[32m+[m
     // Build resting ask-side liquidity at three proce levels.[m
     // Thin liquidity at the top, more further away[m
[31m-    book.submit(Order::new_limit(1, "BTCUSD", Side::Sell, 50_000, 2)).unwrap();[m
[31m-    book.submit(Order::new_limit(2, "BTCUSD", Side::Sell, 50_010, 3)).unwrap();[m
[31m-    book.submit(Order::new_limit(3, "BTCUSD", Side::Sell, 50_025, 10)).unwrap();[m
[31m-    [m
[32m+[m[32m    book.submit(Order::new_limit(1, "BTCUSD", Side::Sell, 50_000, 2))[m
[32m+[m[32m        .unwrap();[m
[32m+[m[32m    book.submit(Order::new_limit(2, "BTCUSD", Side::Sell, 50_010, 3))[m
[32m+[m[32m        .unwrap();[m
[32m+[m[32m    book.submit(Order::new_limit(3, "BTCUSD", Side::Sell, 50_025, 10))[m
[32m+[m[32m        .unwrap();[m
[32m+[m
     println!("Book built: 2 @ 50000, 3 @ 50010, 10 @ 50025 (asks)\n");[m
[31m-    [m
[32m+[m
     // A market but for 8 lots has to walk through mutliple price levels[m
     // to fill, slice the best level only has 2 lots availble.[m
     let expected_price = 50_000; // price a naive trader would expect[m
     let order = Order::new_market(4, "BTCUSD", Side::Buy, 8);[m
[31m-    [m
[32m+[m
     let result = book.submit(order).unwrap();[m
[31m-    [m
[32m+[m
     let trades = match result {[m
         OrderResult::Filled(trades) => trades,[m
         OrderResult::PartialFill(trades, remaining) => {[m
[36m@@ -34,12 +37,12 @@[m [mfn main() {[m
             return;[m
         }[m
     };[m
[31m-    [m
[32m+[m
     println!("Expected {} trades:", trades.len());[m
     for t in &trades {[m
         println!(" {} lots @ {}", t.quantity, t.price);[m
     }[m
[31m-    [m
[32m+[m
     match OrderBook::analyze_slippage(&trades, expected_price) {[m
         Some(report) => {[m
             println!("\nSlippage report:");[m
[36m@@ -50,4 +53,4 @@[m [mfn main() {[m
         }[m
         None => println!("No trades to analyze"),[m
     }[m
[31m-}[m
\ No newline at end of file[m
[32m+[m[32m}[m
