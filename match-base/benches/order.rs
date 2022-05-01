#[macro_use]
extern crate bencher;
extern crate rand;

use std::collections::BTreeMap;
use bencher::Bencher;
use match_base::{Order, OrderPool, OidPrice};
use rand::Rng;

fn or_book(bench: &mut Bencher) {
    let mut or_maps = BTreeMap::<OidPrice, Box<Order>>::new();
    let mut rng = rand::thread_rng();
    let mut oid: u64 = 0;
    bench.iter(|| {
        let price = rng.gen::<i32>();
        let mut qty: u32 = rng.gen::<u32>();
        let b_buy: bool = (rng.gen::<u32>() & 1) != 0;
        qty %= 1000;
        qty += 1;
        oid += 1;
        let ord = Box::new(Order::new(oid, 1, b_buy, price, qty));
        or_maps.insert(ord.to_OidPrice(), ord);
    })
}

fn or_pool_book(bench: &mut Bencher) {
    let mut or_maps = BTreeMap::<OidPrice, u64>::new();
    let mut pool = OrderPool::new();
    let mut rng = rand::thread_rng();
    bench.iter(|| {
        let price = rng.gen::<i32>();
        let mut qty: u32 = rng.gen::<u32>();
        let b_buy: bool = (rng.gen::<u32>() & 1) != 0;
        qty %= 1000;
        qty += 1;
        let ord = pool.new_order(1, b_buy, price, qty).unwrap();
        or_maps.insert(ord.to_OidPrice(), ord.oid());
    })
}

benchmark_group!(benches, or_book, or_pool_book);
benchmark_main!(benches);
