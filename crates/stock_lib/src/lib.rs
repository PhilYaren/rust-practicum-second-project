//! Библиотека для обмена биржевыми котировками по UDP.
//!
//! # Архитектура
//!
//! - [`quote`] — структура котировки акции (`StockQuote`);
//! - [`wire`] — сериализация/десериализация в JSON-байты для передачи по сети;
//! - [`constants`] — константы протокола и ограничения UDP.
//!
//! # Быстрый старт
//!
//! ```
//! use stock_lib::quote::StockQuote;
//!
//! let quote = StockQuote::new("AAPL", 175_00, 8_000);
//! let bytes = quote.to_wire_bytes().unwrap();
//! ```

#[macro_use]
pub mod wire;
pub mod constants;
pub mod framing;
pub mod protocol;
pub mod quote;

pub const MSG_OK: &str = "OK";
pub const MSG_ERR: &str = "ERR";
