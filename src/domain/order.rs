
use std::collections::{BTreeMap, HashMap, VecDeque};
use crate::domain::trade::Trade;

#[derive(Debug)]
pub struct OrderBook {
    bids: BTreeMap<u64, VecDeque<Order>>,
    asks: BTreeMap<u64, VecDeque<Order>>,
    order_index: HashMap<u64, (Side, u64)>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            order_index: HashMap::new()
        }
    }
    pub fn add_order(&mut self, mut order: Order) -> Vec<Trade> {
        let mut trades = Vec::new();

        if order.is_buy() {
            // try to fill the whole buy order with whatever sells are avaialble
            while order.quantity > 0 && self.best_ask().is_some() {
                let best_ask_price = self.best_ask().unwrap();

                if order.price >= best_ask_price {
                    // get deque of orders at the current best price
                    if let Some(ask_queue) = self.asks.get_mut(&best_ask_price) {
                        // Get the front order (FIFO, oldest first)
                        if let Some(mut resting_order) = ask_queue.pop_front() {
                            // Determine fill quantity
                            let fill_qty = order.quantity.min(resting_order.quantity);

                            // Create trade event
                            trades.push(Trade {
                                sell_order_id: resting_order.id,
                                buy_order_id: order.id,
                                price: best_ask_price,
                                quantity: fill_qty
                            });

                            // Reduce quantities
                            order.quantity -= fill_qty;
                            resting_order.quantity -= fill_qty;

                            // If resting order not fully filled, put it back
                            if resting_order.quantity > 0 {
                                ask_queue.push_front(resting_order);
                            }

                            // If price level now empty, remove it
                            if ask_queue.is_empty() {
                                self.asks.remove(&best_ask_price);
                            }
                        }
                    }
                } else {
                    // if order price is too low for the best sell price, end here, then add the remaining amount of the order to the bid queue
                    break;
                }
            }
        } else {
            // Match against bids (similar logic reversed)
            while order.quantity > 0 && self.best_bid().is_some() {
                let best_bid_price = self.best_bid().unwrap();

                if order.price <= best_bid_price {
                    if let Some(bid_queue) = self.bids.get_mut(&best_bid_price) {
                        if let Some(mut resting_order) = bid_queue.pop_front() {
                            let fill_qty = order.quantity.min(resting_order.quantity);

                            trades.push(Trade {
                                buy_order_id: resting_order.id,
                                sell_order_id: order.id,
                                price: best_bid_price,
                                quantity: fill_qty
                            });

                            order.quantity -= fill_qty;
                            resting_order.quantity -= fill_qty;

                            if resting_order.quantity > 0 {
                                bid_queue.push_front(resting_order);
                            }

                            if bid_queue.is_empty() {
                                self.bids.remove(&best_bid_price);
                            }
                        }
                    }
                } else {
                    break;
                }
            }
        }

        // insert any remaining quantity as resting order
        if order.quantity > 0 {
            self.order_index.insert(order.id, (order.side, order.price));
            if order.is_buy() {
                if !self.bids.contains_key(&order.price) {
                    self.bids.insert(order.price, VecDeque::new());
                }
                self.bids.get_mut(&order.price).unwrap().push_back(order);
            } else {
                if !self.asks.contains_key(&order.price) {
                    self.asks.insert(order.price, VecDeque::new());
                }
                self.asks.get_mut(&order.price).unwrap().push_back(order);
            }
        }

        trades
    }


    pub fn cancel_order(&mut self, order_id: u64) -> Option<Order> {
        let (side, price) = self.order_index.get(&order_id)?;

        let book_side = match side{
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks
        };

        let queue = book_side.get_mut(price)?;

        let index = queue.iter().position(|o| o.id == order_id)?;
        let order = queue.remove(index)?;

        if queue.is_empty() {
            book_side.remove(&price);
        }

        self.order_index.remove(&order_id);

        Some(order)
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
    pub quantity: u64 // int quantity
}

impl Order {
    pub fn is_buy(&self) -> bool {
        self.side == Side::Buy
    }
}

