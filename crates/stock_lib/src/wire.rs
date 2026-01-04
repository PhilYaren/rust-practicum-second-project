use thiserror::Error;

/// Ошибки сериализации/десериализации данных для передачи по сети.
#[derive(Debug, Error)]
pub enum WireError {
    /// Ошибка JSON-сериализации или десериализации.
    #[error("{0}")]
    Json(serde_json::Error),

    /// Сериализованный пакет превышает максимальный размер UDP-пакета.
    #[error("Превышен размер пакета {max_size} - размер составил {actual_size}")]
    PackageSizeExceeded {
        /// Максимально допустимый размер пакета в байтах.
        max_size: usize,
        /// Фактический размер сериализованного пакета в байтах.
        actual_size: usize,
    },
}

/// Результат операции сериализации или десериализации.
pub type WireResult<T> = Result<T, WireError>;

/// Добавляет к типу методы сериализации и десериализации через JSON.
///
/// Макрос генерирует два метода:
/// - `from_wire_bytes(bytes: &[u8])` — десериализация из JSON-байтов;
/// - `to_wire_bytes(&self)` — сериализация в JSON-байты с проверкой,
///   что результат не превышает [`MAX_PACKET_SIZE`]($crate::constants::udp::MAX_PACKET_SIZE).
///
/// # Пример
///
/// ```
/// use serde::{Serialize, Deserialize};
/// use stock_lib::impl_json_wire;
///
/// #[derive(Serialize, Deserialize)]
/// pub struct MyData {
///     value: u32,
/// }
///
/// impl_json_wire!(MyData);
/// ```
#[macro_export]
macro_rules! impl_json_wire {
    ($t:ty) => {
        impl $t {
            /// Десериализует структуру из JSON-байтов.
            pub fn from_wire_bytes(bytes: &[u8]) -> $crate::wire::WireResult<Self> {
                serde_json::from_slice(bytes).map_err($crate::wire::WireError::Json)
            }

            /// Сериализует структуру в JSON-байты.
            ///
            /// Возвращает [`WireError::PackageSizeExceeded`]($crate::wire::WireError::PackageSizeExceeded),
            /// если размер результата превышает [`MAX_PACKET_SIZE`]($crate::constants::udp::MAX_PACKET_SIZE).
            pub fn to_wire_bytes(&self) -> $crate::wire::WireResult<Vec<u8>> {
                match serde_json::to_vec(self) {
                    Ok(bytes) => {
                        if bytes.len() > $crate::constants::udp::MAX_PACKET_SIZE {
                            return Err($crate::wire::WireError::PackageSizeExceeded {
                                max_size: $crate::constants::udp::MAX_PACKET_SIZE,
                                actual_size: bytes.len(),
                            });
                        }

                        Ok(bytes)
                    }
                    Err(e) => Err($crate::wire::WireError::Json(e)),
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::udp::MAX_PACKET_SIZE;
    use crate::quote::StockQuote;

    #[test]
    fn stock_quote_wire_roundtrip() {
        let quote = StockQuote::new("AAPL", 175_25, 1_000);
        let bytes = quote.to_wire_bytes().expect("quote should serialize");
        let decoded = StockQuote::from_wire_bytes(&bytes).expect("quote should deserialize");

        assert_eq!(decoded, quote);
    }

    #[test]
    fn stock_quote_rejects_invalid_wire_bytes() {
        let bytes = b"{not valid json";

        let result = StockQuote::from_wire_bytes(bytes);

        assert!(matches!(result, Err(WireError::Json(_))));
    }

    #[test]
    fn stock_quote_serialized_packet_fits_udp_limit() {
        let quote = StockQuote::new("AAPL", 175_25, 1_000);

        let bytes = quote.to_wire_bytes().expect("quote should serialize");

        assert!(bytes.len() <= MAX_PACKET_SIZE);
    }
}
