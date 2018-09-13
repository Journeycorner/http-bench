# http-bench
Small utility to benchmark http requests by repeating them for a given amount. Implemented using Rust nightly, hyper and tokio. Currently only get requests work. 

Use at your own risk!

## Run
* install rust nightly
* cargo run --release {number of repeats} {uri to benchmark}
