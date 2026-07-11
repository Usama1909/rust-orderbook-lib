use crate::order::{
    Order, OrderId, OrderResult, OrderType, Price, PriceLevel, Quantity, Side, SlippageReport,
    Trade,
};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrderBookError {
    #[error("Duplicate order ID: {0}")]
    DuplicateOrderId(OrderId),

    #[error("Order not found: {0}")]
    OrderNotFound(OrderId),

    #[error("Invalid price for market order")]
    InvalidMarketOrderPrice,

    #[error("Symbol mismatch: expected {expected}, got {actual}")]
    SymbolMismatch { expected: String, actual: String },
}

pub struct OrderIdGenerator {
    counter: AtomicU64,
}

impl OrderIdGenerator {
    pub fn new() -> Self {
        OrderIdGenerator {
            counter: AtomicU64::new(1),
        }
    }
    pub fn next_id(&self) -> OrderId {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }
}
impl Default for OrderIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}
/// The order book for a single symbol
pub struct OrderBook {
    /// Buy orders - highest price first (we reverse the key)
    bids: BTreeMap<Price, VecDeque<Order>>,
    /// Sell orders - lowest price first
    asks: BTreeMap<Price, VecDeque<Order>>,
    /// Fast lookup of order by ID for cancelation
    order_index: HashMap<OrderId, (Side, Price)>,
    /// Symbol this handles
    pub symbol: String,
    id_generator: OrderIdGenerator,
}

