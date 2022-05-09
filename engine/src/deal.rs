use std::fmt;

#[derive(Eq, Clone, Default)]
pub struct Deal {
    no:     u64,
    oid:    u64,
    price:  i32,
    qty:    u32,
}

pub struct Deals {
    ids:    u64,
    v:      Vec<Deal>,
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

impl Deals {
    pub fn new() -> Deals {
        Deals { ids: 0, v: Vec::<Deal>::new() }
    }
    pub fn get(&self, idx: u64) -> Option<&Deal> {
        if idx == 0 || idx > self.ids { return None }
        self.v.get(idx as usize - 1)
    }
    pub fn push_deal(&mut self, oid: u64, price: i32, qty: u32) {
        self.ids += 1;
        let deal = Deal::new(self.ids, oid, price, qty);
        self.v.push(deal);
    }
    pub fn clear(&mut self) {
        self.ids = 0;
        self.v.clear();
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
    use super::{Deal, Deals};

    #[test]
    fn test_dealv() {
        let mut deals = Deals::new();
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
