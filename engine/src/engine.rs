use std::collections::HashMap;
//use std::default::Default;
use match_base::{Order, OrderPool, DealPool, Symbols};
use crate::{State};
use crate::order_book::OrderBook;
use measure::Measure;
use log::{error, info, warn};

pub struct MatchEngine {
    state:  State,
    symbols: Symbols,
    pool:   OrderPool,
    book:   HashMap<u32, OrderBook>,
    deals:  DealPool,
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
            if ord.is_canceled() { continue }
            if ord.is_invalid() {
                error!("order({}) is invalid", ord.oid());
                continue
            }
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
                    symbols: Symbols::new(), deals: DealPool::new(),
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
            info!("do change state {}", new_state);
            match new_state {
                State::StateIdle => {
                    let pool = OrderPool::new();
                    pool.init();        // clear orders
                    self.deals.clear();
                    // clear orderBooks
                    let mut it = self.book.iter_mut();
                    while let Some((_, ob)) = it.next() {
                        ob.clear();
                    }
                },
                State::StateCallAuction => {
                    // do call auction
                    // uncross
                },
                _ => { },
            }
            self.state = new_state;
        } else {
            warn!("can't change to: {}", new_state);
        }
        rev
    }
    // init symbols/orders/deals
    pub fn init_market(&mut self) -> bool {
        self.change_state(State::StateIdle)
    }
    // open market
    pub fn begin_market(&mut self) -> bool {
        self.change_state(State::StateStart)
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
    pub fn book(&self, sym: u32) -> Option<&OrderBook> {
        self.book.get(&sym)
    }
    #[inline]
    pub fn set_fill(&mut self, ord: &mut Order, vol: u32, price: i32) {
        ord.fill(vol, price);
        self.deals.push_deal(ord.oid(), price, vol);
        // should pushDeal to mdCache as well
    }
    // return  (last, max_qty, remain_qty)
    fn uncross(&mut self, sym: u32, pclose: i32) -> Option<(i32,u32,u32)> {
        let orb = self.book.get(&sym);
        if orb == None { return None }
        let orb = orb.unwrap();
        let mut bit = orb.pv_iter(true);
        let mut ait = orb.pv_iter(false);
        let bp = bit.next();
        let ap = ait.next();
        if bp == None || ap == None { return None }
        let (mut bp, mut bvol) = bp.unwrap();
        let (mut ap, mut avol) = ap.unwrap();
        if bp < ap { return None }
        let (best_bid, best_ask) = (bp, ap);
        let mut max_qty: u32 = 0;
        let mut remain_qty: u32 = 0;
        let mut last: i32 = pclose;
        let mut b_end = false;
        let mut a_end = false;
        //info!("sym({}) MatchCross BBS: {}/{}", sym, bp, ap);
        while !b_end && !a_end && bp >= ap
        {
            if bvol > avol {
                max_qty += avol;
                bvol -= avol;
                remain_qty = bvol;
                last = ap;
                if let Some((p,v)) = ait.next() {
                    ap = p;
                    avol = v;
                } else {
                    break
                }
            } else if bvol < avol {
                max_qty += bvol;
                avol -= bvol;
                remain_qty = avol;
                last = bp;
                if let Some((p,v)) = bit.next() {
                    bp = p;
                    bvol = v;
                } else {
                    break
                }
            } else {
                max_qty += bvol;
                remain_qty = 0;
                last = bp;
                if bp == ap {
                    break
                }
                let oap = ap;
                let obp = bp;
                if let Some((p,v)) = bit.next() {
                    bp = p;
                    bvol = v;
                    b_end = bp < best_ask;
                } else {
                    b_end = true;
                }
                if let Some((p,v)) = ait.next() {
                    ap = p;
                    avol = v;
                    a_end = ap > best_bid;
                } else {
                    a_end = true;
                }
                if b_end && a_end {
                    if oap > pclose {
                        last = oap;
                    } else if obp < pclose {
                        last = obp;
                    } else {
                        last = pclose;
                    }
                    break
                }
                if b_end { last = oap }
                if a_end { last = obp }
            }
            /*
            info!("update MatchCross price: {} {}/{} volume: {}(left: {})",
                    last, bp, ap, max_qty, remain_qty);
            */
        }
        /*
        info!("symbol({}) MatchCross end, bp/ap: {}/{} volume: {}(left: {})",
                sym, bp, ap, max_qty, remain_qty);
        */
        Some((last, max_qty, remain_qty))
    }
    pub fn match_cross(&mut self, sym: u32, pclose: i32)
    -> Option<(i32,u32,u32)> {
        // only uncross on PreAuction
        if self.state != State::StatePreAuction {
            return None
        } else {
            self.uncross(sym, pclose)
        }
    }
    pub fn load_orders(&mut self, sym: u32, filen: &str) -> bool {
        use std::fs::File;
        use std::io::Read;
        let mut buff = Vec::<u8>::new();
        let mut measure = Measure::start("load_orders bench");
        if let Ok(mut rdr) = File::open(filen) {
            if filen.ends_with(".zst") {
                buff = zstd::stream::decode_all(rdr).unwrap();
            } else {
                rdr.read_to_end(&mut buff).unwrap();
            }
        } else {
            warn!("can't open {}", filen);
            return false
        }
        if let Ok(sbuf) = std::str::from_utf8(buff.as_slice()) {
            let cnt = self.build_orders(sym, sbuf);
            measure.stop();
            info!("load {} orders from {} cost {}ms", cnt, filen,
                  measure.as_ms());
        } else {
            return false
        }
        true
    }
    pub fn build_orders(&mut self, sym: u32, orders: &str) -> u32 {
        let mut it = orders.lines();
        let mut order_cnt: u32 = 0;
        while let Some(aline) = it.next() {
            //info!("send order: {}", aline);
            let v: Vec<&str> = aline.split(',').collect();
            if v.len() < 4 { continue }
            if let Ok(prc) = v[1].trim().parse::<i32>() {
                if let Ok(qty) = v[2].trim().parse::<u32>() {
                    let buy: bool = if let Ok(bb) = v[3].trim().parse::<i32>() {
                                        bb != 0
                                    } else { false };
                    if self.send_order(sym, buy, prc, qty) == None {
                        warn!("send_order failed");
                        return 0
                    }
                    order_cnt += 1;
                } else { continue }
            } else { continue }
        }
        //info!("build {} orders for symbol({})", order_cnt, sym);
        order_cnt
    }
}

