use crate::core::message::Message;

#[derive(Debug, Clone)]
pub enum EngineEvent {
    PeerConnected {
        a: String,
        b: String,
    },
    MessageReceived {
        msg: Message,
    },
    MessageForwarded {
        to: String,
        msg: Message,
    },
}
