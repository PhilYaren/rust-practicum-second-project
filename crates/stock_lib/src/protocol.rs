use std::net::IpAddr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Commands {
    Stream,
}

#[derive(Debug, Error)]
pub enum CommandsError {
    #[error("Command {0} does not exist")]
    InvalidCommand(String),
}

impl TryFrom<&str> for Commands {
    type Error = CommandsError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim().to_lowercase().as_str() {
            "stream" => Ok(Self::Stream),
            _ => Err(CommandsError::InvalidCommand(value.to_string())),
        }
    }
}

#[derive(Debug)]
pub struct CommandRequest {
    command: Commands,
    ip: IpAddr,
    port: u16,
    tickers: Vec<String>,
}

#[derive(Debug, Error)]
pub enum CommandRequestParseError {
    #[error("Expected command")]
    MissingCommand,
    #[error("{0}")]
    CommandError(#[from] CommandsError),
    #[error("Address must start with udp://")]
    InvalidProtocol,
    #[error("IP was not found")]
    MissingIp,
    #[error("Invalid ip - {0} does not exist")]
    InvalidIp(String),
    #[error("Port was not found")]
    MissingPort,
    #[error("Invalid port: {0}")]
    InvalidPort(String),
    #[error("Tickers aren't provided")]
    MissingTickers,
    #[error("{0} is unknown argument")]
    UnknownArgument(String),
}

impl CommandRequest {
    pub fn new(command: Commands, ip: IpAddr, port: u16, tickers: Vec<String>) -> Self {
        Self {
            command,
            ip,
            port,
            tickers,
        }
    }

    pub fn command(&self) -> Commands {
        self.command
    }

