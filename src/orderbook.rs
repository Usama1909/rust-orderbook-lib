use crate::order::{Order, OrderId, OrderResult, Price,Quantity,  Side, Trade};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub enum OrderBookError {
    DuplicateOrderId(OrderId)
}

pub struct OrderIdGenerator {
    counter: AtomicU64,
}

impl OrderIdGenerator {
    pub fn new() -> Self {
        OrderIdGenerator { counter: AtomicU64::new(1) }
    }
    pub fn next_id(&self) -> OrderId {
        self.counter.fetch_add(1, Ordering::Relaxed)
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
    fn add_to_book(&mut self, order: Order) -> Result<(),  OrderBookError> {
    let side = order.side.clone();
    let price = order.price;
    let id = order.id;
        
    if self.order_index.contains_key(&id) {
        return Err(OrderBookError::DuplicateOrderId(id));
        }

    
    match side {
        Side::Buy => self
            .bids.entry(price)
            .or_default()
            .push_back(order),
        Side::Sell => self
            .asks.entry(price)
            .or_default()
            .push_back(order),
    }
    self.order_index.insert(id, (side, price));
    Ok(())
    }

    /// Submit an order - matches immediately if possible, rest in book otherwise
    pub fn submit(&mut self, mut order: Order) -> Result<OrderResult, OrderBookError>{
        let mut trades = Vec::new();

        match order.side {
            Side::Buy => {
                // Match against asks - lowest ask first
                while order.quantity > 0 {
                    let best_ask_price = match self.asks.keys().next().copied() {
                        Some(p) => p,
                        None => break,
                    };
                    if order.price < best_ask_price {
                        break;
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
                    if order.price > best_bid_price {
                        break;
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
            if trades.is_empty() {
                self.add_to_book(order)?;
                return Ok(OrderResult::Resting);
            } else {
                let remaining = order.quantity;
                self.add_to_book(order)?;
                return Ok(OrderResult::PartialFill(trades, remaining));
            }
        }
        Ok(OrderResult::Filled(trades))
    }
    
    pub fn submit_auto(&mut self, symbol:&str, side:Side, price: Price, quantity: Quantity) -> Result<OrderResult,  OrderBookError> {
        let id = self.id_generator.next_id();
        let order = Order::new(id, symbol, side, price, quantity);
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
}