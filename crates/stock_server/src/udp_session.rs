use anyhow::Error;
use log::{error, info};
use std::{
    net::UdpSocket,
    sync::{Arc, atomic::AtomicBool, atomic::Ordering},
    time::{Duration, Instant},
};

use crossbeam_channel::RecvTimeoutError;
use stock_lib::{
    constants::{MAX_PACKET_SIZE, MSG_PING, MSG_PONG},
    protocol::CommandRequest,
    quote::StockQuote,
};

use crate::{constants::CLIENT_INACTIVITY_TIMEOUT_SECS, state::ServerState};

const MAX_QUOTE_AWAIT_TICK: u64 = 100;

fn process_keepalive(socket: &UdpSocket, last_ping: &mut Instant) -> bool {
    let mut buffer = [0; MAX_PACKET_SIZE];

    // Вычерпываем все сообщения до ошибки, либо исчерпания всех сообщений WouldBlock
    loop {
        match socket.recv(&mut buffer) {
            Ok(usize) => {
                let message = &buffer[..usize];

                if message == MSG_PING.as_bytes() {
                    *last_ping = Instant::now();

                    if let Err(err) = socket.send(MSG_PONG.as_bytes()) {
                        error!("Failed to send PONG: {}", err);
                    };
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                break;
            }
            Err(err) => {
                error!("Failed to read UDP ping: {err}");
                break;
            }
        }
    }

    last_ping.elapsed() <= CLIENT_INACTIVITY_TIMEOUT_SECS
}

fn send_quote(socket: &UdpSocket, quote: &StockQuote) -> Result<(), Error> {
    let byte_quote = quote.to_wire_bytes()?;

    socket.send(byte_quote.as_slice())?;

    Ok(())
}

pub fn udp_session(
    socket: &UdpSocket,
    command: &CommandRequest,
    state: &ServerState<StockQuote>,
    running: Arc<AtomicBool>,
) {
    let rx = state.subscribe();
    let mut last_ping = Instant::now();
    let client_addr = command.address();

    while running.load(Ordering::Relaxed) {
        let maybe_quote = match rx.recv_timeout(Duration::from_millis(MAX_QUOTE_AWAIT_TICK)) {
            Ok(quote) => Some(quote),
            Err(RecvTimeoutError::Timeout) => None,
            Err(err) => {
                error!("Receiver closed for {} subscriber. {}", client_addr, err);

                return;
            }
        };

        // проверка на активность сессии
        if !process_keepalive(socket, &mut last_ping) {
            info!("Connection with {client_addr} terminated due to inactivity");

            return;
        }

        let quote = match maybe_quote {
            Some(quote) => quote,
            None => {
                // Не пришла котировка - идем на след заход
                continue;
            }
        };

        if !command.has_ticker(&quote.ticker) {
            continue;
        }

        if let Err(err) = send_quote(socket, &quote) {
            error!("Failed to send ticker due to: {}", err);

            continue;
        };
    }

    info!("UDP session with {} has been closed", client_addr);
}
