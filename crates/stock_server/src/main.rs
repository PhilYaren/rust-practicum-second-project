mod args;
mod constants;
mod generator;
mod handlers;
mod state;
mod udp_session;

use anyhow::Result;
use clap::Parser;
use log::{error, info};
use std::{
    net::TcpListener,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};
use stock_lib::quote::StockQuote;

use args::CLIArgs;
use handlers::handle_client;

use generator::spawn_quote_generator;
use state::ServerState;

use crate::constants::TCP_LOOP_SLEEP_MS;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = CLIArgs::parse();
    let running = Arc::new(AtomicBool::new(true));
    let listener = TcpListener::bind(args.tcp_address)?;
    listener.set_nonblocking(true)?;
    let state = ServerState::<StockQuote>::new();

    info!("Server started on: {}", args.tcp_address);

    {
        let running = running.clone();

        ctrlc::set_handler(move || {
            info!("Shutdown signal received");

            running.store(false, Ordering::Relaxed);
        })?;
    }

    let generator_handle = spawn_quote_generator(state.clone(), running.clone());

    while running.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, addr)) => {
                info!("Client connected: {addr}");

                let state = state.clone();
                let running = running.clone();

                thread::spawn(move || {
                    handle_client(stream, state, running);
                });
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(TCP_LOOP_SLEEP_MS);
            }
            Err(err) => {
                error!("Accept error: {err}");
            }
        }
    }

    info!("Server stopped accepting connections");
    let _ = generator_handle.join();
    info!("Server gracefully shut down");
    Ok(())
}