#[cfg(test)]
mod tests {
    use super::{may_match, is_price_better, get_mid_price, MatchEngine};
    use simple_logger::SimpleLogger;
    use crate::state::State;
    use match_base::Deal;
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


    #[test]
    fn test_cross_one() {
        if let Err(s) = SimpleLogger::new().init() {
            warn!("SimpleLogger init: {}", s);
        }
        log::set_max_level(LevelFilter::Info);
        let orders1 = "1, 42000, 10, 1\n\
2,43000,20,1\n\
3,41000,30,1\n\
4,44000,50,1\n\
5,45000,10,0\n\
6,48000,20,0\n\
7,46000,30,0\n\
8,43500,45,0\n\
9,43900,25,1\n\
10,43200,10,0\n\
11,43800,15,1\n\
12,43200,20,0\n";
        let mut me = MatchEngine::new();
        assert!(me.state.eq(&State::StateIdle));
        assert!(me.begin_market());
        assert!(me.start_market());
        assert_eq!(me.build_orders(1, orders1), 12);
        let orb = me.book(1);
        assert!(orb != None);
        let mc_ret = me.match_cross(1, 40000);
        assert!(mc_ret == Some((43900, 75, 0)));
        let mc_ret = me.match_cross(1, 50000);
        assert!(mc_ret == Some((43900, 75, 0)));
    }