// public enum Side
#[derive(Debug, PartialEq, Clone, Copy)]
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

    #[test]
    fn test_exact_match_full_fill() {
        let mut book = OrderBook::new();

        // Resting: sell 100 @ 50
        book.add_order(Order { id: 1, side: Side::Sell, price: 50, quantity: 100 });

        // Incoming: buy 100 @ 50
        let trades = book.add_order(Order { id: 2, side: Side::Buy, price: 50, quantity: 100 });

        // Should have exactly one trade
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].sell_order_id, 1);
        assert_eq!(trades[0].buy_order_id, 2);
        assert_eq!(trades[0].price, 50);
        assert_eq!(trades[0].quantity, 100);

        // Book should be empty
        assert_eq!(book.best_bid(), None);
        assert_eq!(book.best_ask(), None);
    }

    #[test]
    fn test_partial_fill_incoming_smaller() {
        let mut book = OrderBook::new();

        // Resting: sell 100 @ 50
        book.add_order(Order { id: 1, side: Side::Sell, price: 50, quantity: 100 });

        // Incoming: buy 60 @ 50 (smaller than resting)
        let trades = book.add_order(Order { id: 2, side: Side::Buy, price: 50, quantity: 60 });

        // Should have one trade for 60
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 60);

        // Resting ask should still have 40
        assert_eq!(book.best_ask(), Some(50));
    }

    #[test]
    fn test_partial_fill_incoming_larger() {
        let mut book = OrderBook::new();

        // Resting: sell 50 @ 50
        book.add_order(Order { id: 1, side: Side::Sell, price: 50, quantity: 50 });

        // Incoming: buy 100 @ 50 (larger than resting)
        let trades = book.add_order(Order { id: 2, side: Side::Buy, price: 50, quantity: 100 });

        // Should have one trade for 50
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 50);

        // Incoming order should rest with 50 qty
        assert_eq!(book.best_bid(), Some(50));
    }

    #[test]
    fn test_multi_level_sweep_buy() {
        let mut book = OrderBook::new();

        // Resting book:
        // Ask 50 @ 100
        // Ask 50 @ 101
        // Ask 50 @ 102
        book.add_order(Order { id: 1, side: Side::Sell, price: 100, quantity: 50 });
        book.add_order(Order { id: 2, side: Side::Sell, price: 101, quantity: 50 });
        book.add_order(Order { id: 3, side: Side::Sell, price: 102, quantity: 50 });

        // Incoming: buy 100 @ 102 (sweeps first two levels)
        let trades = book.add_order(Order { id: 4, side: Side::Buy, price: 102, quantity: 100 });

        // Should have two trades
        assert_eq!(trades.len(), 2);

        // First trade: 50 @ 100
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[0].quantity, 50);
        assert_eq!(trades[0].sell_order_id, 1);

        // Second trade: 50 @ 101
        assert_eq!(trades[1].price, 101);
        assert_eq!(trades[1].quantity, 50);
        assert_eq!(trades[1].sell_order_id, 2);

        // 50 should rest at 102
        assert_eq!(book.best_ask(), Some(102));
    }

    #[test]
    fn test_multi_level_sweep_sell() {
        let mut book = OrderBook::new();

        // Resting book:
        // Bid 100 @ 100
        // Bid 100 @ 99
        // Bid 100 @ 98
        book.add_order(Order { id: 1, side: Side::Buy, price: 100, quantity: 100 });
        book.add_order(Order { id: 2, side: Side::Buy, price: 99, quantity: 100 });
        book.add_order(Order { id: 3, side: Side::Buy, price: 98, quantity: 100 });

        // Incoming: sell 150 @ 98 (sweeps first two levels)
        let trades = book.add_order(Order { id: 4, side: Side::Sell, price: 98, quantity: 150 });

        // Should have two trades
        assert_eq!(trades.len(), 2);

        // First trade: 100 @ 100 (best bid first)
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[0].quantity, 100);

        // Second trade: 50 @ 99 (partial fill)
        assert_eq!(trades[1].price, 99);
        assert_eq!(trades[1].quantity, 50);

        // 50 should rest at 98
        assert_eq!(book.best_bid(), Some(99));
    }

    #[test]
    fn test_no_cross_buy_too_low() {
        let mut book = OrderBook::new();

        // Resting: sell 100 @ 100
        book.add_order(Order { id: 1, side: Side::Sell, price: 100, quantity: 100 });

        // Incoming: buy 100 @ 95 (price too low, no match)
        let trades = book.add_order(Order { id: 2, side: Side::Buy, price: 95, quantity: 100 });

        // No trades
        assert_eq!(trades.len(), 0);

        // Incoming order should rest
        assert_eq!(book.best_bid(), Some(95));

        // Ask should be unchanged
        assert_eq!(book.best_ask(), Some(100));
    }

    #[test]
    fn test_no_cross_sell_too_high() {
        let mut book = OrderBook::new();

        // Resting: buy 100 @ 50
        book.add_order(Order { id: 1, side: Side::Buy, price: 50, quantity: 100 });

        // Incoming: sell 100 @ 60 (price too high, no match)
        let trades = book.add_order(Order { id: 2, side: Side::Sell, price: 60, quantity: 100 });

        // No trades
        assert_eq!(trades.len(), 0);

        // Incoming order should rest
        assert_eq!(book.best_ask(), Some(60));

        // Bid should be unchanged
        assert_eq!(book.best_bid(), Some(50));
    }

    #[test]
    fn test_fifo_same_price_during_match() {
        let mut book = OrderBook::new();

        // Resting book at same price, different IDs (test FIFO)
        book.add_order(Order { id: 1, side: Side::Sell, price: 50, quantity: 50 });
        book.add_order(Order { id: 2, side: Side::Sell, price: 50, quantity: 50 });

        // Incoming: buy 75 (should fill id:1 first, then partial id:2)
        let trades = book.add_order(Order { id: 3, side: Side::Buy, price: 50, quantity: 75 });

        // Should have two trades
        assert_eq!(trades.len(), 2);

        // First trade: id:1 fully filled
        assert_eq!(trades[0].sell_order_id, 1);
        assert_eq!(trades[0].quantity, 50);

        // Second trade: id:2 partially filled
        assert_eq!(trades[1].sell_order_id, 2);
        assert_eq!(trades[1].quantity, 25);
    }

    // M3: Cancel order tests

    #[test]
    fn test_cancel_existing_order() {
        let mut book = OrderBook::new();
        book.add_order(Order { id: 1, side: Side::Buy, price: 100, quantity: 10 });

        let result = book.cancel_order(1);

        assert!(result.is_some());
        assert_eq!(result.unwrap().id, 1);
        assert_eq!(book.best_bid(), None);
    }

    #[test]
    fn test_cancel_non_existing_order() {
        let mut book = OrderBook::new();
        book.add_order(Order { id: 1, side: Side::Buy, price: 100, quantity: 10 });

        let result = book.cancel_order(999);

        assert!(result.is_none());
        // Original order should still be there
        assert_eq!(book.best_bid(), Some(100));
    }

    #[test]
    fn test_cancel_best_bid() {
        let mut book = OrderBook::new();
        book.add_order(Order { id: 1, side: Side::Buy, price: 100, quantity: 10 });
        book.add_order(Order { id: 2, side: Side::Buy, price: 105, quantity: 10 });
        book.add_order(Order { id: 3, side: Side::Buy, price: 102, quantity: 10 });

        // Cancel the best bid (105)
        let result = book.cancel_order(2);

        assert!(result.is_some());
        assert_eq!(result.unwrap().price, 105);
        // Next best should be 102
        assert_eq!(book.best_bid(), Some(102));
    }

    #[test]
    fn test_cancel_mid_level() {
        let mut book = OrderBook::new();
        // Three orders at the same price (FIFO queue)
        book.add_order(Order { id: 1, side: Side::Sell, price: 50, quantity: 10 });
        book.add_order(Order { id: 2, side: Side::Sell, price: 50, quantity: 20 });
        book.add_order(Order { id: 3, side: Side::Sell, price: 50, quantity: 30 });

        // Cancel the middle one
        let result = book.cancel_order(2);

        assert!(result.is_some());
        assert_eq!(result.unwrap().quantity, 20);

        // Buy 40 - should match id:1 (10) then id:3 (30)
        let trades = book.add_order(Order { id: 4, side: Side::Buy, price: 50, quantity: 40 });

        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].sell_order_id, 1);
        assert_eq!(trades[0].quantity, 10);
        assert_eq!(trades[1].sell_order_id, 3);
        assert_eq!(trades[1].quantity, 30);
    }

    #[test]
    fn test_book_state_consistent_after_cancel() {
        let mut book = OrderBook::new();
        // Set up book with bids and asks
        book.add_order(Order { id: 1, side: Side::Buy, price: 100, quantity: 50 });
        book.add_order(Order { id: 2, side: Side::Sell, price: 110, quantity: 50 });

        // Cancel the bid
        book.cancel_order(1);

        // Add a new bid and verify it works
        book.add_order(Order { id: 3, side: Side::Buy, price: 105, quantity: 25 });
        assert_eq!(book.best_bid(), Some(105));

        // Matching should still work - sell into the new bid
        let trades = book.add_order(Order { id: 4, side: Side::Sell, price: 105, quantity: 25 });

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].buy_order_id, 3);
        assert_eq!(trades[0].quantity, 25);
    }

}
