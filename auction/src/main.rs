use clap::Parser;
use simple_logger::SimpleLogger;
use rand::Rng;
use measure::Measure;
use engine::MatchEngine;
#[allow(unused_imports)]
use log::{error, info, warn, LevelFilter};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    file1: String,
    file2: String,

    /// Name of the person to greet
    #[clap(short, long, default_value = "rust-auction")]
    name: String,

    /// Number of times to greet
    #[clap(short, long, default_value_t = 2_000_000)]
    count: u32,
}

fn main() {
    let args = Args::parse();
    SimpleLogger::new().init().unwrap();
    log::set_max_level(LevelFilter::Info);
    info!("{} start!", args.name);
    let sym_name = "cu1906";
    let mut me = MatchEngine::new();
    let instr: u32;
    if let Some(idx) = me.symbol_idx(&sym_name) {
        instr = idx;
    } else {
        warn!("symbols {} not FOUND", &sym_name);
        instr = 0;
    }
    assert!(me.begin_market());
    assert!(me.start_market());
    me.load_orders(instr, &args.file1);
    me.load_orders(instr, &args.file2);
    assert!(args.count >= 10000);
    let mut measure = Measure::start("cross bench");
    let mc_ret = me.match_cross(instr, 50000);
    measure.stop();
    let (last, qty, rem_qty) = mc_ret.unwrap();
    println!("MatchCross last: {}, volume: {}, remain: {}",
             last, qty, rem_qty);
    println!("MatchCross cost {}us", measure.as_us());
    assert!(me.uncross(instr, last, qty), "uncross failed");
    // now benchmark trading continue
    assert!(me.call_auction()); // do nothing currently
    assert!(me.start_trading());
    let mut rng = rand::thread_rng();
    let cnt = args.count;
    let mut measure = Measure::start("TC bench");
    for _i in 0 .. cnt {
        // send order
        let price = (rng.gen::<i32>() % 10000) + 40000;
        let qty: u32 = (rng.gen::<u32>() % 200) + 1;
        let buy: bool = (rng.gen::<u32>() & 1) != 0;
        me.send_order(instr, buy, price, qty);
    }
    measure.stop();
    let ns_ops = measure.as_ns() / (cnt as u64);
    println!("TradingContinue cost {}ms, {} ns per op",
             measure.as_ms(), ns_ops);
    let ops = 1_000_000 * (cnt as u64) / measure.as_us();
    println!("TradingContinue order process: {} per second", ops);
}
