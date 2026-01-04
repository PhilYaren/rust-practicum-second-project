use anyhow::{Context, Result, bail};
use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use clap::{self, Parser};

#[derive(Parser, Debug)]
#[command(
    name = "stock_client",
    about = "UDP client for receiving stock quotes from stock_server"
)]
pub struct CLIArgs {
    #[arg(
        long = "stock-server",
        default_value = "127.0.0.1:8080",
        help = "TCP address of stock_server"
    )]
    pub stock_server: SocketAddr,

    #[arg(
        long = "udp-address",
        default_value = "127.0.0.1:0",
        help = "Local UDP address for receiving quotes; use port 0 to auto-select a free port"
    )]
    pub udp_address: SocketAddr,

    #[arg(
        long = "tickers-file",
        help = "Path to a file with stock tickers, one ticker per line"
    )]
    tickers_file: PathBuf,

    #[arg(skip)]
    pub tickers: Vec<String>,
}

impl CLIArgs {
    pub fn parse_and_load_tickers() -> Result<Self> {
        let mut args = Self::parse();

        args.tickers = read_tickers_from_file(&args.tickers_file)?;

        Ok(args)
    }

    pub fn update_udp(&mut self, address: SocketAddr) {
        self.udp_address = address;
    }

    pub fn tickers_to_string(&self) -> String {
        self.tickers.join(",")
    }

    pub fn gen_stream_command(&self) -> String {
        format!(
            "STREAM udp://{} {}",
            self.udp_address,
            self.tickers_to_string()
        )
    }
}

fn read_tickers_from_file(path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read tickers file: {}", path.display()))?;

    let tickers = content
        .lines()
        .map(str::trim)
        .filter(|ticker| !ticker.is_empty())
        .map(str::to_uppercase)
        .collect::<Vec<_>>();

    if tickers.is_empty() {
        bail!(
            "Tickers file is empty or contains only blank lines: {}",
            path.display()
        );
    }

    Ok(tickers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn reads_tickers_from_file_and_normalizes_them() {
        let mut file = tempfile::NamedTempFile::new().expect("temp file should be created");

        writeln!(file, "AAPL").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "  tsla  ").unwrap();
        writeln!(file, "googl").unwrap();

        let tickers = read_tickers_from_file(file.path()).expect("tickers should be read");

        assert_eq!(tickers, vec!["AAPL", "TSLA", "GOOGL"]);
    }

    #[test]
    fn rejects_empty_tickers_file() {
        let mut file = tempfile::NamedTempFile::new().expect("temp file should be created");

        writeln!(file, "   ").unwrap();
        writeln!(file, "").unwrap();

        let result = read_tickers_from_file(file.path());

        assert!(result.is_err());
    }
}
