use std::fmt;
use std::sync::{Once, atomic};

#[derive(Eq, Clone, Default)]
pub struct Deal {
    no:     u64,
    oid:    u64,
    price:  i32,
    qty:    u32,
}

pub struct DealPool ();

const MAX_DEALS: u32 = 30_000_000;
static INIT: Once = Once::new();
static mut DEAL_POOL: Vec<Deal> = Vec::new();
static mut POOL_LOCK: atomic::AtomicBool = atomic::AtomicBool::new(false);
//static mut DEAL_POOL: &mut [Deal] = &mut [];
//unsafe { DEAL_POOL = std::slice::from_raw_parts_mut( data: *mut Order, len: usize) }

// init orders db
fn init_deals() {
    unsafe {
        DEAL_POOL = Vec::<Deal>::with_capacity(2048);
    }
}

pub fn clear_deals() {
    INIT.call_once(|| {
        init_deals();
    });
    unsafe {
        while POOL_LOCK.swap(true, atomic::Ordering::Acquire) {
            std::thread::yield_now();
        }
        DEAL_POOL.clear();
        POOL_LOCK.store(false, atomic::Ordering::Release);
    }
}


impl Deal {
    pub const fn new(no: u64, oid: u64, price: i32, qty: u32) -> Deal {
        Deal {no, oid, price, qty}
    }
    pub fn no(&self) -> u64 {
        self.no
    }
    pub fn oid(&self) -> u64 {
        self.oid
    }
    pub fn price(&self) -> i32 {
        self.price
    }
    pub fn qty(&self) -> u32 {
        self.qty
    }
}

impl DealPool {
    pub fn new() -> DealPool {
        INIT.call_once(|| {
            init_deals();
        });
        DealPool()
    }
    pub fn reserve(siz: usize) {
        unsafe {
                while POOL_LOCK.swap(true, atomic::Ordering::Acquire) {
                    std::thread::yield_now();
                }
            DEAL_POOL.reserve(siz);
            POOL_LOCK.store(false, atomic::Ordering::Release);
        }
    }
    pub fn push_deal(&self, oid: u64, price: i32, qty: u32) -> bool {
        let  v_len: usize;
        unsafe {
            v_len = DEAL_POOL.len();
        }
        if v_len >= MAX_DEALS as usize {
            false
        } else {
            unsafe {
                while POOL_LOCK.swap(true, atomic::Ordering::Acquire) {
                    std::thread::yield_now();
                }
                let v_len = DEAL_POOL.len() as u64;
                DEAL_POOL.push(Deal::new(v_len+1, oid, price, qty));
                POOL_LOCK.store(false, atomic::Ordering::Release);
            }
            true
        }
    }
    pub fn get(&self, idx: u64) -> Option<&'static Deal> {
        let  v_len: usize;
        unsafe {
            v_len = DEAL_POOL.len();
        }
        if idx == 0 || idx > v_len as u64 { return None }
        let ret: &'static Deal;
        unsafe {
            ret = &DEAL_POOL[idx as usize - 1];
        }
        Some(ret)
    }
    pub fn clear(&mut self) {
        clear_deals();
    }
    pub fn eq(&self, v2: &Vec<Deal>) -> bool {
        for adeal in v2 {
            if adeal.no == 0 { break }
            if let Some(vdeal) = self.get(adeal.no()) {
                if vdeal != adeal { return false }
            } else {
                return false
            }
        }
        true
    }
}

impl PartialEq for Deal {
    fn eq(&self, rhs: &Self) -> bool {
        self.no == rhs.no && self.oid == rhs.oid &&
            self.price == rhs.price && self.qty == rhs.qty
    }
}

impl fmt::Display for Deal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "No: {} Oid({}): qty({}) filled @{}", self.no, 
               self.oid, self.qty, self.price)
    }
}

#[cfg(test)]
mod tests {
    use super::{Deal, DealPool};

    #[test]
    fn test_dealv() {
        let deals = DealPool::new();
        let deals1 = vec![Deal::new(1,4, 43500, 45),
                        Deal::new(2, 8, 43500, 45),
                        Deal::new(3, 4, 43500, 5),
                        Deal::new(0, 0, 0, 0) ];
        for de in &deals1 {
            if de.no() == 0 { break }
            deals.push_deal(de.oid(), de.price(), de.qty());
        }
        assert!(deals.eq(&deals1));
    }
}
