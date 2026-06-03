use tokio::sync::broadcast;
use crate::core::events::EngineEvent;

#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<EngineEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { sender: tx }
    }

    pub fn emit(&self, event: EngineEvent) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<EngineEvent> {
        self.sender.subscribe()
    }
}
