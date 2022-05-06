use std::collections::HashMap;
use std::fmt;
use std::default::Default;
//use std::sync::Once;

pub struct Symbol {
    name:   String,
    idx:    u32,
    market: u16,     // list on exchange market
    _state:  u8,
    digits: i8,
    _vol_min:    u32,
    _vol_max:    u32,
    lot_size:   u32,
    _vol_step:   u32,
    price_step: u32,
    _turnover_mul:   u32,
}

//static INIT: Once = Once::new();

pub struct Symbols {
    id_map: HashMap<u32, Symbol>,
    name_map: HashMap<String, u32>,
    ids:    u32,
}

//static mut symbols: HashMap<String, Symbol>;

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "name:{} id:{} market({}) digits({}) lot({}) price_step({})",
                self.name, self.idx, self.market, self.digits,
                self.lot_size, self.price_step)
    }
}

impl Default for Symbol {
    fn default() -> Self {
        Symbol {
            name: String::new(), idx: 0,
            market: 1, _state: 1, digits: 0, _vol_min: 1,
            _vol_max: 2000, lot_size: 5, _vol_step: 1,
            price_step: 10, _turnover_mul: 5,
        }
    }
}

impl Symbols {
    pub fn new() -> Symbols {
        Symbols { ids: 0, id_map: HashMap::<u32, Symbol>::new(),
            name_map: HashMap::<String, u32>::new() }
    }
    pub fn get_idx(&self, name: &str) -> Option<u32> {
        if let Some(idx) = self.name_map.get(name) {
            Some(*idx)
        } else {
            None
        }
    }
    pub fn get_symbol(&self, idx: u32) -> Option<&Symbol> {
        self.id_map.get(&idx)
    }
    pub fn add_symbol(&mut self, name: &str) {
        if self.name_map.get(name) != None { return }
        self.ids += 1;
        let sym = Symbol { idx: self.ids,
                    name: name.to_string(),
                    ..Default::default() };
        self.name_map.insert(name.to_string(), self.ids);
        self.id_map.insert(self.ids, sym);
    }
}
