use std::sync::Arc;

use tokio::sync::{
    mpsc::{self, UnboundedSender},
    Mutex,
};

use crate::{
    device::{audio, card_device::CardDevice, card_device_type::CardDeviceType},
    pubsub::{message::Message, message_topic::MessageTopic, PubSubMessage},
};

pub async fn start(pubsub_tx: Arc<Mutex<UnboundedSender<PubSubMessage>>>) {
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    pubsub_tx
        .lock()
        .await
        .send((
            MessageTopic::Register,
            Message::new_register(MessageTopic::AudioState, Arc::new(tx)),
        ))
        .unwrap();

    let mut default_source: Option<CardDevice> = None;
    let mut default_sink: Option<CardDevice> = None;

    loop {
        if let Some(Message::AudioState {
            ref sources,
            ref sinks,
            ..
        }) = rx.recv().await
        {
            default_source = sources
                .iter()
                .find(|source| source.is_default)
                .unwrap()
                .clone()
                .into();

            default_sink = sinks
                .iter()
                .find(|sink| sink.is_default)
                .unwrap()
                .clone()
                .into();
        }

        if let Some(ref source) = default_source {
            audio::move_audio_clients(CardDeviceType::Source, source.index, &source.name)
                .await
                .unwrap_or_default();
        }

        if let Some(ref sink) = default_sink {
            audio::move_audio_clients(CardDeviceType::Sink, sink.index, &sink.name)
                .await
                .unwrap_or_default();
        }
    }
}
