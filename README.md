The OrderBook was inspired by CodingJesus's OrderBook. It can be found here: https://github.com/Tzadiko/Orderbook

The server, as is (with the current main function) can be run with:

RUST_BACKTRACE=1 RUSTFLAGS="--cfg tokio_unstable" cargo run -p server

The RUSTFLAGS is necessary to work with the tokio-console.

The client can be run on a different console with:

cargo run -p client

respectively.
