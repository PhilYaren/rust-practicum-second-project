# Stock Quotes Streaming

Учебный проект на Rust для стриминга биржевых котировок по сети.

Проект реализует клиент-серверную систему:

- сервер принимает TCP-команды от клиентов;
- сервер генерирует поток случайных котировок в отдельном потоке;
- клиенты подписываются на интересующие тикеры через файл;
- котировки доставляются клиентам по UDP;
- живость UDP-сессии поддерживается через `PING/PONG`;
- сервер и клиент поддерживают graceful shutdown через `Ctrl+C`.

## Структура проекта

Проект организован как Cargo workspace:

```text
.
├── Cargo.toml
├── crates
│   ├── stock_lib
│   ├── stock_server
│   └── stock_client
```

### `stock_lib`

Общая библиотека для клиента и сервера.

Содержит:

- модель котировки `StockQuote`;
- сериализацию и десериализацию котировок;
- парсинг TCP-команды `STREAM`;
- константы протокола и UDP-буфера;
- line-based TCP framing;
- unit-тесты для протокола, wire-сериализации, framing и форматирования цены.

### `stock_server`

Сервер котировок.

Отвечает за:

- приём TCP-подключений;
- обработку команды `STREAM`;
- запуск общего генератора котировок;
- хранение подписчиков;
- отправку подходящих котировок клиентам по UDP;
- обработку `PING`;
- отправку `PONG`;
- graceful shutdown.

### `stock_client`

Клиент котировок.

Отвечает за:

- чтение тикеров из файла;
- открытие локального UDP-сокета;
- отправку TCP-команды `STREAM`;
- получение UDP-котировок;
- запуск отдельного ping thread;
- обработку `PONG`;
- graceful shutdown.

## Протокол

### TCP control channel

TCP используется для начальной команды подписки.

Клиент отправляет серверу строку:

```text
STREAM udp://<client_udp_ip>:<client_udp_port> <TICKER_1>,<TICKER_2>,...
```

Пример:

```text
STREAM udp://127.0.0.1:54321 AAPL,TSLA
```

Сервер отвечает:

```text
OK
```

или:

```text
ERR <reason>
```

TCP-сообщения используют line-based framing: одно сообщение заканчивается `\n`.

### UDP data channel

После успешной TCP-команды сервер создаёт UDP-сессию для клиента и начинает отправлять котировки.

Котировки передаются в JSON-формате.

Пример логического сообщения:

```json
{
  "ticker": "AAPL",
  "price": 17500,
  "volume": 1000,
  "timestamp": 1234567890
}
```

`price` хранится как целое число в центах:

```text
17500 -> 175.00
```

Для пользовательского вывода клиент форматирует цену как десятичное значение:

```text
QUOTE ticker=AAPL price=175.00 volume=1000 timestamp=1234567890
```

### Keepalive: PING/PONG

Для проверки живости UDP-сессии используется прикладной keepalive:

```text
client -> server: PING
server -> client: PONG
```

Схема работы:

1. Клиент получает первый UDP-пакет от сервера.
2. Из `recv_from` клиент узнаёт UDP-адрес серверной сессии.
3. Клиент запускает отдельный ping thread.
4. Ping thread периодически отправляет `PING`.
5. Сервер получает `PING`, обновляет `last_ping` и отправляет `PONG`.
6. Если сервер не получает `PING` больше заданного времени, UDP-сессия клиента завершается.
7. Если клиент не получает ни котировки, ни `PONG`, он завершает listener по timeout.

`PONG` нужен клиенту, чтобы сессия оставалась активной даже в случае, когда подходящие котировки временно не приходят.

## Архитектура сервера

Сервер запускает один общий генератор котировок:

```text
quote generator thread
  └── генерирует StockQuote
  └── рассылает котировку всем подписчикам
```

Клиенты подписываются через `ServerState`:

```text
ServerState
  └── Arc<Mutex<Vec<Sender<StockQuote>>>>
```

Для каждого клиента создаётся отдельный канал:

```text
client 1 -> Receiver<StockQuote>
client 2 -> Receiver<StockQuote>
client 3 -> Receiver<StockQuote>
```

Генератор рассылает каждую котировку всем `Sender`, а клиентская UDP-сессия фильтрует котировки по тикерам конкретного клиента.

Схема:

```text
generator
  └── broadcast quote
        ├── client session A
        │     └── filter by tickers
        │     └── send UDP quote
        └── client session B
              └── filter by tickers
              └── send UDP quote
```

## Файл тикеров

Клиент принимает путь к файлу тикеров через `--tickers-file`.

Пример файла `tickers.txt`:

```text
AAPL
TSLA

  googl
AMZN
```

При чтении файла клиент:

- читает тикеры построчно;
- обрезает пробелы вокруг строки;
- игнорирует пустые строки;
- нормализует тикеры в uppercase.

Пример выше будет преобразован в:

