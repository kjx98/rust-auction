use std::cmp::Ordering;
use std::fmt;
//use std::string::String;
//use std::sync::Once;

//#[repr(align(16))]
#[derive(Eq, Clone)]
pub struct Order {
    id:     u64,
    price:  i32,
    sym_idx: u32,
    qty:    u32,
    filled: u32,
    buy:   bool,
    canceled:  bool,
    price_filled:i32,
}

// for use for orderBook order, bid high as best while ask low as best
// bid(buy) order, negative price as order price
#[derive(Eq, Clone)]
pub struct OidPrice {
    id:     u32,
    price:  i32,
}

pub struct OrderPool {
    len:    u64,
    v:      std::vec::Vec<Order>,
}

const MAX_ORDERS: u32 = 60_000_000;
//static INIT: Once = Once::new();

/*
// init orders db
fn init_orders() {
}

fn new_oid() -> u64 {
    unsafe {
        INIT.call_once(|| {
            init_orders();
        });
        if ORDER_NO >= MAX_ORDERS {
            return 0
        }
        ORDER_NO += 1;
        let oid = ORDER_NO as u64;
        oid
    }
}

pub fn init_oid() {
    INIT.call_once(|| {
        init_orders();
    });
    unsafe {
        ORDER_NO = 0
    }
}
*/


impl Order {
    pub const fn new(id: u64, sym_idx: u32, buy: bool, price: i32, qty: u32) -> Order {
        let ret = Order {id, price, sym_idx, qty,
                       filled: 0, buy, canceled: false, price_filled: 0
                    };
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
    pub fn oid(&self) -> u64 {
        self.id
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

impl OrderPool {
    pub fn new() -> OrderPool {
        OrderPool { len: 0, v: Vec::<Order>::with_capacity(2048) }
    }
    pub fn init(&mut self) {
        self.v.clear();
        self.len = 0;
    }
    pub fn new_order(&mut self, sym_idx: u32, buy: bool, price: i32, qty: u32) -> Option<&Order> {
        self.len = self.v.len() as u64;
        if self.len >= MAX_ORDERS as u64 {
            None
        } else {
            self.v.push(Order::new(self.len+1, sym_idx, buy, price, qty));
            let res = &self.v[self.len as usize];
            self.len += 1;
            Some(res)
        }
    }
    pub fn get_order_mut(&mut self, oid: u64) -> Option<&mut Order> {
        if oid == 0 || oid as usize > self.v.len() {
            None
        } else {
            let id = oid - 1;
            Some(&mut self.v[id as usize])
        }
    }
    pub fn get_order(&self, oid: u64) -> Option<&Order> {
        if oid == 0 || oid as usize > self.v.len() {
            None
        } else {
            let id = oid - 1;
            Some(&self.v[id as usize])
        }
    }
}

#[cfg(test)]
mod tests {
    //use super::new;
    use super::OidPrice;
    use super::Order;
    use super::OrderPool;
    //use super::init_oid;
    use std::collections::BTreeMap;
    use std::cmp::Ordering;
    use std::mem;
    use auction_measure::Measure;
    use rand::Rng;

    #[test]
    fn order_cmp() {
        //init_oid();
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
    }

    #[test]
    fn order_btree() {
        //init_oid();
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
        let mut it = or_maps.iter_mut();
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
    fn orderpool_btree() {
        //init_oid();
        let mut pool = OrderPool::new();
        let mut or_maps = BTreeMap::<OidPrice, u64>::new();
        let or1=pool.new_order(1, true, 10000, 100).unwrap();
        or_maps.insert(or1.to_OidPrice(), or1.oid());
        assert_eq!(or_maps.len(), 1);
        let ord=pool.new_order(1, true, 11000, 50).unwrap();
        or_maps.insert(ord.to_OidPrice(), ord.oid());
        let ord=pool.new_order(1, true, 10000, 30).unwrap();
        or_maps.insert(ord.to_OidPrice(), ord.oid());
        assert_eq!(or_maps.len(), 3);
        let mut it = or_maps.iter_mut();
        let (_, oid) = it.next().unwrap();
        let ord = pool.get_order(*oid).unwrap();
        assert_eq!(ord.oid(), 2);
        assert_eq!(ord.qty(), 50);
        let (_, oid) = it.next().unwrap();
        let ord = pool.get_order(*oid).unwrap();
        assert_eq!(ord.oid(), 1);
        let op1 = ord.to_OidPrice();
        assert_eq!(ord.qty(), 100);
        let (_, oid) = it.next().unwrap();
        let ord = pool.get_order_mut(*oid).unwrap();
        assert_eq!(ord.oid(), 3);
        assert_eq!(ord.qty(), 30);
        assert!(ord.fill(10, 10000));
        assert_eq!(ord.remain_qty(), 20);
        // follow need derived(Debug) w/ Order
        assert!(or_maps.remove(&op1) != None);
        let mut it = or_maps.iter();
        let (_, oid) = it.next().unwrap();
        let ord = pool.get_order(*oid).unwrap();
        assert_eq!(ord.oid(), 2);
        let (_, oid) = it.next().unwrap();
        let ord = pool.get_order(*oid).unwrap();
        assert!(! ord.is_filled() );
        assert_eq!(ord.oid(), 3);
        assert_eq!(ord.remain_qty(), 20);

        for (_, oid) in or_maps.iter() {
            let ord = pool.get_order(*oid).unwrap();
            println!("{}: {}", ord.oid(), ord)
        }
    }

    #[test]
    fn bench_orderbook_insert() {
        //init_oid();
        let mut or_maps = BTreeMap::<OidPrice, Box<Order>>::new();
        let mut rng = rand::thread_rng();
        let mut measure = Measure::start("orderbook bench");
        let mut oid: u64 = 0;
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
    fn bench_orderbook_pool_insert() {
        //init_oid();
        let mut pool = OrderPool::new();
        let mut or_maps = BTreeMap::<OidPrice, u64>::new();
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
            or_maps.insert(ord.to_OidPrice(), ord.oid());
        }
        measure.stop();
        let ns_ops = measure.as_ns() / (N as u64);
        assert!(ns_ops < 10_000);
        println!("orderPool orderBook insert cost {} ns per Op", ns_ops);
    }
}
