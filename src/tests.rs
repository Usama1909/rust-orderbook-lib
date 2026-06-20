#[cfg(test)]
mod tests {
    use crate::order::{Order, Side};
    use crate::orderbook::{OrderBook, OrderBookError};

    fn buy(id: u64, price: u64, qty: u64) -> Order {
        Order::new_limit(id, "NVDA", Side::Buy, price, qty)
    }

    fn sell(id: u64, price: u64, qty: u64) -> Order {
        Order::new_limit(id, "NVDA", Side::Sell, price, qty)
    }
    #[test]
    fn test_full_fill() {
        let mut book = OrderBook::new("NVDA");
        book.submit(sell(1, 100, 10)).unwrap();
        let result = book.submit(buy(2, 100, 10));
        assert!(matches!(result, Ok(crate::order::OrderResult::Filled(_))));
        assert_eq!(book.order_count(), 0);
    }
    #[test]
    fn test_no_cross_no_match() {
        let mut book = OrderBook::new("NVDA");
        book.submit(sell(1, 200, 10)).unwrap();
        let result = book.submit(buy(2, 100, 10));
        println!("DEBUG: {:?}", result);
        assert!(matches!(result, Ok(crate::order::OrderResult::Resting)));
        assert_eq!(book.order_count(), 2);
    }
    #[test]
    fn test_cancel_order() {
        let mut book = OrderBook::new("NVDA");
        book.submit(buy(1, 100, 10)).unwrap(); // rests in book, no match
        assert_eq!(book.order_count(), 1);
        assert!(book.cancel_order(1));
        assert_eq!(book.order_count(), 0);
    }
    #[test]
    fn test_cancel_nonexistent() {
        let mut book = OrderBook::new("NVDA");
        assert!(!book.cancel_order(99));
    }
    #[test]
    fn test_partial_fill() {
        let mut book = OrderBook::new("NVDA");
        book.submit(sell(1, 100, 10)).unwrap();
        let result = book.submit(buy(2, 100, 5));
        assert!(matches!(result, Ok(crate::order::OrderResult::Filled(_))));
        assert_eq!(book.order_count(), 1); // sell has 5 remaining
    }

    #[test]
    fn test_price_time_priority() {
        let mut book = OrderBook::new("NVDA");
        book.submit(sell(1, 100, 10)).unwrap();
        book.submit(sell(2, 100, 10)).unwrap();
        let result = book.submit(buy(3, 100, 15));
        if let Ok(crate::order::OrderResult::PartialFill(trades, _)) = result {
            assert_eq!(trades[0].maker_order_id, 1); // order 1 fills first
        }
    }

    #[test]
    fn test_better_price_first() {
        let mut book = OrderBook::new("NVDA");
        book.submit(sell(1, 101, 10)).unwrap();
        book.submit(sell(2, 100, 10)).unwrap();
        let result = book.submit(buy(3, 110, 10));
        if let Ok(crate::order::OrderResult::Filled(trades)) = result {
            assert_eq!(trades[0].price, 100); // cheaper ask fills first
        }
    }

    #[test]
    fn test_multi_trade_match() {
        let mut book = OrderBook::new("NVDA");
        book.submit(sell(1, 100, 5)).unwrap();
        book.submit(sell(2, 100, 5)).unwrap();
        let result = book.submit(buy(3, 100, 10));
        if let Ok(crate::order::OrderResult::Filled(trades)) = result {
            assert_eq!(trades.len(), 2); // two trades generated
        }
        assert_eq!(book.order_count(), 0);
    }
    #[test]
    fn test_best_bid_ask() {
        let mut book = OrderBook::new("NVDA");
        book.submit(buy(1, 100, 10)).unwrap();
        println!("after order 1: count={}", book.order_count());
        book.submit(buy(2, 105, 10)).unwrap();
        println!("after order 2: count={}", book.order_count());
        book.submit(sell(3, 110, 10)).unwrap();
        println!("after order 3: count={}", book.order_count());
        println!("best_bid={:?}", book.best_bid());
        println!("best_ask={:?}", book.best_ask());
        assert_eq!(book.best_bid(), Some(105));
        assert_eq!(book.best_ask(), Some(110));
    }
    #[test]
    fn test_duplicate_id_rejected() {
        let mut book = OrderBook::new("NVDA");
        book.submit(buy(1, 100, 10)).unwrap();
        let result = book.submit(buy(1, 100, 10));
        assert!(matches!(result, Err(OrderBookError::DuplicateOrderId(1))));
    }
    #[test]
    fn test_concurrent_submit_auto_no_dulicate_ids() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let book = Arc::new(Mutex::new(OrderBook::new("NVDA")));
        let mut handles = vec![];

        for _ in 0..10 {
            let book = Arc::clone(&book);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    let mut book = book.lock().unwrap();
                    book.submit_auto("NVDA", Side::Buy, 100, 1).unwrap();
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        let book = book.lock().unwrap();
        assert_eq!(book.order_count(), 1000);
    }
}
