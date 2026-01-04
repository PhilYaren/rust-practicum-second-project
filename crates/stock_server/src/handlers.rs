use log::{error, info};
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpStream, UdpSocket},
    sync::{Arc, atomic::AtomicBool},
};

use stock_lib::{
    MSG_ERR, MSG_OK, framing::line_frame, protocol::CommandRequest, quote::StockQuote,
};

use crate::udp_session::udp_session;
use crate::{constants::SERVER_UDP_BIND_ADDR, state::ServerState};

fn send_tcp_message(stream: &mut TcpStream, message: &str) -> std::io::Result<()> {
    stream.write_all(&line_frame(message))?;
    stream.flush()?;
    Ok(())
}

pub fn handle_client(
    mut stream: TcpStream,
    state: ServerState<StockQuote>,
    running: Arc<AtomicBool>,
) {
    let mut data = String::new();

    let read_bytes = {
        let mut reader = BufReader::new(&stream);

        match reader.read_line(&mut data) {
            Ok(bytes) => bytes,
            Err(err) => {
                error!(
                    "An error has occurred while reading data from: {}. Error: {}",
                    stream
                        .peer_addr()
                        .map(|socket| socket.to_string())
                        .unwrap_or_else(|_| "Unknown address".to_string()),
                    err
                );

                return;
            }
        }
    };

    if read_bytes == 0 {
        info!("Client has closed connection");

        return;
    }

    let command = match CommandRequest::try_from(data.trim_end()) {
        Ok(command) => command,
        Err(e) => {
            error!("Parser error: {e}");
            let message = format!("{MSG_ERR} {e}");

            if let Err(err) = send_tcp_message(&mut stream, &message) {
                error!("Failed to send err response. {err}")
            };

            return;
        }
    };

    if let Err(err) = send_tcp_message(&mut stream, MSG_OK) {
        error!("An error has occurred while writing Ok response: {err}");

        return;
    }
    info!(
        "STREAM accepted: client_udp={}, tickers={:?}",
        command.address(),
        command.tickers(),
    );

    let udp_socket = match UdpSocket::bind(SERVER_UDP_BIND_ADDR) {
        Ok(stream) => stream,
        Err(err) => {
            error!("Failed to create UDP socket: {err}");

            return;
        }
    };

    if let Err(err) = udp_socket.set_nonblocking(true) {
        error!(
            "Failed to set UDP socket nonblocking for client {}: {err}",
            command.address()
        );

        return;
    }

    // UDP socket is connected to a single client session.
    // After connection send/recv are used instead of send_to/recv_from.
    if let Err(err) = udp_socket.connect(command.address()) {
        error!(
            "Could not connect socket to client {} due to {}",
            command.address(),
            err
        );

        return;
    };

    udp_session(&udp_socket, &command, &state, running);
}
