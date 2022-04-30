use std::cmp::Ordering;
//use static_init::dynamic;

#[derive(Eq)]
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


//const MAX_ORDERS: u32 = 20000000;
//#[dynamic(lazy)]    //lazy or lesser_lazy are equivalent for thread_local
//static mut ORDER_NO: u32 = 0;

//lazy_static! {
//    static ref OrderNo: u32 = 0;
//}

pub const  fn new(oid: u64, sym_idx: u32, buy: bool, price: i32, qty: u32) -> Order {
//    let ooid = *ORDER_NO.read() + 1;
//    if ooid <= MAX_ORDERS {
//        *ORDER_NO.write() = ooid;
//    }
    // let oid: u64 = *OrderNo as u64;
    Order {id: oid, price, sym_idx, qty,
               filled: 0, buy, canceled: false, price_filled: 0
    }
}


impl Order {
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

#[cfg(test)]
mod tests {
    use crate::order::new;
    use crate::order::OidPrice;
    use crate::order::Order;
    use std::collections::BTreeMap;
    use std::cmp::Ordering;
    use static_init::dynamic;
    use auction_measure::Measure;
    use rand::Rng;

    #[dynamic(lazy)]    //lazy or lesser_lazy are equivalent for thread_local
    static mut NORMAL: u64 = 0;


    #[test]
    fn order_cmp() {
        let oid = *NORMAL.read();
        let or1=new(oid, 1, true, 10000, 100);
        *NORMAL.write() = oid + 1;
        let oid = *NORMAL.read();
        let or2=new(oid, 1, true, 11000, 50);
        *NORMAL.write() = oid + 1;
        let oid = *NORMAL.read();
        let or3=new(oid, 1, true, 10000, 30);
        *NORMAL.write() = oid + 1;
        let oid = *NORMAL.read();
        let mut or4=new(oid, 1, false, 12000, 70);
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
        assert_eq!(or1.oid(), 0);
        assert_eq!(or2.oid(), 1);
        assert_eq!(or3.oid(), 2);
        assert_eq!(or4.oid(), 3);
        assert_eq!(or1.remain_qty(), 0);
        assert_eq!(or2.remain_qty(), 50);
        assert_eq!(or3.remain_qty(), 30);
        assert_eq!(or4.remain_qty(), 70);
        assert!(or4.fill(30, 12500));
        assert_eq!(or4.remain_qty(), 40);
        or4.cancel();
        assert_eq!(or4.remain_qty(), 0);
    }

    #[test]
    fn order_btree() {
        let or1=new(0, 1, true, 10000, 100);
        let or2=new(1, 1, true, 11000, 50);
        let or3=new(2, 1, true, 10000, 30);
        let op1 = or1.to_OidPrice();
        let mut or_maps = BTreeMap::<OidPrice, Box<Order>>::new();
        or_maps.insert(or1.to_OidPrice(), Box::new(or1));
        assert_eq!(or_maps.len(), 1);
        or_maps.insert(or2.to_OidPrice(), Box::new(or2));
        or_maps.insert(or3.to_OidPrice(), Box::new(or3));
        assert_eq!(or_maps.len(), 3);
        let mut it = or_maps.iter_mut();
        let (_, ord) = it.next().unwrap();
        assert_eq!(ord.oid(), 1);
        assert_eq!(ord.qty(), 50);
        let (_, ord) = it.next().unwrap();
        assert_eq!(ord.oid(), 0);
        assert_eq!(ord.qty(), 100);
        let (_, ord) = it.next().unwrap();
        assert_eq!(ord.oid(), 2);
        assert_eq!(ord.qty(), 30);
        assert!(ord.fill(10, 10000));
        assert_eq!(ord.remain_qty(), 20);
        or_maps.remove(&op1);
        let mut it = or_maps.iter();
        let (_, ord) = it.next().unwrap();
        assert_eq!(ord.oid(), 1);
        let (_, ord) = it.next().unwrap();
        assert!(! ord.is_filled() );
        assert_eq!(ord.oid(), 2);
        assert_eq!(ord.remain_qty(), 20);

        for (_, ord) in or_maps.iter() {
            println!("{}: qty {} @{}", ord.oid(), ord.qty(), ord.price())
        }
    }

    #[test]
    fn bench_orderbook_insert() {
        let mut or_maps = BTreeMap::<OidPrice, Box<Order>>::new();
        let mut oid: u64 = 0;
        let mut rng = rand::thread_rng();
        let mut measure = Measure::start("orderbook bench");
        const N: u32 = 1_000_000;
        for _it in 0 .. N {
            oid += 1;
            let price = rng.gen::<i32>();
            let mut qty: u32 = rng.gen::<u32>();
            let b_buy: bool = (rng.gen::<u32>() & 1) != 0;
            qty %= 1000;
            qty += 1;
            let ord = Box::new(new(oid, 1, b_buy, price, qty));
            or_maps.insert(ord.to_OidPrice(), ord);
        }
        measure.stop();
        let ns_ops = measure.as_ns() / (N as u64);
        assert!(ns_ops < 10_000);
        println!("orderBook insert cost {} ns per Op", ns_ops);
    }
}
