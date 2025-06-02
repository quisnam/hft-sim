This repository includes an OrderBook, a server using that OrderBook and a simple-client to test the server and OrderBook.

The Client sends creates OrderRequests and sends them to the Server via TCP. The Server then parses the byte stream and adds
the orders to the OrderBook.

An Order is either on the Sell or on the Buy Side, and when added to the OrderBook, is first matched against existing Orders within the OrderBook that are on the other Side.
If the requirements stated in the OrderRequest are met, a Trade against that order is executed, and its meta data is send to the Server. 
The Server then notifies the issuers of the two involved (in the Trade) involved parties via TCP again.


The OrderBook was inspired by CodingJesus's OrderBook. It can be found here: https://github.com/Tzadiko/Orderbook

The server, as is (with the current main function), can be run with:

RUST_BACKTRACE=1 RUSTFLAGS="--cfg tokio_unstable" cargo run -p server

The RUSTFLAGS is necessary to work with the tokio-console.

The client can be run on a different console with:

cargo run -p client

respectively.
