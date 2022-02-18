use std::{any::Any, sync::Arc};

use tokio::sync::mpsc::UnboundedSender;

use super::message_topic::MessageTopic;

#[derive(Debug)]
pub struct MessageRegister {
    topic: MessageTopic,
    tx: Arc<UnboundedSender<Message>>,
}

impl MessageRegister {
    pub fn new(topic: MessageTopic, tx: Arc<UnboundedSender<Message>>) -> Self {
        Self { topic, tx }
    }
    pub fn topic(&self) -> MessageTopic {
        self.topic
    }

    pub fn tx(&self) -> Arc<UnboundedSender<Message>> {
        self.tx.clone()
    }
}
