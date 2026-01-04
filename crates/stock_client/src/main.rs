mod args;
mod constants;
mod handlers;
mod keepalive;

use anyhow::Result;
use args::CLIArgs;
use handlers::connect_and_send_command;
use log::{error, info};
use std::{
    net::{TcpStream, UdpSocket},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use stock_lib::MSG_OK;

use crate::{constants::MESSAGE_TIMEOUT, handlers::stock_listener};

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut args = CLIArgs::parse_and_load_tickers()?;
    let running = Arc::new(AtomicBool::new(true));

    {
        let running = running.clone();

        ctrlc::set_handler(move || {
            info!("Shutdown signal received");

            running.store(false, Ordering::Relaxed);
        })?;
    }

    let udp_socket = UdpSocket::bind(args.udp_address)?;
    udp_socket.set_read_timeout(Some(MESSAGE_TIMEOUT))?;
    let mut tcp_stream = TcpStream::connect(args.stock_server)?;
    let addr = udp_socket.local_addr()?;

    args.update_udp(addr);

    let command = args.gen_stream_command();
    let resp = connect_and_send_command(&mut tcp_stream, &command)?;

    if !resp.starts_with(MSG_OK) {
        error!("Server terminated connection with reason: {}", &resp);

        return Ok(());
    }
    info!("Connected to stock server: {}", args.stock_server);
    info!("Local UDP address: {}", args.udp_address);
    info!("Subscribed to tickers: {:?}", args.tickers);

    stock_listener(&udp_socket, running)?;

    info!("Client gracefully shut down");
    Ok(())
}
