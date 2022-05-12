use {
    std::fmt,
    std::sync::{Once, atomic},
    log::{info, warn}
};

// mid ... match id u32 as well
// oid ... order id u32
#[derive(Eq, Clone, Default)]
pub struct Deal {
    no:     u64,
    mid:    u32,
    oid:    u32,
    price:  i32,
    qty:    u32,
}

pub struct DealPool ();

const MAX_DEALS: u32 = 30_000_000;
static INIT: Once = Once::new();
static mut DEAL_POOL: Vec<Deal> = Vec::new();
static mut DEAL_NO: u64 = 0;
static mut MATCH_NO: u32 = 0;
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
        DEAL_NO = 0;
        MATCH_NO = 0;
        DEAL_POOL.clear();
        POOL_LOCK.store(false, atomic::Ordering::Release);
    }
}


impl Deal {
    pub const fn new(no: u64, mid: u32, oid: u32, price: i32, qty: u32)
    -> Deal {
        Deal {no, mid, oid, price, qty}
    }
    pub fn no(&self) -> u64 {
        self.no
    }
    pub fn oid(&self) -> u32 {
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
    pub fn clear(&mut self) {
        clear_deals();
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
    pub fn new_match() {
        unsafe {
            MATCH_NO += 1;
        }
    }
    pub fn push_deal(&self, oid: u32, price: i32, qty: u32) -> bool {
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
                let mid = MATCH_NO;
                let v_len = DEAL_POOL.len() as u64;
                DEAL_POOL.push(Deal::new(v_len+1, mid, oid, price, qty));
                DEAL_NO = DEAL_POOL.len() as u64;
                POOL_LOCK.store(false, atomic::Ordering::Release);
            }
            true
        }
    }
    pub fn get(&self, idx: u64) -> Option<&'static Deal> {
        let  v_len: u64;
        unsafe {
            v_len = DEAL_NO;
        }
        if idx == 0 || idx > v_len { return None }
        let ret: &'static Deal;
        unsafe {
            ret = &DEAL_POOL[idx as usize - 1];
        }
        Some(ret)
    }
    pub fn eq(&self, v2: &Vec<Deal>) -> bool {
        for adeal in v2 {
            if adeal.no == 0 { break }
            if let Some(vdeal) = self.get(adeal.no()) {
                if vdeal != adeal {
                    warn!("deal diff expect {} got {}", adeal, vdeal);
                    return false
                }
            } else {
                warn!("deal({}) not FOUND", adeal.no);
                return false
            }
        }
        info!("deals equal");
        true
    }
}

impl PartialEq for Deal {
    fn eq(&self, rhs: &Self) -> bool {
        self.no == rhs.no && self.oid == rhs.oid && self.mid == rhs.mid &&
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
        let deals1 = vec![Deal::new(1, 0, 4, 43500, 45),
                        Deal::new(2, 0, 8, 43500, 45),
                        Deal::new(3, 0, 4, 43500, 5),
                        Deal::new(0, 0, 0, 0, 0) ];
        for de in &deals1 {
            if de.no() == 0 { break }
            deals.push_deal(de.oid(), de.price(), de.qty());
        }
        assert!(deals.eq(&deals1));
    }
}
