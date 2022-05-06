use std::collections::BTreeMap;
use std::fmt;
use match_base::{OidPrice, OrderKey, Order};

pub struct OrderBook {
    sym_idx:    u32,
    sym_name:   String,
    bids:       BTreeMap<OidPrice, OrderKey>,
    asks:       BTreeMap<OidPrice, OrderKey>,
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
}

impl fmt::Display for OrderBook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "symbol idx({}): name({}) bids(len:{}) asks(len:{})",
                self.sym_idx, self.sym_name, self.bids.len(),
                self.asks.len())
    }
}
