//! Константы протокола и сетевые ограничения.
//!
//! - [`protocol`] — команды и сообщения протокола;
//! - [`udp`] — ограничения размера UDP-пакетов.

pub mod protocol;
pub mod udp;

pub use protocol::*;
pub use udp::*;