    // clear orders cause order_book test fails since static orderPool
    #[test]
    #[ignore]
    fn test_cross() {
        if let Err(s) = SimpleLogger::new().init() {
            warn!("SimpleLogger init: {}", s);
        }
        log::set_max_level(LevelFilter::Info);
        let deals1 = vec![Deal::new(1, 4, 43500, 45),
                        Deal::new(2, 8, 43500, 45),
                        Deal::new(3, 4, 43200, 5),
                        Deal::new(4, 10, 43200, 5),
                        Deal::new(5, 9, 43200, 5),
                        Deal::new(6, 10, 43200, 5),
                        Deal::new(7, 9, 43200, 20),
                        Deal::new(8, 12, 43200, 20)];
        let orders1 = "1, 42000, 10, 1\n\
2,43000,20,1\n\
3,41000,30,1\n\
4,44000,50,1\n\
5,45000,10,0\n\
6,48000,20,0\n\
7,46000,30,0\n\
8,43500,45,0\n\
9,43900,25,1\n\
10,43200,10,0\n\
11,43800,15,1\n\
12,43200,20,0\n";
        let orders2 = "1, 43000, 20, 1\n\
2, 44000, 50, 1\n\
3, 45000, 10, 0\n\
4, 43500, 45, 0\n\
5, 43200, 10, 0\n\
6, 43900, 25, 1\n\
7, 43200, 20, 0\n";
        let orders3 = "1, 43000, 20, 1\n\
2, 44000, 50, 1\n\
3, 43900, 15, 1\n\
4, 45000, 10, 0\n\
5, 43500, 45, 0\n\
6, 43200, 10, 0\n\
7, 43200, 20, 0\n";
        let mut me = MatchEngine::new();
        assert!(me.state.eq(&State::StateIdle));
        assert!(me.begin_market());
        assert!(me.start_market());
        assert_eq!(me.build_orders(1, orders1), 12);
        let orb = me.book(1);
        assert!(orb != None);
        let mc_ret = me.match_cross(1, 40000);
        assert!(mc_ret == Some((43900, 75, 0)));
        let mc_ret = me.match_cross(1, 50000);
        assert!(mc_ret == Some((43900, 75, 0)));
        assert!(me.stop_trading());
        assert!(me.init_market());
        //let mut me = MatchEngine::new();
        //assert!(me.state.eq(&State::StateIdle));
        assert!(me.begin_market());
        assert!(me.start_market());
        assert!(me.build_orders(1, orders2) == 7);
        let orb = me.book(1);
        assert!(orb != None);
        let mc_ret = me.match_cross(1, 40000);
        assert!(mc_ret == Some((43500, 75, 0)));
        let mc_ret = me.match_cross(1, 50000);
        assert!(mc_ret == Some((43900, 75, 0)));
        assert!(me.stop_trading());
        assert!(me.init_market());
        assert!(me.begin_market());
        assert!(me.start_market());
        assert_eq!(me.build_orders(1, orders3), 7);
        let orb = me.book(1);
        assert!(orb != None);
        let mc_ret = me.match_cross(1, 40000);
        assert!(mc_ret == Some((43900, 65, 10)));
        let mc_ret = me.match_cross(1, 50000);
        assert!(mc_ret == Some((43900, 65, 10)));
    }

    #[test]
    #[ignore]
    fn bench_cross() {
        use measure::Measure;
        if let Err(s) = SimpleLogger::new().init() {
            warn!("SimpleLogger init: {}", s);
        }
        log::set_max_level(LevelFilter::Info);
        let mut me = MatchEngine::new();
        assert!(me.state.eq(&State::StateIdle));
        assert!(me.begin_market());
        assert!(me.start_market());
        assert!(me.load_orders(1, "/tmp/long.txt.zst"));
        assert!(me.load_orders(1, "/tmp/short.txt.zst"));
        let mut measure = Measure::start("cross bench");
        let mc_ret = me.match_cross(1, 50000);
        measure.stop();
        let (last, qty, rem_qty) = mc_ret.unwrap();
        info!("MatchCross last: {}, volume: {}, remain: {}",
              last, qty, rem_qty);
        info!("MatchCross cost {}us", measure.as_us());
    }
}