    pub fn ip(&self) -> &IpAddr {
        &self.ip
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn tickers(&self) -> &[String] {
        &self.tickers
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    pub fn add_ticker<T: Into<String>>(&mut self, ticker: T) {
        self.tickers.push(ticker.into());
    }

    pub fn has_ticker(&self, ticker: &str) -> bool {
        self.tickers.iter().any(|item| item == ticker)
    }

    pub fn remove_ticker<T: Into<String>>(&mut self, ticker: T) -> Option<String> {
        let ticker = ticker.into();

        self.tickers
            .iter()
            .position(|elem| elem == &ticker)
            .map(|index| self.tickers.remove(index))
    }
}

impl TryFrom<&str> for CommandRequest {
    type Error = CommandRequestParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut parts = value.split_whitespace();

        let command = parts
            .next()
            .ok_or(CommandRequestParseError::MissingCommand)?
            .to_string();

        let command = Commands::try_from(command.as_str())?;

        let address = parts.next().ok_or(CommandRequestParseError::MissingIp)?;

        let mut address = address
            .strip_prefix("udp://")
            .ok_or(CommandRequestParseError::InvalidProtocol)?
            .split(':');

        let ip_str = address
            .next()
            .filter(|ip| !ip.is_empty())
            .ok_or(CommandRequestParseError::MissingIp)?;

        let ip = ip_str
            .parse::<IpAddr>()
            .map_err(|_| CommandRequestParseError::InvalidIp(ip_str.to_owned()))?;

        let port_unparsed = address
            .next()
            .filter(|ip| !ip.is_empty())
            .ok_or(CommandRequestParseError::MissingPort)?;

        if let Some(extra) = address.next() {
            return Err(CommandRequestParseError::InvalidPort(format!(
                "found extra data({extra}) after \":\" delimiter"
            )));
        }

        let port = port_unparsed
            .parse()
            .map_err(|_| CommandRequestParseError::InvalidPort(port_unparsed.to_string()))?;

        let tickers = parts
            .next()
            .ok_or(CommandRequestParseError::MissingTickers)?
            .split(',')
            .map(|ticker| ticker.trim().to_uppercase())
            .filter(|ticker| !ticker.is_empty())
            .collect::<Vec<_>>();

        if tickers.is_empty() {
            return Err(CommandRequestParseError::MissingTickers);
        }

        if let Some(extra_arg) = parts.next() {
            return Err(CommandRequestParseError::UnknownArgument(
                extra_arg.to_string(),
            ));
        }

        Ok(Self::new(command, ip, port, tickers))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_stream_command() {
        let command = CommandRequest::try_from("STREAM udp://127.0.0.1:34254 AAPL,TSLA")
            .expect("valid command should be parsed");

        assert_eq!(command.command(), Commands::Stream);
        assert_eq!(command.ip().to_string(), "127.0.0.1");
        assert_eq!(command.port(), 34254);
        assert_eq!(command.address(), "127.0.0.1:34254");
        assert_eq!(command.tickers(), &["AAPL".to_string(), "TSLA".to_string()]);
    }

    #[test]
    fn parses_stream_command_and_uppercases_tickers() {
        let command = CommandRequest::try_from("stream udp://127.0.0.1:34254 aapl,tsla,googl")
            .expect("valid command should be parsed");

        assert_eq!(
            command.tickers(),
            &["AAPL".to_string(), "TSLA".to_string(), "GOOGL".to_string(),]
        );
    }

    #[test]
    fn has_ticker_returns_true_for_existing_ticker() {
        let command = CommandRequest::try_from("STREAM udp://127.0.0.1:34254 AAPL,TSLA")
            .expect("valid command should be parsed");

        assert!(command.has_ticker("AAPL"));
        assert!(command.has_ticker("TSLA"));
    }

    #[test]
    fn has_ticker_returns_false_for_missing_ticker() {
        let command = CommandRequest::try_from("STREAM udp://127.0.0.1:34254 AAPL,TSLA")
            .expect("valid command should be parsed");

        assert!(!command.has_ticker("GOOGL"));
    }

    #[test]
    fn parses_commands_case_insensitive() {
        assert!(matches!(Commands::try_from("STREAM"), Ok(Commands::Stream)));
        assert!(matches!(Commands::try_from("stream"), Ok(Commands::Stream)));
        assert!(matches!(Commands::try_from("Stream"), Ok(Commands::Stream)));
    }

    #[test]
    fn rejects_missing_command() {
        let err = CommandRequest::try_from("").unwrap_err();

        assert!(matches!(err, CommandRequestParseError::MissingCommand));
    }

    #[test]
    fn rejects_unknown_command() {
        let err = CommandRequest::try_from("SUBSCRIBE udp://127.0.0.1:34254 AAPL").unwrap_err();

        assert!(matches!(err, CommandRequestParseError::CommandError(_)));
    }

    #[test]
    fn rejects_missing_udp_prefix() {
        let err = CommandRequest::try_from("STREAM 127.0.0.1:34254 AAPL").unwrap_err();

        assert!(matches!(err, CommandRequestParseError::InvalidProtocol));
    }

    #[test]
    fn rejects_invalid_ip() {
        let err = CommandRequest::try_from("STREAM udp://invalid:34254 AAPL").unwrap_err();

        assert!(matches!(err, CommandRequestParseError::InvalidIp(_)));
    }

    #[test]
    fn rejects_missing_port() {
        let err = CommandRequest::try_from("STREAM udp://127.0.0.1 AAPL").unwrap_err();

        assert!(matches!(err, CommandRequestParseError::MissingPort));
    }

    #[test]
    fn rejects_invalid_port() {
        let cases = [
            "STREAM udp://127.0.0.1:bad AAPL",
            "STREAM udp://127.0.0.1:99999 AAPL",
            "STREAM udp://127.0.0.1:34254:9999 AAPL",
        ];

        for case in cases {
            let err = CommandRequest::try_from(case).unwrap_err();

            assert!(
                matches!(err, CommandRequestParseError::InvalidPort(_)),
                "expected invalid port for case: {case}, got: {err:?}"
            );
        }
    }

    #[test]
    fn rejects_missing_tickers() {
        let cases = [
            "STREAM udp://127.0.0.1:34254",
            "STREAM udp://127.0.0.1:34254 ,,,",
        ];

        for case in cases {
            let err = CommandRequest::try_from(case).unwrap_err();

            assert!(
                matches!(err, CommandRequestParseError::MissingTickers),
                "expected missing tickers for case: {case}, got: {err:?}"
            );
        }
    }

    #[test]
    fn rejects_extra_arguments() {
        let err = CommandRequest::try_from("STREAM udp://127.0.0.1:34254 AAPL extra").unwrap_err();

        assert!(matches!(err, CommandRequestParseError::UnknownArgument(_)));
    }
}
