use std::time::Duration;

pub const TICKERS: [&str; 110] = [
    "AAPL", "MSFT", "GOOGL", "AMZN", "NVDA", "META", "TSLA", "JPM", "JNJ", "V", "PG", "UNH", "HD",
    "DIS", "PYPL", "NFLX", "ADBE", "CRM", "INTC", "CSCO", "PFE", "ABT", "TMO", "ABBV", "LLY",
    "PEP", "COST", "TXN", "AVGO", "ACN", "QCOM", "DHR", "MDT", "NKE", "UPS", "RTX", "HON", "ORCL",
    "LIN", "AMGN", "LOW", "SBUX", "SPGI", "INTU", "ISRG", "T", "BMY", "DE", "PLD", "CI", "CAT",
    "GS", "UNP", "AMT", "AXP", "MS", "BLK", "GE", "SYK", "GILD", "MMM", "MO", "LMT", "FISV", "ADI",
    "BKNG", "C", "SO", "NEE", "ZTS", "TGT", "DUK", "ICE", "BDX", "PNC", "CMCSA", "SCHW", "MDLZ",
    "TJX", "USB", "CL", "EMR", "APD", "COF", "FDX", "AON", "WM", "ECL", "ITW", "VRTX", "D", "NSC",
    "PGR", "ETN", "FIS", "PSA", "KLAC", "MCD", "ADP", "APTV", "AEP", "MCO", "SHW", "DD", "ROP",
    "SLB", "HUM", "BSX", "NOC", "EW",
];
pub const CLIENT_INACTIVITY_TIMEOUT_SECS: Duration = Duration::from_secs(5);
pub const SERVER_UDP_BIND_ADDR: &str = "127.0.0.1:0";
pub const TCP_LOOP_SLEEP_MS: Duration = Duration::from_millis(100);
