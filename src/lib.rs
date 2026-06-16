//! # rust-orderbook-lib
//!
//! A production_grade order book library with price-time priority matching.

//! ## Example
//! ```
//! use rust_orderbook_lib::order::{Order, Side};
//! use rust_orderbook_lib::orderbook::OrderBook;
//!
//! let mut book = OrderBook::new("NVDA");
//! let order = Order::new(1, "NVDA", Side::Buy, 500, 10);
//! book.submit(order);
//! ```
pub mod order;
pub mod orderbook;
mod tests;
