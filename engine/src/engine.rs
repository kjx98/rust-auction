use std::collections::HashMap;
use std::vec::Vec;
use std::default::Default;
use match_base::{Order, OrderPool, Symbols};
use crate::{State, Deal};
use crate::order_book::OrderBook;

pub struct MatchEngine {
    state:  State,
    symbols: Symbols,
    pool:   OrderPool,
    book:   HashMap<u32, OrderBook>,
    deals:  Vec<Deal>,
}

#[inline]
fn may_match(buy: bool, book_price: i32, take_price: i32) -> bool {
    if buy {
        book_price >= take_price
    } else {
        book_price <= take_price
    }
}

#[inline]
fn is_price_better(buy: bool, prc1: i32, prc2: i32) -> bool {
    if buy {
        prc1 > prc2
    } else {
        prc1 < prc2
    }
}

impl MatchEngine {
    pub fn new() -> MatchEngine {
        let pool = OrderPool::new();
        let mut me = MatchEngine { pool, state: Default::default(),
                    symbols: Symbols::new(), deals: Vec::<Deal>::new(),
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
            self.state = new_state;
        }
        rev
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
        let deal_no = self.deals.len() + 1;
        self.deals.push(Deal::new(deal_no as u64, ord.oid(), price, vol));
        // should pushDeal to mdCache as well
    }
}

#[cfg(test)]
mod tests {
    use super::{may_match, is_price_better};

    #[test]
    fn test_inlines() {
        assert!(may_match(true, 34000,34000));
        assert!(may_match(true, 34000,33000));
        assert!(may_match(false, 34000, 34000));
        assert!(may_match(false, 34000, 34500));
        assert!(!is_price_better(true, 34000,34000));
        assert!(is_price_better(true, 34000,33000));
        assert!(!is_price_better(false, 34000, 34000));
        assert!(is_price_better(false, 34000, 34500));
    }
}
