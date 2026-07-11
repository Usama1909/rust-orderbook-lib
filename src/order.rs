/// Unique identifier for each order
pub type OrderId = u64;

/// Price in ticks (integer, no floats ever)
pub type Price = u64;

///Quantity in lots (integer)
pub type Quantity = u64;

use std::sync::atomic::{AtomicU64, Ordering};

/// thread-safe generator for unique order Ids
pub struct OrderIdGenerator {
    next_id: AtomicU64,
}

impl OrderIdGenerator {
    /// Create a new generator starting at 1
    pub fn new() -> Self {
        OrderIdGenerator {
            next_id: AtomicU64::new(1),
        }
    }
    /// Get the next unique IF - safe to call from multiple thread at once
    pub fn next_id(&self) -> OrderId {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
}
impl Default for OrderIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Side of the order
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

/// Type of order - Limit waits in book, Market fills immediately or discards
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    Market,
}
#[derive(Debug, Clone)]
pub struct Order {
    pub id: OrderId,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Option<Price>,
    pub quantity: Quantity,
}
impl Order {
    pub fn new_limit(
        id: OrderId,
        symbol: &str,
        side: Side,
        price: Price,
        quantity: Quantity,
    ) -> Self {
        Self {
            id,
            symbol: symbol.to_string(),
            side,
            order_type: OrderType::Limit,
            price: Some(price),
            quantity,
        }
    }
    pub fn new_market(id: OrderId, symbol: &str, side: Side, quantity: Quantity) -> Self {
        Self {
            id,
            symbol: symbol.to_string(),
            side,
            order_type: OrderType::Market,
            price: None,
            quantity,
        }
    }
}

/// A trade that occurred when teo orders matched
#[derive(Debug, Clone)]
pub struct Trade {
    pub maker_order_id: OrderId,
    pub taker_order_id: OrderId,
    pub price: Price,
    pub quantity: Quantity,
    pub symbol: String,
}
/// Result of submitting an order
#[derive(Debug)]
pub enum OrderResult {
    ///Order fully filled - list of trades
    Filled(Vec<Trade>),
    /// Order partially filled - trades so far, remaining quantity
    PartialFill(Vec<Trade>, Quantity),
    Unfilled,
    /// Order added to book with no immediate match
    Resting,
}
// Analysis of execution quality for a set of trades
#[derive(Debug, Clone)]
pub struct SlippageReport {
    pub symbol: String,
    pub expected_price: Price,
    pub actual_avg_price: Price,
    pub total_quantity: Quantity,
    pub slippage_ticks: i64,
}

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: Price,
    pub quantity: Quantity,
}

impl SlippageReport {
    pub fn slippage_pct(&self) -> f64 {
        (self.slippage_ticks.abs() as f64 / self.expected_price as f64) * 100.0
    }
}
