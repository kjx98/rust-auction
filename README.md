# rust-auction
prototype of Match Engine in rust-lang
try BTree for orderbook

## test

[![asciicast](https://asciinema.org/a/491201.svg)](https://asciinema.org/a/491201)

### test and orderBook benchmark
<pre>
cargo test
cargo bench
</pre>

## performance

Benchmark order insert (btree for orderBook)
<pre>
running 1 test
test or_book ... bench:         336 ns/iter (+/- 28)

test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured
</pre>

## TODO
Benchmark Cross/Continue match (btree for orderBook)
Cross for 2 million orders, buy/sell half/half
