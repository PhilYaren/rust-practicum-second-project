use anyhow::Result;
use log::{error, info};
use std::{
    io::{BufRead, BufReader, ErrorKind, Write},
    net::{TcpStream, UdpSocket},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use stock_lib::{constants::MAX_PACKET_SIZE, framing::line_frame, quote::StockQuote};

use crate::constants::MESSAGE_TIMEOUT;
use crate::keepalive::{is_pong_message, spawn_ping_thread};

pub fn connect_and_send_command(tcp_stream: &mut TcpStream, command: &str) -> Result<String> {
    tcp_stream.write_all(&line_frame(command))?;
    tcp_stream.flush()?;

    let mut response = String::new();

    let res_size = {
        let mut reader = BufReader::new(tcp_stream);

        reader.read_line(&mut response)?
    };

    if res_size == 0 {
        return Ok("Stream closed".to_owned());
    }

    Ok(response.trim_end().to_owned())
}

pub fn stock_listener(socket: &UdpSocket, running: Arc<AtomicBool>) -> Result<()> {
    let mut handle = None;

    while running.load(Ordering::Relaxed) {
        let mut buffer = [0; MAX_PACKET_SIZE];

        let (res_length, from) = match socket.recv_from(&mut buffer) {
            Ok(res) => res,
            Err(err) if matches!(err.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                error!(
                    "Connection timed out: no quotes or PONG received for {} seconds",
                    MESSAGE_TIMEOUT.as_secs()
                );

                running.store(false, Ordering::Relaxed);
                break;
            }
            Err(err) => return Err(err.into()),
        };

        if handle.is_none() {
            let ping_socket = (*socket).try_clone()?;
            let running = running.clone();
            info!("UDP session established with server: {from}");
            handle = Some(spawn_ping_thread(ping_socket, from, running));
        }

        let message = &buffer[..res_length];

        // PONG keeps the listener alive even when no matching quotes are produced.
        if is_pong_message(message) {
            continue;
        }

        let quote = match StockQuote::from_wire_bytes(message) {
            Ok(quote) => quote,
            Err(err) => {
                error!("Failed to parse quote: {err}");

                continue;
            }
        };

        println!(
            "QUOTE ticker={} price={} volume={} timestamp={}",
            quote.ticker,
            quote.formatted_price(),
            quote.volume,
            quote.timestamp
        );
    }

    if let Some(ping_thread) = handle {
        let _ = ping_thread.join();
    }

    Ok(())
}
