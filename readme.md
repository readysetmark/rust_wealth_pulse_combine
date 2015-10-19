# Wealth Pulse

The latest implementation! Because I just can't quit rewriting this in new
languages!

Plan this time around in to use Rust for the parser and query engine (or
basically all the backend stuff), and use Electron for the UI.

This time I will have a separate process (probably written in Rust again) for
scraping commodity prices that can be run on-demand.

TBD:

- Use JS+react within Electron or Elm?

- How will communication happen between Rust and Electron? Use ZeroMQ or
Nanomsg?


## Tasks:

### Parsing

[ ] Parse ledger file
	[ ] Parsing routines
	[ ] Autobalance transactions
	[ ] Ensure transactions balance (if not autobalanced)

[ ] Parse pricedb file
	[ ] Parsing routines

[ ] Parse config file
	[ ] Parsing routines


### Balance Report

### Register Report

### Net Worth Report

### Price Fetching