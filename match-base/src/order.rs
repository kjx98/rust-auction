use std::cmp::Ordering;
use std::fmt;
use std::default::Default;
use std::sync::{Once, atomic};

type Oid = u64;

//#[repr(align(16))]
#[derive(Eq, Clone, Default)]
pub struct Order {
    id:     Oid,
    price:  i32,
    sym_idx: u32,
    qty:    u32,
    filled: u32,
    buy:   bool,
    canceled:  bool,
    price_filled:i32,
}

#[derive(PartialEq)]
pub struct OrderKey(u32);

// for use for orderBook order, bid high as best while ask low as best
// bid(buy) order, negative price as order price
#[derive(Eq, Clone)]
pub struct OidPrice {
    id:     u32,
    price:  i32,
}

pub struct OrderPool ();

const MAX_ORDERS: u32 = 60_000_000;
static INIT: Once = Once::new();
static mut ORDER_POOL: Vec<Order> = Vec::new();
static mut POOL_LOCK: atomic::AtomicBool = atomic::AtomicBool::new(false);
//static mut ORDER_POOL: &mut [Order] = &mut [];
//unsafe { ORDER_POOL = std::slice::from_raw_parts_mut( data: *mut Order, len: usize) }

// init orders db
fn init_orders() {
    unsafe {
        ORDER_POOL = Vec::<Order>::with_capacity(2048);
    }
}

pub fn clear_orders() {
    INIT.call_once(|| {
        init_orders();
    });
    unsafe {
        while POOL_LOCK.swap(true, atomic::Ordering::Acquire) {
            std::thread::yield_now();
        }
        ORDER_POOL.clear();
        POOL_LOCK.store(false, atomic::Ordering::Release);
    }
}

fn reserve_orders(siz: usize) {
    unsafe {
        while POOL_LOCK.swap(true, atomic::Ordering::Acquire) {
            std::thread::yield_now();
        }
        ORDER_POOL.reserve(siz);
        POOL_LOCK.store(false, atomic::Ordering::Release);
    }
}

impl Order {
    pub fn new(id: Oid, sym_idx: u32, buy: bool, price: i32, qty: u32)
    -> Order {
        let ret = Order {id, price, sym_idx, qty,
                       buy, ..Default::default() };
        ret
    }
    #[allow(non_snake_case)]
    pub fn to_OidPrice(&self) -> OidPrice {
        if self.buy {
            // negative price, for reverse order
            OidPrice{id: self.id as u32, price: - self.price}
        } else {
            OidPrice{id: self.id as u32, price: self.price}
        }
    }
    pub fn is_buy(&self) -> bool {
        self.buy
    }
    pub fn is_filled(&self) -> bool {
        self.filled == self.qty
    }
    pub fn is_canceled(&self) -> bool {
        self.canceled
    }
    pub fn is_invalid(&self) -> bool {
        self.id == 0 || self.filled > self.qty
    }
    pub fn dir(&self) -> String {
        if self.buy {
            "buy".to_string()
        } else {
            "sell".to_string()
        }
    }
    pub fn status(&self) -> String {
        if self.canceled {
            "canceled".to_string()
        } else if self.is_filled() {
            "filled".to_string()
        } else if self.filled > 0 {
            "part filled".to_string()
        } else {
            "pending".to_string()
        }
    }
    pub fn remain_qty(&self) -> u32 {
        if self.canceled || self.id == 0 {
            0
        } else {
            self.qty - self.filled
        }
    }
    pub fn oid(&self) -> Oid {
        self.id
    }
    // OrderKey for fast index
    pub fn key(&self) -> OrderKey {
        OrderKey::from(self.id)
    }
    pub fn symbol(&self) -> u32 {
        self.sym_idx
    }
    pub fn price(&self) -> i32 {
        self.price
    }
    pub fn qty(&self) -> u32 {
        self.qty
    }
    pub fn fill(&mut self, vol: u32, price: i32) -> bool {
        if self.canceled || self.id == 0 {
            return false
        }
        if vol + self.filled > self.qty {
            self.filled = self.qty
        } else {
            self.filled += vol
        }
        self.price_filled = price;
        true
    }
    pub fn cancel(&mut self) {
        self.canceled = true
    }
}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialEq for OidPrice {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.price == other.price
    }
}

impl Ord for OidPrice {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.price == other.price {
            self.id.cmp(&other.id)
        } else {
            self.price.cmp(&other.price)
        }
    }
}

impl PartialOrd for OidPrice {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Oid({}): qty({}) {} @{} filled({}) -- {}", self.id, self.qty,
                self.dir(), self.price, self.filled, self.status())
    }
}

