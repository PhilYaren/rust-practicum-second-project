use crossbeam_channel::{Receiver, Sender, unbounded};
use log::error;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct ServerState<T: Clone + Send + 'static> {
    subscribers: Arc<Mutex<Vec<Sender<T>>>>,
}

impl<T: Clone + Send + 'static> ServerState<T> {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn subscribe(&self) -> Receiver<T> {
        let (tx, rx) = unbounded();

        let mut subscribers = match self.subscribers.lock() {
            Ok(subscribers) => subscribers,
            Err(poisoned) => {
                error!("Mutex was poisoned due to: {}", &poisoned);

                poisoned.into_inner()
            }
        };
        subscribers.push(tx);

        rx
    }

    pub fn broadcast(&self, value: T) {
        let mut subscribers = match self.subscribers.lock() {
            Ok(subscribers) => subscribers,
            Err(poisoned) => {
                error!("Mutex was poisoned due to: {}", poisoned);

                poisoned.into_inner()
            }
        };

        // retain оставит только активные каналы, остальные дропнутся
        subscribers.retain(|tx| tx.send(value.clone()).is_ok());
    }
}
