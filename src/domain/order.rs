
use std::collections::{BTreeMap, VecDeque};
use crate::domain::trade::Trade;

pub struct OrderBook {
    bids: BTreeMap<u64, VecDeque<Order>>,
    asks: BTreeMap<u64, VecDeque<Order>>
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new()
        }
    }
    pub fn add_order(&mut self, order: Order) -> Vec<Trade> {
        if order.is_buy() {
            if !self.bids.contains_key(&order.price){
                self.bids.insert(order.price, VecDeque::<Order>::new());
            }
            self.bids.get_mut(&order.price).unwrap().push_back(order);
        }
        else {
            if !self.asks.contains_key(&order.price) {
                self.asks.insert(order.price, VecDeque::<Order>::new());
            }
            self.asks.get_mut(&order.price).unwrap().push_back(order);
        }
        return Vec::new()
    }

    pub fn cancel_order(&mut self, order_id: u64) {

    }

    pub fn best_bid(&self) -> Option<u64> {
        self.bids.last_key_value().map(|(price, _)| *price)
    }
    pub fn best_ask(&self) -> Option<u64> {
        self.asks.first_key_value().map(|(price, _)| *price)
    }



}

// public class Order
#[derive(Debug, PartialEq, Clone)]
pub struct Order {
    pub id: u64, // int Id
    pub side: Side,
    pub price: u64, // int price
    pub quantity: u64  // int quantity
}

impl Order {
    pub fn is_buy(&self) -> bool {
        self.side == Side::Buy
    }
}

// public enum Side
#[derive(Debug, PartialEq, Clone)]
pub enum Side {
    Buy,
    Sell
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_add_bid() {
        let mut book = OrderBook::new();
        let order = Order {
            id: 1,
            side: Side::Buy,
            price: 100,
            quantity: 10
        };
        book.add_order(order);
        assert_eq!(book.best_bid(), Some(100));
    }

    #[test]
    fn test_add_ask() {
        let mut book = OrderBook::new();
        let order = Order {
            id: 1,
            side: Side::Sell,
            price: 100,
            quantity: 10
        };
        book.add_order(order);
        assert_eq!(book.best_ask(), Some(100));
    }

    #[test]
    fn test_best_bid_multiple() {
        let mut book = OrderBook::new();
        book.add_order(Order { id: 1, side: Side::Buy, price: 100, quantity: 10 });
        book.add_order(Order { id: 2, side: Side::Buy, price: 105, quantity: 10 });
        book.add_order(Order { id: 3, side: Side::Buy, price: 102, quantity: 10 });
        assert_eq!(book.best_bid(), Some(105)); // highest bid
    }

    #[test]
    fn test_best_ask_multiple() {
        let mut book = OrderBook::new();
        book.add_order(Order { id: 1, side: Side::Sell, price: 100, quantity: 10 });
        book.add_order(Order { id: 2, side: Side::Sell, price: 105, quantity: 10 });
        book.add_order(Order { id: 3, side: Side::Sell, price: 102, quantity: 10 });
        assert_eq!(book.best_ask(), Some(100)); // lowest ask
    }

    #[test]
    fn test_fifo_same_price() {
        let mut book = OrderBook::new();
        let order1 = Order { id: 1, side: Side::Buy, price: 100, quantity: 10 };
        let order2 = Order { id: 2, side: Side::Buy, price: 100, quantity: 5 };

        book.add_order(order1);
        book.add_order(order2);

        // best_bid should still be 100
        assert_eq!(book.best_bid(), Some(100));
    }
}
