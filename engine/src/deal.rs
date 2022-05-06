use std::fmt;
//use std::cmp::Ordering;

#[derive(Eq, Clone, Default)]
pub struct Deal {
    no:     u64,
    oid:    u64,
    price:  i32,
    vol:    u32,
}

impl Deal {
    pub const fn new(no: u64, oid: u64, price: i32, vol: u32) -> Deal {
        Deal {no, oid, price, vol}
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
}

impl PartialEq for Deal {
    fn eq(&self, rhs: &Self) -> bool {
        self.no == rhs.no && self.oid == rhs.oid &&
            self.price == rhs.price && self.vol == rhs.vol
    }
}

impl fmt::Display for Deal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "No: {} Oid({}): vol({}) filled @{}", self.no, 
               self.oid, self.vol, self.price)
    }
}
