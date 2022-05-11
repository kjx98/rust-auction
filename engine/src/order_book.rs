use std::collections::{btree_map, BTreeMap};
use std::fmt;
use match_base::{OidPrice, OrderKey, Order};
use log::{error, info, warn};

type OrderBookMap = BTreeMap<OidPrice, OrderKey>;
type OrderBookIter<'a> = btree_map::Iter<'a, OidPrice, OrderKey>;

pub struct OrderBook {
    sym_idx:    u32,
    sym_name:   String,
    bids:       OrderBookMap,
    asks:       OrderBookMap,
}

pub struct OrderPriceQty<'a> {
    it:     OrderBookIter<'a>,
    last_oid:   u64,
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
    pub fn clear(&mut self) {
        info!("clear symbol({}) orderBook", self.sym_name);
        self.bids.clear();
        self.asks.clear();
    }
    pub fn insert(&mut self, buy: bool, ord: &Order) {
        if buy {
            self.bids.insert(ord.to_OidPrice(), ord.key());
        } else {
            self.asks.insert(ord.to_OidPrice(), ord.key());
        }
    }
    pub fn symbol(&self) -> &str {
        &self.sym_name
    }
    pub fn pv_iter(&self, buy: bool) -> OrderPriceQty {
        if buy {
            OrderPriceQty { it: self.bids.iter(), last_oid: 0}
        } else {
            OrderPriceQty { it: self.asks.iter(), last_oid: 0}
        }
    }
    pub fn len(&self) -> (usize, usize) {
        (self.bids.len(), self.asks.len())
    }
    pub fn retain(&mut self, buy: bool, okey: OrderKey) {
        let ord = okey.get().unwrap();
        let key = ord.to_OidPrice();
        if buy {
            self.bids = self.bids.split_off(&key);
            if ord.is_filled() {
                self.bids.remove(&key);
            }
        } else {
            self.asks = self.asks.split_off(&key);
            if ord.is_filled() {
                self.asks.remove(&key);
            }
        }
    }
    //#[allow(dead_code)]
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
        #[cfg(test)]
        info!("validate bid orderBook for {}", self.sym_name);
        if self.bids.len() > 1 {
            let mut it = self.bids.iter();
            let (_, orkey) = it.next().unwrap();
            let ord = orkey.get().unwrap();
            let mut last = ord.price();
            let mut oid = ord.oid();
            while let Some((_, orkey)) = it.next() {
                if let Some(ord) = orkey.get() {
                    if ord.is_canceled() { continue }
                    if ord.is_filled() {
                        error!("{} order oid({}) is filled, MUST removed",
                                self.sym_name, ord.oid());
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
                } else {
                    error!("bids orkey {} not found", orkey.key());
                    return false
                }
            }
        }
        // validate asks
        #[cfg(test)]
        info!("validate ask orderBook for {}", self.sym_name);
        if self.asks.len() < 2 { return true }
        let mut it = self.asks.iter();
        let (_, orkey) = it.next().unwrap();
        let ord = orkey.get().unwrap();
        let mut last = ord.price();
        let mut oid = ord.oid();
        while let Some((_, orkey)) = it.next() {
            if let Some(ord) = orkey.get() {
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
            } else {
                error!("asks orkey {} not found", orkey.key());
                return false
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

impl OrderPriceQty<'_> {
    pub fn next(&mut self) -> Option<(i32, u32)> {
        let prc: i32;
        let mut qty: u32;
        if self.last_oid == 0 {
            if let Some((_, orkey)) = self.it.next() {
                let ord = orkey.get().unwrap();
                prc = ord.price();
                qty = ord.remain_qty();
            } else {
                return None
            }
        } else {
            let orkey = OrderKey::from(self.last_oid);
            let ord = orkey.get().unwrap();
            prc = ord.price();
            qty = ord.remain_qty();
        }
        while let Some((_, orkey)) = self.it.next() {
            let ord = orkey.get().unwrap();
            if prc == ord.price() {
                qty += ord.remain_qty();
                continue
            }
            self.last_oid = ord.oid();
            return Some((prc, qty))
        }
        self.last_oid = 0;
        Some((prc, qty))
    }
}

#[cfg(test)]
mod tests {
    use simple_logger::SimpleLogger;
    use log::{info, warn, error, LevelFilter};
    use super::OrderBook;
    use match_base::OrderPool;
    use rand::Rng;
    use measure::Measure;

    #[test]
    fn test_orderbook() {
        if let Err(s) = SimpleLogger::new().init() {
            warn!("SimpleLogger init: {}", s);
        }
        log::set_max_level(LevelFilter::Info);
        info!("build orderBook");
        let pool = OrderPool::new();
        let mut orb = OrderBook::new(1, "cu1906");
        let mut rng = rand::thread_rng();
        let mut measure = Measure::start("orderbook bench");
        const N: u32 = 2_000;
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
        assert_eq!(orb.len(), (orb.bids.len(), orb.asks.len()));
        println!("build orderBook cost {} us, bids: {}, asks: {}",
                 measure.as_us(), orb.bids.len(), orb.asks.len());
        println!("orderBook insert cost {} ns per Op", ns_ops);
        assert!(orb.validate(), "orderBook disorder");
    }

    #[test]
    fn test_orderbook_pv() {
        if let Err(s) = SimpleLogger::new().init() {
            warn!("SimpleLogger init: {}", s);
        }
        log::set_max_level(LevelFilter::Info);
        info!("build orderBook");
        let pool = OrderPool::new();
        let mut orb = OrderBook::new(1, "cu1906");
        let b_buy = true;
        let ord = pool.new_order(1, b_buy, 30000, 10).unwrap();
        orb.insert(b_buy, ord);
        let ord = pool.new_order(1, b_buy, 30000, 15).unwrap();
        orb.insert(b_buy, ord);
        let ord = pool.new_order(1, b_buy, 31000, 18).unwrap();
        orb.insert(b_buy, ord);
        let mut pv_it = orb.pv_iter(b_buy);
        let opv = pv_it.next();
        assert!(opv != None);
        let (prc, vol) = opv.unwrap();
        assert_eq!(prc, 31000);
        assert_eq!(vol, 18);
        let opv = pv_it.next();
        assert!(opv != None);
        let (prc, vol) = opv.unwrap();
        assert_eq!(prc, 30000);
        assert_eq!(vol, 25);
        assert!(pv_it.next() == None);
    }

    #[test]
    #[ignore]
    fn bench_orderbook() {
        if let Err(s) = SimpleLogger::new().init() {
            warn!("SimpleLogger init: {}", s);
        }
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
            if let Some(ord) = pool.new_order(1, b_buy, price, qty) {
                orb.insert(b_buy, ord);
            } else {
                error!("OrderPool new_order failed");
                break
            }
        }
        measure.stop();
        let ns_ops = measure.as_ns() / (N as u64);
        assert!(ns_ops < 10_000);
        println!("build orderBook cost {} us, bids: {}, asks: {}",
                 measure.as_us(), orb.bids.len(), orb.asks.len());
        println!("orderBook insert cost {} ns per Op", ns_ops);
        let mut measure = Measure::start("orderbook bench");
        let valid = orb.validate();
        measure.stop();
        let ns_ops = measure.as_ns() / (N as u64);
        assert!(ns_ops < 10_000);
        println!("validate orderBook cost {} us, bids: {}, asks: {}",
                 measure.as_us(), orb.bids.len(), orb.asks.len());
        assert!(valid, "orderBook disorder");
    }
}
