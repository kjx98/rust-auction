use std::collections::HashMap;
use std::default::Default;
use match_base::{OrderPool, Symbols};
use crate::State;
use crate::order_book::OrderBook;

pub struct MatchEngine {
    state:  State,
    symbols: Symbols,
    pool:   OrderPool,
    book:   HashMap<u32, OrderBook>,
}

impl MatchEngine {
    pub fn new() -> MatchEngine {
        let pool = OrderPool::new();
        let mut me = MatchEngine { pool, state: Default::default(),
                    symbols: Symbols::new(),
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
        if let Some(ord) = self.pool.new_order(sym, buy, price, qty) {
            // try match or insert to orderBook
            if let Some(or_book) = self.book.get_mut(&sym) {
                or_book.insert(buy, ord);
            } else {
                let mut or_book = OrderBook::new(sym, "symbol");
                or_book.insert(buy, ord);
                self.book.insert(sym, or_book);
            }
            Some(ord.oid())
        } else {
            None
        }
    }
}
