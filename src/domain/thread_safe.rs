use std::sync::{Arc, Mutex, MutexGuard};

use crate::domain::order::{Order, OrderBook};
use crate::domain::trade::Trade;

#[derive(Debug)]
pub struct ThreadSafeOrderBook {
    inner: Arc<Mutex<OrderBook>>,
}

impl ThreadSafeOrderBook {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(OrderBook::new())),
        }
    }

    pub fn from_order_book(book: OrderBook) -> Self {
        Self {
            inner: Arc::new(Mutex::new(book)),
        }
    }


    pub fn add_order(&self, order: Order) -> Vec<Trade> {
        self.inner.lock().unwrap().add_order(order)
    }

    pub fn cancel_order(&self, order_id: u64) -> Option<Order> {
        self.inner.lock().unwrap().cancel_order(order_id)
    }

    pub fn best_bid(&self) -> Option<u64> {
        self.inner.lock().unwrap().best_bid()
    }

    pub fn best_ask(&self) -> Option<u64> {
        self.inner.lock().unwrap().best_ask()
    }

    pub fn spread(&self) -> Option<u64> {
        let guard = self.inner.lock().unwrap();
        match (guard.best_bid(), guard.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask.saturating_sub(bid)),
            _ => None,
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, OrderBook> {
        self.inner.lock().unwrap()
    }
}

impl Clone for ThreadSafeOrderBook {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Default for ThreadSafeOrderBook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::order::Side;
    use std::thread;

    #[test]
    fn test_basic_thread_safe_operations() {
        let book = ThreadSafeOrderBook::new();

        book.add_order(Order {
            id: 1,
            side: Side::Buy,
            price: 100,
            quantity: 10,
        });

        assert_eq!(book.best_bid(), Some(100));
        assert_eq!(book.best_ask(), None);
    }

    #[test]
    fn test_clone_shares_state() {
        let book1 = ThreadSafeOrderBook::new();
        let book2 = book1.clone();

        book1.add_order(Order {
            id: 1,
            side: Side::Buy,
            price: 100,
            quantity: 10,
        });

        // book2 should see the order added via book1
        assert_eq!(book2.best_bid(), Some(100));
    }

    #[test]
    fn test_concurrent_order_submission() {
        let book = ThreadSafeOrderBook::new();
        let mut handles = vec![];

        // Spawn 10 threads, each adding a buy order
        for i in 0..10 {
            let book_clone = book.clone();
            let handle = thread::spawn(move || {
                book_clone.add_order(Order {
                    id: i,
                    side: Side::Buy,
                    price: 100 + i,
                    quantity: 10,
                });
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Best bid should be the highest price (109)
        assert_eq!(book.best_bid(), Some(109));
    }

    #[test]
    fn test_concurrent_matching() {
        let book = ThreadSafeOrderBook::new();

        // Add resting sell orders
        for i in 0..5 {
            book.add_order(Order {
                id: i,
                side: Side::Sell,
                price: 100,
                quantity: 10,
            });
        }

        let mut handles = vec![];
        let total_trades = Arc::new(Mutex::new(Vec::new()));

        // Spawn 5 threads, each submitting a buy order that should match
        for i in 5..10 {
            let book_clone = book.clone();
            let trades_clone = Arc::clone(&total_trades);
            let handle = thread::spawn(move || {
                let trades = book_clone.add_order(Order {
                    id: i,
                    side: Side::Buy,
                    price: 100,
                    quantity: 10,
                });
                trades_clone.lock().unwrap().extend(trades);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // All 5 sell orders should have been matched
        let trades = total_trades.lock().unwrap();
        assert_eq!(trades.len(), 5);

        // Book should be empty after all matches
        assert_eq!(book.best_bid(), None);
        assert_eq!(book.best_ask(), None);
    }

    #[test]
    fn test_concurrent_add_and_cancel() {
        let book = ThreadSafeOrderBook::new();

        // Add an order
        book.add_order(Order {
            id: 1,
            side: Side::Buy,
            price: 100,
            quantity: 10,
        });

        let book1 = book.clone();
        let book2 = book.clone();

        // One thread tries to cancel, another adds a new order
        let handle1 = thread::spawn(move || {
            book1.cancel_order(1)
        });

        let handle2 = thread::spawn(move || {
            book2.add_order(Order {
                id: 2,
                side: Side::Buy,
                price: 105,
                quantity: 5,
            })
        });

        let cancelled = handle1.join().unwrap();
        handle2.join().unwrap();

        // Order 1 should have been cancelled
        assert!(cancelled.is_some());
        assert_eq!(cancelled.unwrap().id, 1);

        // Order 2 should now be the best bid
        assert_eq!(book.best_bid(), Some(105));
    }

    #[test]
    fn test_spread_calculation() {
        let book = ThreadSafeOrderBook::new();

        // No spread when book is empty
        assert_eq!(book.spread(), None);

        book.add_order(Order {
            id: 1,
            side: Side::Buy,
            price: 100,
            quantity: 10,
        });

        // No spread with only bids
        assert_eq!(book.spread(), None);

        book.add_order(Order {
            id: 2,
            side: Side::Sell,
            price: 105,
            quantity: 10,
        });

        // Spread should be 5
        assert_eq!(book.spread(), Some(5));
    }
}
