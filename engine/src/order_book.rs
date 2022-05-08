use std::collections::BTreeMap;
use std::fmt;
use match_base::{OidPrice, OrderKey, Order};
use log::{error, info, warn};

type OrderBookMap = BTreeMap<OidPrice, OrderKey>;

pub struct OrderBook {
    sym_idx:    u32,
    sym_name:   String,
    bids:       OrderBookMap,
    asks:       OrderBookMap,
}

impl PartialEq for OrderBook {
    fn eq(&self, other: &Self) -> bool {
        self.sym_idx == other.sym_idx && self.bids == other.bids &&
            self.asks == other.asks
    }
}

impl OrderBook {
    pub fn new(sym_idx: u32, sym_name: &str) -> OrderBook {
        OrderBook{sym_idx, sym_name: String::from(sym_name),
            bids: BTreeMap::<OidPrice, OrderKey>::new(),
            asks: BTreeMap::<OidPrice, OrderKey>::new(),
        }
    }
    pub fn insert(&mut self, buy: bool, ord: &Order) {
        if buy {
            self.bids.insert(ord.to_OidPrice(), ord.key());
        } else {
            self.asks.insert(ord.to_OidPrice(), ord.key());
        }
    }
    pub fn book(&self, buy: bool) -> &OrderBookMap {
        if buy {
            &self.bids
        } else {
            &self.asks
        }
    }
    pub fn book_mut(&mut self, buy: bool) -> &mut OrderBookMap {
        if buy {
            &mut self.bids
        } else {
            &mut self.asks
        }
    }
    pub fn validate(&self) -> bool {
        // validate bids
        info!("validate bid orderBook for {}", self.sym_name);
        if self.bids.len() > 1 {
            let mut it = self.bids.iter();
            let (_, orkey) = it.next().unwrap();
            let ord = orkey.get().unwrap();
            let mut last = ord.price();
            let mut oid = ord.oid();
            while let Some((_, orkey)) = it.next() {
                let ord = orkey.get().unwrap();
                if ord.is_canceled() {
                    continue
                }
                if ord.is_filled() {
                    error!("order oid({}) is filled, MUST removed", ord.oid());
                    return false
                }
                if last < ord.price() {
                    error!("Bid order book price disorder for oid({})",
                            ord.oid());
                    return false
                }
                if last == ord.price() {
                    if oid >= ord.oid() {
                        error!("Bid order book oid disorder for oid({})",
                                ord.oid());
                        return false
                    }
                    oid = ord.oid();
                    continue
                }
                last = ord.price();
                oid = ord.oid();
            }
        }
        // validate asks
        info!("validate ask orderBook for {}", self.sym_name);
        if self.asks.len() > 1 {
            let mut it = self.asks.iter();
            let (_, orkey) = it.next().unwrap();
            let ord = orkey.get().unwrap();
            let mut last = ord.price();
            let mut oid = ord.oid();
            while let Some((_, orkey)) = it.next() {
                let ord = orkey.get().unwrap();
                if ord.is_canceled() {
                    continue
                }
                if ord.is_filled() {
                    warn!("order oid({}) is filled, MUST removed", ord.oid());
                    return false
                }
                if ord.is_invalid() {
                    error!("order oid({}) is invalid", ord.oid());
                    return false
                }
                if last > ord.price() {
                    error!("Ask order book price disorder for oid({})",
                            ord.oid());
                    return false
                }
                if last == ord.price() {
                    if oid >= ord.oid() {
                        error!("Ask order book oid disorder for oid({})",
                                ord.oid());
                        return false
                    }
                    oid = ord.oid();
                    continue
                }
                last = ord.price();
                oid = ord.oid();
            }
        }
        true
    }
}

impl fmt::Display for OrderBook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "symbol idx({}): name({}) bids(len:{}) asks(len:{})",
                self.sym_idx, self.sym_name, self.bids.len(),
                self.asks.len())
    }
}

#[cfg(test)]
mod tests {
    use simple_logger::SimpleLogger;
    use log::{info, warn, LevelFilter};
    use super::OrderBook;
    use match_base::OrderPool;
    use rand::Rng;
    use measure::Measure;

    #[test]
    #[ignore]
    fn orderbook_test() {
        SimpleLogger::new().init().unwrap();
        log::set_max_level(LevelFilter::Info);
        info!("build orderBook");
        let pool = OrderPool::new();
        let mut orb = OrderBook::new(1, "cu1906");
        let mut rng = rand::thread_rng();
        let mut measure = Measure::start("orderbook bench");
        const N: u32 = 2_000_000;
        for _it in 0 .. N {
            let price = rng.gen::<i32>();
            let mut qty: u32 = rng.gen::<u32>();
            let b_buy: bool = (rng.gen::<u32>() & 1) != 0;
            qty %= 1000;
            qty += 1;
            let ord = pool.new_order(1, b_buy, price, qty).unwrap();
            orb.insert(b_buy, ord);
        }
        measure.stop();
        let ns_ops = measure.as_ns() / (N as u64);
        assert!(ns_ops < 10_000);
        println!("build orderBook cost {} ms, bids: {}, asks: {}",
                 measure.as_ms(), orb.bids.len(), orb.asks.len());
        println!("orderBook insert cost {} ns per Op", ns_ops);
        assert!(orb.validate(), "orderBook disorder");
        warn!("no warn");
    }
}
