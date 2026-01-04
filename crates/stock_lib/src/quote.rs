//! Модуль содержит структуру котировки акции и методы для её создания.
//!
//! `StockQuote` — основная единица данных, передаваемая между клиентом и сервером
//! по UDP. Сериализация и десериализация в JSON-байты осуществляется через макрос
//! [`impl_json_wire`], который добавляет методы:
//! - [`StockQuote::from_wire_bytes`] — десериализация из байтов;
//! - [`StockQuote::to_wire_bytes`] — сериализация в байты с проверкой размера пакета.
//!
//! Временная метка котировки устанавливается автоматически при создании.
//!
//! # Пример
//!
//! ```
//! use stock_lib::quote::StockQuote;
//!
//! let quote = StockQuote::new("AAPL", 150_00, 10_000);
//! let bytes = quote.to_wire_bytes().unwrap();
//! let restored = StockQuote::from_wire_bytes(&bytes).unwrap();
//! assert_eq!(restored.ticker, "AAPL");
//! ```

use serde::{self, Deserialize, Serialize};

/// Котировка акции: тикер, цена, объём и временная метка.
///
/// Структура сериализуется в JSON для передачи по сети и проверяется на соответствие
/// максимальному размеру UDP-пакета ([`MAX_PACKET_SIZE`](crate::constants::udp::MAX_PACKET_SIZE)).
///
/// # Поля
///
/// - `ticker` — символ акции (например, `"AAPL"`, `"GOOGL"`);
/// - `price` — текущая цена в центах;
/// - `volume` — объём торгов (количество акций);
/// - `timestamp` — время котировки в миллисекундах с UNIX-эпохи.
///
/// # Примеры
///
/// Создание котировки с конкретными значениями:
///
/// ```
/// use stock_lib::quote::StockQuote;
///
/// let quote = StockQuote::new("TSLA", 245_00, 5_000);
/// assert_eq!(quote.ticker, "TSLA");
/// ```
///
/// Генерация случайной котировки для тестирования:
///
/// ```ignore
/// use stock_lib::quote::StockQuote;
///
/// let quote = StockQuote::random_ticker("MSFT");
/// assert_eq!(quote.ticker, "MSFT");
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StockQuote {
    /// Символ акции (тикер), например `"AAPL"`.
    pub ticker: String,

    /// Текущая цена акции в центах.
    pub price: u32,

    /// Объём торгов — количество акций.
    pub volume: u32,

    /// Временная метка котировки в миллисекундах с UNIX-эпохи.
    pub timestamp: u128,
}

impl_json_wire!(StockQuote);

impl StockQuote {
    fn generate_timestamp() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
    }

    /// Создаёт новую котировку с заданными параметрами.
    ///
    /// Временная метка устанавливается автоматически — текущее время в миллисекундах
    /// с UNIX-эпохи.
    ///
    /// # Аргументы
    ///
    /// - `ticker` — символ акции;
    /// - `price` — цена в центах;
    /// - `volume` — объём торгов.
    ///
    /// # Пример
    ///
    /// ```
    /// use stock_lib::quote::StockQuote;
    ///
    /// let quote = StockQuote::new("AAPL", 175_00, 8_000);
    /// ```
    pub fn new(ticker: &str, price: u32, volume: u32) -> Self {
        Self {
            ticker: ticker.to_owned(),
            price,
            volume,
            timestamp: Self::generate_timestamp(),
        }
    }

    pub fn formatted_price(&self) -> String {
        format!("{}.{:02}", self.price / 100, self.price % 100)
    }

    /// Генерирует котировку со случайными ценой и объёмом для указанного тикера.
    ///
    /// Цена выбирается случайно в диапазоне от 1 до 300000 где 1 - это цент. Диапазон цены $0.01–$3 000.00.
    /// Объём — случайное число от 1 до 1000.
    /// Временная метка устанавливается в текущий момент времени.
    ///
    /// Предназначен для тестирования и симуляции данных.
    ///
    /// # Аргументы
    ///
    /// - `ticker_name` — символ акции.
    ///
    /// # Пример
    ///
    /// ```ignore
    /// use stock_lib::quote::StockQuote;
    ///
    /// let quote = StockQuote::random_ticker("GOOGL");
    /// assert_eq!(quote.ticker, "GOOGL");
    /// ```
    #[cfg(feature = "random_generation")]
    pub fn random_ticker(ticker_name: &str) -> Self {
        Self {
            ticker: ticker_name.to_owned(),
            #[allow(clippy::inconsistent_digit_grouping)]
            price: rand::random_range(1..=3_000_00),
            volume: rand::random_range(1..=1000),
            timestamp: Self::generate_timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_price_from_cents() {
        let quote = StockQuote::new("AAPL", 175_25, 1_000);

        assert_eq!(quote.formatted_price(), "175.25");
    }

    #[test]
    fn formats_price_with_leading_zero_cents() {
        let quote = StockQuote::new("AAPL", 175_05, 1_000);

        assert_eq!(quote.formatted_price(), "175.05");
    }

    #[test]
    fn formats_price_less_than_one_unit() {
        let quote = StockQuote::new("AAPL", 5, 1_000);

        assert_eq!(quote.formatted_price(), "0.05");
    }

    #[test]
    fn creates_quote_with_expected_fields() {
        let quote = StockQuote::new("TSLA", 245_00, 5_000);

        assert_eq!(quote.ticker, "TSLA");
        assert_eq!(quote.price, 245_00);
        assert_eq!(quote.volume, 5_000);
        assert!(quote.timestamp > 0);
    }
}
