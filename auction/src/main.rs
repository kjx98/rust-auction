use clap::Parser;

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
    println!("Hello {}!", args.name);
    println!("Args name({}), count({}), files({:?},{:?})", args.name,
        args.count, args.file1, args.file2);
}
