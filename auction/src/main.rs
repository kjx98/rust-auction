use clap::Parser;
use simple_logger::SimpleLogger;
use log::{error, info, warn, LevelFilter};
use tcmalloc::TCMalloc;

#[global_allocator]
static GLOBAL: TCMalloc = TCMalloc;


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    file1: Option<String>,
    file2: Option<String>,

    /// Name of the person to greet
    #[clap(short, long, default_value = "test")]
    name: String,

    /// Number of times to greet
    #[clap(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    let args = Args::parse();
    SimpleLogger::new().init().unwrap();
    log::set_max_level(LevelFilter::Info);
    info!("Hello {}!", args.name);
    info!("Args name({}), count({}), files({:?},{:?})", args.name,
        args.count, args.file1, args.file2);
    warn!("ready to exit");
    error!("no error");
}
