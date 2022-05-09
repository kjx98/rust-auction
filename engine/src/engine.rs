use std::collections::HashMap;
use std::vec::Vec;
use std::default::Default;
use match_base::{Order, OrderPool, Symbols};
use crate::{State, Deals};
use crate::order_book::OrderBook;
use log::{error, info, warn};

pub struct MatchEngine {
    state:  State,
    symbols: Symbols,
    pool:   OrderPool,
    book:   HashMap<u32, OrderBook>,
    deals:  Deals,
}

#[allow(dead_code)]
#[inline]
fn may_match(buy: bool, book_price: i32, take_price: i32) -> bool {
    if buy {
        book_price >= take_price
    } else {
        book_price <= take_price
    }
}

#[allow(dead_code)]
#[inline]
fn is_price_better(buy: bool, prc1: i32, prc2: i32) -> bool {
    if buy {
        prc1 > prc2
    } else {
        prc1 < prc2
    }
}

#[allow(dead_code)]
#[inline]
fn get_mid_price(hi: i32, lo: i32, clast: i32) -> i32 {
    if hi == lo || clast > hi {
        hi
    } else if clast < lo {
        lo
    } else {
        clast
    }
}


#[allow(dead_code)]
#[inline]
fn get_match_qty(orb: &OrderBook, buy: bool, prc: i32, qty: u32) -> u32 {
    let ob = orb.book(buy);
    if ob.len() == 0 {
        return 0
    }
    let mut it = ob.iter();
    let mut fill_qty = 0;
    while let Some((_, okey)) = it.next() {
        if let Some(ord) = okey.get() {
            if !may_match(buy, ord.price(), prc) {
                return fill_qty
            }
            fill_qty += ord.remain_qty();
            if qty != 0 && fill_qty >= qty {
                break
            }
        }
    }
    fill_qty
}

impl MatchEngine {
    pub fn new() -> MatchEngine {
        let pool = OrderPool::new();
        let mut me = MatchEngine { pool, state: Default::default(),
                    symbols: Symbols::new(), deals: Deals::new(),
                    book: HashMap::<u32, OrderBook>::new() };
        me.symbols.add_symbol("cu1906");
        me.symbols.add_symbol("cu1908");
        me.symbols.add_symbol("cu1909");
        me.symbols.add_symbol("cu1912");
        me
    }
    pub fn change_state(&mut self, new_state: State) -> bool {
        let rev = self.state.review(&new_state);
        if rev {
            // do somethine
            match new_state {
                State::StateIdle => {
                    let pool = OrderPool::new();
                    pool.init();        // clear orders
                    self.deals.clear();
                },
                State::StateCallAuction => {
                    // do call auction
                    // uncross
                },
                _ => { },
            }
            self.state = new_state;
        }
        rev
    }
    // init symbols/orders/deals
    pub fn init_market(&mut self) -> bool {
        self.change_state(State::StateIdle)
    }
    // goto preAuction
    pub fn start_market(&mut self) -> bool {
        self.change_state(State::StatePreAuction)
    }
    // uncross
    pub fn call_auction(&mut self) -> bool {
        self.change_state(State::StateCallAuction)
    }
    // start trading
    pub fn start_trading(&mut self) -> bool {
        self.change_state(State::StateTrading)
    }
    // pause trading
    pub fn pause_trading(&mut self) -> bool {
        self.change_state(State::StatePause)
    }
    // stop trading
    pub fn stop_trading(&mut self) -> bool {
        self.change_state(State::StateStop)
    }
    // end market
    pub fn end_market(&mut self) -> bool {
        self.change_state(State::StateEnd)
    }
    pub fn symbol_idx(&self, name: &str) -> Option<u32> {
        self.symbols.get_idx(name)
    }
    pub fn send_order(&mut self, sym: u32, buy: bool, price: i32, qty: u32)
    -> Option<u64> {
        if !self.state.can_book() {
            return None
        }
        let new_or = self.pool.new_order(sym, buy, price, qty);
        if new_or == None {
            return None
        }
        let ord = new_or.unwrap();
        // try match or insert to orderBook
        if self.state.is_tc() {
            // try_match
            if self.try_match(ord) {
                return Some(ord.oid())
            }
        }
        if let Some(or_book) = self.book.get_mut(&sym) {
            or_book.insert(buy, ord);
        } else {
            let mut or_book = OrderBook::new(sym, "symbol");
            or_book.insert(buy, ord);
            self.book.insert(sym, or_book);
        }
        Some(ord.oid())
    }
    #[inline]
    pub fn try_match(&mut self, ord: &mut Order) -> bool {
        // filled
        let or_book = self.book.get_mut(& ord.symbol());
        if or_book == None {
            return false
        }
        let or_book = or_book.unwrap().book_mut(!ord.is_buy());
        if or_book.len() == 0 {
            return false
        }
        true
    }
    #[inline]
    pub fn set_fill(&mut self, ord: &mut Order, vol: u32, price: i32) {
        ord.fill(vol, price);
        self.deals.push_deal(ord.oid(), price, vol);
        // should pushDeal to mdCache as well
    }
}

#[cfg(test)]
mod tests {
    use super::{may_match, is_price_better, get_mid_price};
    use simple_logger::SimpleLogger;
    use log::{info, warn, LevelFilter};

    #[test]
    fn test_inlines() {
        if let Err(s) = SimpleLogger::new().init() {
            warn!("SimpleLogger init: {}", s);
        }
        log::set_max_level(LevelFilter::Info);
        info!("test may_match");
        assert!(may_match(true, 34000,34000));
        assert!(may_match(true, 34000,33000));
        assert!(may_match(false, 34000, 34000));
        assert!(may_match(false, 34000, 34500));
        info!("test is_price_better");
        assert!(!is_price_better(true, 34000,34000));
        assert!(is_price_better(true, 34000,33000));
        assert!(!is_price_better(false, 34000, 34000));
        assert!(is_price_better(false, 34000, 34500));
        info!("test get_mid_price");
        assert_eq!(get_mid_price(32000, 30000, 31000), 31000);
        assert_eq!(get_mid_price(32000, 30000, 29000), 30000);
        assert_eq!(get_mid_price(32000, 30000, 33000), 32000);
        assert_eq!(get_mid_price(32000, 30000, 32000), 32000);
    }
}
