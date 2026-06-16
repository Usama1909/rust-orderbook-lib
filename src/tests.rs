#[cfg(test)]
mod tests {
    use crate::order::{Order, Side};
    use crate::orderbook::OrderBook;

    fn buy(id: u64, price: u64, qty: u64) -> Order {
        Order::new(id, "NVDA", Side::Buy, price, qty)
    }

    fn sell(id: u64, price: u64, qty: u64) -> Order {
        Order::new(id, "NVDA", Side::Sell, price, qty)
    }
    #[test]
    fn test_full_fill() {
        let mut book = OrderBook::new("NVDA");
        book.submit(sell(1, 100, 10));
        let result = book.submit(buy(2, 100, 10));
        assert!(matches!(result, crate::order::OrderResult::Filled(_)));
        assert_eq!(book.order_count(), 0);
    }
    #[test]
    fn test_no_cross_no_match() {
        let mut book = OrderBook::new("NVDA");
        book.submit(sell(1, 200, 10));
        let result = book.submit(buy(2, 100, 10));
        assert!(matches!(result, crate::order::OrderResult::Resting));
        assert_eq!(book.order_count(), 2);
    }
    #[test]
    fn test_cancel_order() {
        let mut book = OrderBook::new("NVDA");
        book.submit(buy(1, 100, 10)); // rests in book, no match
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
        book.submit(sell(1, 100, 10));
        let result = book.submit(buy(2, 100, 5));
        assert!(matches!(result, crate::order::OrderResult::Filled(_)));
        assert_eq!(book.order_count(), 1); // sell has 5 remaining
    }

    #[test]
    fn test_price_time_priority() {
        let mut book = OrderBook::new("NVDA");
        book.submit(sell(1, 100, 10));
        book.submit(sell(2, 100, 10));
        let result = book.submit(buy(3, 100, 15));
        if let crate::order::OrderResult::PartialFill(trades, _) = result {
            assert_eq!(trades[0].maker_order_id, 1); // order 1 fills first
        }
    }

    #[test]
    fn test_better_price_first() {
        let mut book = OrderBook::new("NVDA");
        book.submit(sell(1, 101, 10));
        book.submit(sell(2, 100, 10));
        let result = book.submit(buy(3, 110, 10));
        if let crate::order::OrderResult::Filled(trades) = result {
            assert_eq!(trades[0].price, 100); // cheaper ask fills first
        }
    }

    #[test]
    fn test_multi_trade_match() {
        let mut book = OrderBook::new("NVDA");
        book.submit(sell(1, 100, 5));
        book.submit(sell(2, 100, 5));
        let result = book.submit(buy(3, 100, 10));
        if let crate::order::OrderResult::Filled(trades) = result {
            assert_eq!(trades.len(), 2); // two trades generated
        }
        assert_eq!(book.order_count(), 0);
    }
    #[test]
    fn test_best_bid_ask() {
        let mut book = OrderBook::new("NVDA");
        book.submit(buy(1, 100, 10));
        book.submit(buy(2, 105, 10));
        book.submit(sell(3, 110, 10));
        assert_eq!(book.best_bid(), Some(105));
        assert_eq!(book.best_ask(), Some(110));
    }
}