impl OrderBook {
    /// create a new order book
    pub fn new(symbol: &str) -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            order_index: HashMap::new(),
            symbol: symbol.to_string(),
            id_generator: OrderIdGenerator::new(),
        }
    }

    /// Number of orders currently in the book
    pub fn order_count(&self) -> usize {
        self.order_index.len()
    }
    /// Best bid price (highest buy)
    pub fn best_bid(&self) -> Option<Price> {
        self.bids.keys().next_back().copied()
    }

    /// Best ask price (lowest sell)
    pub fn best_ask(&self) -> Option<Price> {
        self.asks.keys().next().copied()
    }

    /// Add an order to the resting book
    fn add_to_book(&mut self, order: Order) -> Result<(), OrderBookError> {
        let side = order.side.clone();
        let price = order.price.expect("add_to_book called with a market order");
        let id = order.id;

        if self.order_index.contains_key(&id) {
            return Err(OrderBookError::DuplicateOrderId(id));
        }

        match side {
            Side::Buy => self.bids.entry(price).or_default().push_back(order),
            Side::Sell => self.asks.entry(price).or_default().push_back(order),
        }
        self.order_index.insert(id, (side, price));
        Ok(())
    }

    /// Submit an order - matches immediately if possible, rest in book otherwise
    pub fn submit(&mut self, mut order: Order) -> Result<OrderResult, OrderBookError> {
        let mut trades = Vec::new();

        match order.side {
            Side::Buy => {
                // Match against asks - lowest ask first
                while order.quantity > 0 {
                    let best_ask_price = match self.asks.keys().next().copied() {
                        Some(p) => p,
                        None => break,
                    };
                    if let Some(limit_price) = order.price {
                        if limit_price < best_ask_price {
                            break;
                        }
                    }
                    let queue = self.asks.get_mut(&best_ask_price).unwrap();
                    let resting = queue.front_mut().unwrap();
                    let fill_qty = order.quantity.min(resting.quantity);
                    trades.push(Trade {
                        maker_order_id: resting.id,
                        taker_order_id: order.id,
                        price: best_ask_price,
                        quantity: fill_qty,
                        symbol: order.symbol.clone(),
                    });
                    order.quantity -= fill_qty;
                    resting.quantity -= fill_qty;
                    if resting.quantity == 0 {
                        let id = resting.id;
                        queue.pop_front();
                        if queue.is_empty() {
                            self.asks.remove(&best_ask_price);
                        }
                        self.order_index.remove(&id);
                    }
                }
            }
            Side::Sell => {
                // Match against bids - highest bid first
                while order.quantity > 0 {
                    let best_bid_price = match self.bids.keys().next_back().copied() {
                        Some(p) => p,
                        None => break,
                    };

                    if let Some(limit_price) = order.price {
                        if limit_price > best_bid_price {
                            break;
                        }
                    }
                    let queue = self.bids.get_mut(&best_bid_price).unwrap();
                    let resting = queue.front_mut().unwrap();
                    let fill_qty = order.quantity.min(resting.quantity);
                    trades.push(Trade {
                        maker_order_id: resting.id,
                        taker_order_id: order.id,
                        price: best_bid_price,
                        quantity: fill_qty,
                        symbol: order.symbol.clone(),
                    });
                    order.quantity -= fill_qty;
                    resting.quantity -= fill_qty;
                    if resting.quantity == 0 {
                        let id = resting.id;
                        queue.pop_front();
                        if queue.is_empty() {
                            self.bids.remove(&best_bid_price);
                        }
                        self.order_index.remove(&id);
                    }
                }
            }
        }
        if order.quantity > 0 {
            if order.order_type == OrderType::Market {
                // Market orders never rest in the book - discard whatever didn't fill
                if trades.is_empty() {
                    return Ok(OrderResult::Unfilled);
                } else {
                    return Ok(OrderResult::PartialFill(trades, order.quantity));
                }
            } else {
                // Limit orders rest in the book waiting for a match
                if trades.is_empty() {
                    self.add_to_book(order)?;
                    return Ok(OrderResult::Resting);
                } else {
                    let remaining = order.quantity;
                    self.add_to_book(order)?;
                    return Ok(OrderResult::PartialFill(trades, remaining));
                }
            }
        }
        Ok(OrderResult::Filled(trades))
    }

    pub fn submit_auto(
        &mut self,
        symbol: &str,
        side: Side,
        price: Price,
        quantity: Quantity,
    ) -> Result<OrderResult, OrderBookError> {
        let id = self.id_generator.next_id();
        let order = Order::new_limit(id, symbol, side, price, quantity);
        self.submit(order)
    }
    /// Cancel an order by ID. Returns true if found and removed.
    pub fn cancel_order(&mut self, id: OrderId) -> bool {
        let (side, price) = match self.order_index.remove(&id) {
            Some(entry) => entry,
            None => return false,
        };

        let book = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };

        if let Some(queue) = book.get_mut(&price) {
            queue.retain(|o| o.id != id);
            if queue.is_empty() {
                book.remove(&price);
            }
        }
        true
    }
    /// Total value of all resting Orders on the side
    pub fn total_bid_value(&self) -> u64 {
        self.bids
            .values()
            .flat_map(|queue| queue.iter())
            .map(|order| order.price.unwrap_or(0) * order.quantity)
            .sum()
    }

    /// All orders for a specific symbol (returns cloned vec)
    pub fn orders_for_symbol(&self, symbol: &str) -> Vec<Order> {
        self.bids
            .values()
            .chain(self.asks.values())
            .flat_map(|queue| queue.iter())
            .filter(|order| order.symbol == symbol)
            .cloned()
            .collect()
    }
    pub fn amend_order(
        &mut self,
        id: OrderId,
        new_price: Option<Price>,
        new_quantity: Quantity,
    ) -> Result<(), OrderBookError> {
        let (side, price) = self
            .order_index
            .get(&id)
            .cloned()
            .ok_or(OrderBookError::OrderNotFound(id))?;

        let book = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };

        let price_changed = matches!(new_price, Some(p) if p != price);

        if !price_changed {
            if let Some(queue) = book.get_mut(&price) {
                if let Some(order) = queue.iter_mut().find(|o| o.id == id) {
                    if new_quantity <= order.quantity {
                        order.quantity = new_quantity;
                        return Ok(());
                    }
                }
            }
        }

        // Price changed, or quantity increased - loses time priority.
        // Cancel and resubmit as a new resting order at the (possible new) price.
        let symbol = if let Some(queue) = book.get(&price) {
            queue
                .iter()
                .find(|o| o.id == id)
                .map(|o| o.symbol.clone())
                .ok_or(OrderBookError::OrderNotFound(id))?
        } else {
            return Err(OrderBookError::OrderNotFound(id));
        };
        self.cancel_order(id);
        let target_price = new_price.unwrap_or(price);
        self.submit(Order::new_limit(
            id,
            &symbol,
            side,
            target_price,
            new_quantity,
        ))?;
        Ok(())
    }
    /// Count of bids at a speicifc price level
    pub fn depth_at_price(&self, price: Price) -> usize {
        self.bids.get(&price).map(|queue| queue.len()).unwrap_or(0)
    }
    /// Returns top `depth` price levels on each side: (bids, asks)
    pub fn book_snapshot(&self, depth: usize) -> (Vec<PriceLevel>, Vec<PriceLevel>) {
        let bid_levels = self
            .bids
            .iter()
            .rev()
            .take(depth)
            .map(|(price, queue)| PriceLevel {
                price: *price,
                quantity: queue.iter().map(|o| o.quantity).sum(),
            })
            .collect();

        let ask_levels = self
            .asks
            .iter()
            .take(depth)
            .map(|(price, queue)| PriceLevel {
                price: *price,
                quantity: queue.iter().map(|o| o.quantity).sum(),
            })
            .collect();

        (bid_levels, ask_levels)
    }

    /// Analyze slippage for a set of executed trades against an expected price
    pub fn analyze_slippage(trades: &[Trade], expected_price: Price) -> Option<SlippageReport> {
        if trades.is_empty() {
            return None;
        }

        let total_quantity: Quantity = trades.iter().map(|t| t.quantity).sum();

        let weighted_price: u64 =
            trades.iter().map(|t| t.price * t.quantity).sum::<u64>() / total_quantity;

        let slippage_ticks = weighted_price as i64 - expected_price as i64;

        Some(SlippageReport {
            symbol: trades[0].symbol.clone(),
            expected_price,
            actual_avg_price: weighted_price,
            total_quantity,
            slippage_ticks,
        })
    }
}
