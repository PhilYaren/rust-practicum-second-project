use std::net::SocketAddr;

use clap::{self, Parser};

#[derive(Parser, Debug)]
#[command(name = "stock_server", about = "TCP/UDP stock quotes streaming server")]
pub struct CLIArgs {
    #[arg(
        long = "tcp-address",
        default_value = "127.0.0.1:8080",
        help = "TCP address for accepting client STREAM commands"
    )]
    pub tcp_address: SocketAddr,
}