```text
AAPL,TSLA,GOOGL,AMZN
```

Если файл отсутствует, не читается или не содержит ни одного тикера, клиент завершится с ошибкой.

## Запуск

### Запуск сервера

```bash
cargo run -p stock_server
```

По умолчанию сервер слушает TCP-адрес:

```text
127.0.0.1:8080
```

Можно указать другой адрес:

```bash
cargo run -p stock_server -- --tcp-address 127.0.0.1:9000
```

### Запуск клиента

```bash
cargo run -p stock_client -- --tickers-file tickers.txt
```

По умолчанию клиент:

- подключается к серверу `127.0.0.1:8080`;
- открывает локальный UDP-сокет на `127.0.0.1:0`.

Порт `0` означает, что операционная система сама выберет свободный UDP-порт.

Пример с явными аргументами:

```bash
cargo run -p stock_client -- \
  --stock-server 127.0.0.1:8080 \
  --udp-address 127.0.0.1:0 \
  --tickers-file tickers.txt
```

## CLI help

Сервер:

```bash
cargo run -p stock_server -- --help
```

Клиент:

```bash
cargo run -p stock_client -- --help
```

Основные аргументы клиента:

```text
--stock-server <ADDR>     TCP address of stock_server
--udp-address <ADDR>      Local UDP address for receiving quotes
--tickers-file <PATH>     Path to a file with stock tickers, one ticker per line
```

Основной аргумент сервера:

```text
--tcp-address <ADDR>      TCP address for accepting client STREAM commands
```

## Проверка несколькими клиентами

Терминал 1:

```bash
cargo run -p stock_server
```

Подготовить файлы:

```bash
echo AAPL > tickers-aapl.txt
echo TSLA > tickers-tsla.txt
printf "GOOGL\nAMZN\n" > tickers-mixed.txt
```

Терминал 2:

```bash
cargo run -p stock_client -- --tickers-file tickers-aapl.txt
```

Терминал 3:

```bash
cargo run -p stock_client -- --tickers-file tickers-tsla.txt
```

Терминал 4:

```bash
cargo run -p stock_client -- --tickers-file tickers-mixed.txt
```

Ожидаемое поведение:

- каждый клиент получает только котировки по своим тикерам;
- сервер использует один общий генератор котировок;
- при отсутствии подходящих котировок клиент остаётся активен за счёт `PING/PONG`;
- при остановке клиента сервер через timeout завершает соответствующую UDP-сессию.

## Graceful shutdown

Сервер и клиент поддерживают завершение через `Ctrl+C`.

Для graceful shutdown используется общий флаг:

```rust
Arc<AtomicBool>
```

При получении `Ctrl+C` флаг переводится в `false`, после чего рабочие циклы постепенно завершаются.

На сервере останавливаются:

- accept loop;
- генератор котировок;
- UDP-сессии клиентов.

На клиенте останавливаются:

- UDP listener;
- ping thread.

## Логирование

Проект использует `log` и `env_logger`.

По умолчанию включён уровень `info`.

Запуск с явным уровнем логирования:

```bash
RUST_LOG=info cargo run -p stock_server
```

```bash
RUST_LOG=debug cargo run -p stock_client -- --tickers-file tickers.txt
```

Для показа только ошибок:

```bash
RUST_LOG=error cargo run -p stock_server
```

## Примеры вывода

Пример клиентского вывода:

```text
Connection established OK
Connected to stock server: 127.0.0.1:8080
Local UDP address: 127.0.0.1:54321
Subscribed to tickers: ["AAPL", "TSLA"]
UDP session established with server: 127.0.0.1:53421
QUOTE ticker=AAPL price=175.25 volume=1200 timestamp=1234567890
QUOTE ticker=AAPL price=176.10 volume=900 timestamp=1234567891
```

Пример серверного вывода:

```text
Server started on: 127.0.0.1:8080
Client connected: 127.0.0.1:54320
STREAM accepted: client_udp=127.0.0.1:54321, tickers=["AAPL", "TSLA"]
```

## Тесты

Запуск всех тестов:

```bash
cargo test
```

Основные зависимости:

- `clap` — CLI-аргументы;
- `serde` / `serde_json` — сериализация котировок;
- `crossbeam-channel` — каналы для подписчиков;
- `rand` — генерация случайных котировок;
- `ctrlc` — обработка `Ctrl+C`;
- `anyhow` — удобный error handling на уровне приложений;
- `thiserror` — типизированные ошибки библиотеки;
- `log` / `env_logger` — логирование.

## Основная цель проекта

Проект предназначен для закрепления тем:

- TCP и UDP в Rust;
- line-based TCP protocol;
- UDP datagrams;
- сериализация данных;
- многопоточность;
- `Arc`;
- `Mutex`;
- `AtomicBool`;
- каналы;
- graceful shutdown;
- разделение общей библиотеки, сервера и клиента в Cargo workspace.
