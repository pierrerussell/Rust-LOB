# Rust LOB

A limit order book engine written in Rust with price-time priority matching.

## Design Decisions

### Why `BTreeMap<Price, VecDeque<Order>>` for bids/asks?

The order book needs two things: sorted price levels and FIFO ordering within each level.

**Price levels (`BTreeMap`):**
- `BTreeMap` keeps prices sorted automatically — O(log n) insert/lookup
- `best_bid()` and `best_ask()` are O(1) via `last_key_value()` / `first_key_value()`
- Alternative: `HashMap` would be O(1) lookup but requires manual sorting to find best price

**Orders within a level (`VecDeque`):**
- Price-time priority requires FIFO — oldest order at a price fills first
- `VecDeque` gives O(1) push_back (new orders) and pop_front (fills)
- Alternative: `Vec` would be O(n) for front removal

### Why `HashMap<OrderId, (Side, Price)>` for order index?

Cancellation needs to find an order by ID quickly. Without an index, we'd have to:
1. Search all price levels in bids
2. Then search all price levels in asks
3. Then linear scan the queue at each level

The index gives O(1) lookup to find exactly which side and price level contains the order. Tradeoff is memory overhead and keeping the index in sync during adds/cancels/fills.

### Why `u64` for prices instead of floats?

- No floating point precision issues (0.1 + 0.2 ≠ 0.3)
- Integer comparison is exact and fast
- Real exchanges use integer "ticks" — price = tick × tick_size
- Tradeoff: caller must handle tick conversion

### Why `Arc<Mutex<OrderBook>>` for thread safety?

**Why thread safety at all:**
Future agent simulations will have multiple threads submitting orders concurrently. The order book is shared mutable state.

**Why `Mutex` over `RwLock`:**
- Order books are write-heavy — most operations mutate state
- `RwLock` shines when reads vastly outnumber writes
- `Mutex` is simpler with less overhead for write-heavy workloads
- Can revisit if profiling shows read contention on `best_bid()`/`best_ask()`

**Why coarse-grained (one lock for entire book) over fine-grained:**
- The matching loop in `add_order()` reads and writes `bids`, `asks`, and `order_index` atomically
- Fine-grained locks (per-collection) would risk inconsistent state mid-match
- Fine-grained also introduces deadlock risk if lock ordering isn't strict
- Tradeoff: lower throughput under high concurrency, but correctness is guaranteed
- For 10-100 simulated agents, coarse-grained is sufficient

**Why `Arc`:**
- Multiple threads need ownership of the same order book
- `Arc` provides shared ownership with reference counting
- `Clone` on the wrapper clones the `Arc` (cheap), not the data

### Why return `Vec<Trade>` from `add_order()`?

- Trades are events, not state — they represent what happened
- Returning them lets the caller decide what to do (log, update P&L, broadcast)
- Alternative: store trades inside OrderBook, but that couples storage decisions to the engine

## Project Structure

```
src/
└── domain/
    ├── order.rs        # OrderBook, Order, Side
    ├── trade.rs        # Trade event
    └── thread_safe.rs  # ThreadSafeOrderBook (Arc<Mutex<T>> wrapper)
```

## Running Tests

```bash
cargo test
```

24 tests covering: insertion, best bid/ask, FIFO ordering, full/partial fills, multi-level sweeps, cancellation, and concurrent access.

## License

MIT
