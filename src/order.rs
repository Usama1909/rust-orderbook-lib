/// Unique identifier for each order
pub type OrderId = u64;

/// Price in ticks (integer, no floats ever)
pub type Price = u64;

///Quantity in lots (integer)
pub type Quantity = u64;

/// Side of the order
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

/// A signal order in the book
#[derive(Debug, Clone)]
pub struct Order {
    pub id: OrderId,
    pub price: Price,
    pub quantity: Quantity,
    pub side: Side,
    pub symbol: String,
}
impl Order {
    ///Create a new order
    pub fn new(id: OrderId, symbol: &str, side: Side, price: Price, quantity: Quantity) -> Self {
        Order {
            id,
            price,
            quantity,
            side,
            symbol: symbol.to_string(),
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
    /// Order added to book with no immediate match
    Resting,
}
