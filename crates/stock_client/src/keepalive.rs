use log::{error, info};
use std::{
    net::{SocketAddr, UdpSocket},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

use stock_lib::constants::{MSG_PING, MSG_PONG};

use crate::constants::PING_INTERVAL;

pub fn spawn_ping_thread(
    socket: UdpSocket,
    server_addr: SocketAddr,
    running: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        while running.load(Ordering::Relaxed) {
            // Даем время на восстановление
            if let Err(err) = socket.send_to(MSG_PING.as_bytes(), server_addr) {
                error!("Unable to send PING to {server_addr}: {err}");
            }

            thread::sleep(PING_INTERVAL);
        }
        info!("Ping thread stopped");
    })
}

pub fn is_pong_message(message: &[u8]) -> bool {
    message == MSG_PONG.as_bytes()
}
