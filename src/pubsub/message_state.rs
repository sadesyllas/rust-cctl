use std::{default::default, sync::Arc, time::SystemTime};

use serde::{ser::SerializeStruct, Serialize};

use crate::device::{card::Card, card_device::CardDevice};

use super::{message::MessagePayload, message_topic::MessageTopic};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct MessageState {
    cards: Arc<Vec<Card>>,
    sources: Arc<Vec<CardDevice>>,
    sinks: Arc<Vec<CardDevice>>,
    timestamp: u128,
}

impl MessageState {
    pub fn new(
        cards: Arc<Vec<Card>>,
        sources: Arc<Vec<CardDevice>>,
        sinks: Arc<Vec<CardDevice>>,
    ) -> Self {
        Self {
            cards,
            sources,
            sinks,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        }
    }

    pub fn sources(&self) -> Arc<Vec<CardDevice>> {
        self.sources.clone()
    }

    pub fn sinks(&self) -> Arc<Vec<CardDevice>> {
        self.sinks.clone()
    }
}

impl Default for MessageState {
    fn default() -> Self {
        Self {
            cards: default(),
            sources: default(),
            sinks: default(),
            timestamp: default(),
        }
    }
}

impl Serialize for MessageState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer
            .serialize_struct(std::any::type_name::<MessageState>(), 3)
            .unwrap();
        s.serialize_field("cards", self.cards.as_ref()).unwrap();
        s.serialize_field("sources", self.sources.as_ref()).unwrap();
        s.serialize_field("sinks", self.sinks.as_ref()).unwrap();
        s.serialize_field("timestamp", &self.timestamp).unwrap();
        s.end()
    }
}

impl MessagePayload for MessageState {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl MessagePayload for (MessageTopic, Arc<MessageState>) {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
