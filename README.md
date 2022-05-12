# rust-auction
prototype of Match Engine in rust-lang
try BTree for orderbook

The value of free software is to be found in its ability to allow us to actually own and maintain control over our systems.

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
<br/>
Benchmark for uncross 2 million orders
<pre>
---- engine::tests::bench_cross stdout ----
load 1000000 orders from /tmp/long.txt.zst cost 594ms
load 1000000 orders from /tmp/short.txt.zst cost 573ms
MatchCross last: 50500, volume: 2753442, remain: 25718
MatchCross cost 51619us
MatchUnCross cost 122845us
After uncross qlen: 499392/499164
</pre>
<br/>
Benchmark uncross and trading continue(total 4 millio orders)
<pre>
2022-05-12T04:02:17.615Z WARN [engine::engine::tests] SimpleLogger init: attempted to set a logger after the logging system was already initialized
load 1000000 orders from /tmp/long.txt.zst cost 637ms
load 1000000 orders from /tmp/short.txt.zst cost 566ms
MatchCross last: 50500, volume: 2753442, remain: 25718
MatchCross cost 48442us
MatchUnCross cost 111169us
After uncross qlen: 499392/499164
TradingContinue cost 695ms, 347 ns per op
TradingContinue order process: 2876005 per second
</pre>
[![asciicast](https://asciinema.org/a/493740.svg)](https://asciinema.org/a/493740)

## TODO -- done
Benchmark Continue match (btree for orderBook)
Cross for 2 million orders, buy/sell half/half