impl OrderKey {
    pub const fn from(id: Oid) -> OrderKey {
        OrderKey(id as u32)
    }
    pub fn get_mut(&self) -> Option<&'static mut Order> {
        let  v_len: usize;
        unsafe {
            v_len = ORDER_POOL.len();
        }
        if self.0 == 0 || self.0 as usize > v_len {
            None
        } else {
            let id = self.0 - 1;
            let ret: &'static mut Order;
            unsafe {
                ret = &mut ORDER_POOL[id as usize];
            }
            Some(ret)
        }
    }
    pub fn get(&self) -> Option<&'static Order> {
        let  v_len: usize;
        unsafe {
            v_len = ORDER_POOL.len();
        }
        if self.0 == 0 || self.0 as usize > v_len {
            None
        } else {
            let id = self.0 - 1;
            let ret: &'static Order;
            unsafe {
                ret = &ORDER_POOL[id as usize];
            }
            Some(ret)
        }
    }
}

impl OrderPool {
    pub fn new() -> OrderPool {
        INIT.call_once(|| {
            init_orders();
        });
        OrderPool()
    }
    pub fn init(&self) {
        clear_orders();
    }
    pub fn reserve(siz: usize) {
        reserve_orders(siz);
    }
    pub fn new_order(&self, sym_idx: u32, buy: bool, price: i32, qty: u32)
    -> Option<&'static mut Order> {
        let  v_len: usize;
        unsafe {
            v_len = ORDER_POOL.len();
        }
        if v_len >= MAX_ORDERS as usize {
            None
        } else {
            let res: &'static mut Order;
            unsafe {
                while POOL_LOCK.swap(true, atomic::Ordering::Acquire) {
                    std::thread::yield_now();
                }
                let v_len = ORDER_POOL.len() as u64;
                ORDER_POOL.push(Order::new(v_len+1, sym_idx, buy, price, qty));
                POOL_LOCK.store(false, atomic::Ordering::Release);
                res = &mut ORDER_POOL[v_len as usize];
            }
            Some(res)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Oid;
    use super::OidPrice;
    use super::Order;
    use super::OrderKey;
    use super::OrderPool;
    use std::collections::BTreeMap;
    use std::cmp::Ordering;
    use std::mem;
    use measure::Measure;
    use rand::Rng;
    use tcmalloc::TCMalloc;

    #[global_allocator]
    static GLOBAL: TCMalloc = TCMalloc;

    #[test]
    fn order_cmp() {
        let or1=Order::new(1, 1, true, 10000, 100);
        let or2=Order::new(2, 1, true, 11000, 50);
        let or3=Order::new(3, 1, true, 10000, 30);
        let mut or4=Order::new(4, 1, false, 12000, 70);
        assert!(or1 != or2);
        assert!(or1 != or3);
        let op1=or1.to_OidPrice();
        let op2=or2.to_OidPrice();
        let op3=or3.to_OidPrice();
        assert!(op1 > op2);
        assert!(op1 < op3);
        assert!(op2 < op3);
        assert_eq!(op1.cmp(&op2), Ordering::Greater);
        assert_eq!(op1.cmp(&op3), Ordering::Less);
        assert!(or1.is_buy());
        assert!(or2.is_buy());
        assert!(or3.is_buy());
        assert!(! or4.is_buy());
        assert!(! or1.is_filled());
        assert!(! or2.is_filled());
        assert!(! or3.is_filled());
        assert!(! or4.is_filled());
        assert_eq!(or1.oid(), 1);
        assert_eq!(or2.oid(), 2);
        assert_eq!(or3.oid(), 3);
        assert_eq!(or4.oid(), 4);
        assert_eq!(or1.remain_qty(), 100);
        assert_eq!(or2.remain_qty(), 50);
        assert_eq!(or3.remain_qty(), 30);
        assert_eq!(or4.remain_qty(), 70);
        assert!(or4.fill(30, 12500));
        assert_eq!(or4.remain_qty(), 40);
        or4.cancel();
        assert_eq!(or4.remain_qty(), 0);
        println!("sizeof Order: {}", mem::size_of::<Order>());
        println!("sizeof OidPrice: {}", mem::size_of::<OidPrice>());
        println!("sizeof OrderKey: {}", mem::size_of::<OrderKey>());
    }

    #[test]
    fn order_btree() {
        let or1=Order::new(1, 1, true, 10000, 100);
        let or2=Order::new(2, 1, true, 11000, 50);
        let or3=Order::new(3, 1, true, 10000, 30);
        let op1 = or1.to_OidPrice();
        let mut or_maps = BTreeMap::<OidPrice, Box<Order>>::new();
        or_maps.insert(or1.to_OidPrice(), Box::new(or1));
        assert_eq!(or_maps.len(), 1);
        or_maps.insert(or2.to_OidPrice(), Box::new(or2));
        or_maps.insert(or3.to_OidPrice(), Box::new(or3));
        assert_eq!(or_maps.len(), 3);
        // first_entry/last_entry is nightly-only API
        /*
        if let Some(mut entry) = or_maps.first_entry() {
            assert!(entry.get().oid() == 2)
        } else {
            assert!(false, "first entry MUST exist")
        }
        */
        let mut it = or_maps.iter_mut();
        /* need impl Ord for Order
        let it_min = it.min();
        assert!(it_min != None);
        let ord = it_min.unwrap();
        assert_eq!(ord.oid(), 2);
        */
        let (_, ord) = it.next().unwrap();
        assert_eq!(ord.oid(), 2);
        assert_eq!(ord.qty(), 50);
        let (_, ord) = it.next().unwrap();
        assert_eq!(ord.oid(), 1);
        assert_eq!(ord.qty(), 100);
        let (_, ord) = it.next().unwrap();
        assert_eq!(ord.oid(), 3);
        assert_eq!(ord.qty(), 30);
        assert!(ord.fill(10, 10000));
        assert_eq!(ord.remain_qty(), 20);
        // follow need derived(Debug) w/ Order
        assert!(or_maps.remove(&op1) != None);
        // assert_ne/assert_eq need derive(Debug)
        //assert_ne!(or_maps.remove(&op1), None);
        let mut it = or_maps.iter();
        let (_, ord) = it.next().unwrap();
        assert_eq!(ord.oid(), 2);
        let (_, ord) = it.next().unwrap();
        assert!(! ord.is_filled() );
        assert_eq!(ord.oid(), 3);
        assert_eq!(ord.remain_qty(), 20);

        for (_, ord) in or_maps.iter() {
            println!("{}: {}", ord.oid(), ord)
        }
    }

    #[test]
    fn orderpool_test() {
        let pool = OrderPool::new();
        let or1=pool.new_order(1, true, 10000, 100).unwrap();
        let oid1 = or1.oid();
        let ret = or1.key().get();
        assert!(ret != None);
        assert!(ret.unwrap().oid() == oid1);
    }

    #[test]
    #[ignore]
    fn orderpool_btree() {
        let pool = OrderPool::new();
        let mut or_maps = BTreeMap::<OidPrice, OrderKey>::new();
        let or1=pool.new_order(1, true, 10000, 100).unwrap();
        let oid1 = or1.oid();
        or_maps.insert(or1.to_OidPrice(), or1.key());
        assert_eq!(or_maps.len(), 1);
        let ord=pool.new_order(1, true, 11000, 50).unwrap();
        let oid2 = ord.oid();
        or_maps.insert(ord.to_OidPrice(), ord.key());
        let ord=pool.new_order(1, true, 10000, 30).unwrap();
        let oid3 = ord.oid();
        or_maps.insert(ord.to_OidPrice(), ord.key());
        assert_eq!(or_maps.len(), 3);
        let mut it = or_maps.iter_mut();
        let (_, oid) = it.next().unwrap();
        let ord = oid.get().unwrap();
        assert_eq!(ord.oid(), oid2);
        assert_eq!(ord.qty(), 50);
        let (_, oid) = it.next().unwrap();
        let ord = oid.get().unwrap();
        assert_eq!(ord.oid(), oid1);
        let op1 = ord.to_OidPrice();
        assert_eq!(ord.qty(), 100);
        let (_, oid) = it.next().unwrap();
        let ord = oid.get_mut().unwrap();
        assert_eq!(ord.oid(), oid3);
        assert_eq!(ord.qty(), 30);
        assert!(ord.fill(10, 10000));
        assert_eq!(ord.remain_qty(), 20);
        // follow need derived(Debug) w/ Order
        assert!(or_maps.remove(&op1) != None);
        let mut it = or_maps.iter();
        let (_, oid) = it.next().unwrap();
        let ord = oid.get().unwrap();
        assert_eq!(ord.oid(), oid2);
        let (_, oid) = it.next().unwrap();
        let ord = oid.get().unwrap();
        assert!(! ord.is_filled() );
        assert_eq!(ord.oid(), oid3);
        assert_eq!(ord.remain_qty(), 20);

        for (_, oid) in or_maps.iter() {
            let ord = oid.get().unwrap();
            println!("{}: {}", ord.oid(), ord)
        }
    }

    #[test]
    #[ignore]
    fn bench_orderbook_insert() {
        let mut or_maps = BTreeMap::<OidPrice, Box<Order>>::new();
        let mut rng = rand::thread_rng();
        let mut measure = Measure::start("orderbook bench");
        let mut oid: Oid = 0;
        const N: u32 = 2_000_000;
        for _it in 0 .. N {
            let price = rng.gen::<i32>();
            let mut qty: u32 = rng.gen::<u32>();
            let b_buy: bool = (rng.gen::<u32>() & 1) != 0;
            qty %= 1000;
            qty += 1;
            oid += 1;
            let ord = Box::new(Order::new(oid, 1, b_buy, price, qty));
            or_maps.insert(ord.to_OidPrice(), ord);
        }
        measure.stop();
        let ns_ops = measure.as_ns() / (N as u64);
        assert!(ns_ops < 10_000);
        println!("orderBook insert cost {} ns per Op", ns_ops);
    }

    #[test]
    #[ignore]
    fn bench_orderbook_pool_insert() {
        let pool = OrderPool::new();
        let mut or_maps = BTreeMap::<OidPrice, OrderKey>::new();
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
            or_maps.insert(ord.to_OidPrice(), ord.key());
        }
        measure.stop();
        let ns_ops = measure.as_ns() / (N as u64);
        assert!(ns_ops < 10_000);
        println!("orderPool orderBook insert cost {} ns per Op", ns_ops);
    }
}
