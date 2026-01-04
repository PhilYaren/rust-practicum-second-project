use log::info;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;
use stock_lib::quote::StockQuote;

use crate::constants::TICKERS;
use crate::state::ServerState;

const QUOTE_MIN_DELAY: u64 = 300;
const QUOTE_MAX_DELAY: u64 = 2000;

pub fn spawn_quote_generator(
    state: ServerState<StockQuote>,
    running: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while running.load(std::sync::atomic::Ordering::Relaxed) {
            let random_index = rand::random_range(0..TICKERS.len());

            let ticker = TICKERS[random_index];

            let rand_quote = StockQuote::random_ticker(ticker);

            let random_millis = rand::random_range(QUOTE_MIN_DELAY..=QUOTE_MAX_DELAY);
            thread::sleep(Duration::from_millis(random_millis));

            state.broadcast(rand_quote);
        }

        info!("Quote generator stopped");
    })
}
