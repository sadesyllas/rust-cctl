use std::{sync::Arc, time::SystemTime};

use serde::{ser::SerializeStruct, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::device::{card::Card, card_device::CardDevice};

use super::message_topic::MessageTopic;

#[derive(Clone, Debug)]
pub enum Message {
    Register {
        topic: MessageTopic,
        tx: Arc<UnboundedSender<Message>>,
    },

    AudioState {
        cards: Arc<Vec<Card>>,
        sources: Arc<Vec<CardDevice>>,
        sinks: Arc<Vec<CardDevice>>,
        timestamp: u128,
    },
}

impl Message {
    pub fn new_register(topic: MessageTopic, tx: Arc<UnboundedSender<Message>>) -> Self {
        Message::Register { topic, tx }
    }

    pub fn new_audio_state(
        cards: Arc<Vec<Card>>,
        sources: Arc<Vec<CardDevice>>,
        sinks: Arc<Vec<CardDevice>>,
    ) -> Self {
        Message::AudioState {
            cards,
            sources,
            sinks,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        }
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Message::AudioState {
            ref cards,
            ref sources,
            ref sinks,
            ref timestamp,
        } = self
        {
            let mut s = serializer
                .serialize_struct(std::any::type_name::<Message>(), 3)
                .unwrap();
            s.serialize_field("cards", cards.as_ref()).unwrap();
            s.serialize_field("sources", sources.as_ref()).unwrap();
            s.serialize_field("sinks", sinks.as_ref()).unwrap();
            s.serialize_field("timestamp", &timestamp).unwrap();
            s.end()
        } else {
            unreachable!()
        }
    }
}
