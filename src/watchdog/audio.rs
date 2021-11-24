use std::sync::Arc;

use tokio::sync::{
    mpsc::{self, UnboundedSender},
    Mutex,
};

use crate::{
    device::{audio, card_device::CardDevice, card_device_type::CardDeviceType},
    pubsub::{
        message::Message, message_register::MessageRegister, message_state::MessageState,
        message_topic::MessageTopic, try_downcast_ref::try_downcast_ref,
    },
};

pub async fn start(pubsub_tx: Arc<Mutex<UnboundedSender<Message>>>) {
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    pubsub_tx
        .lock()
        .await
        .send(Arc::new(MessageRegister::new(
            MessageTopic::AudioState,
            Arc::new(tx),
        )))
        .unwrap();

    let mut default_source: Option<CardDevice> = None;
    let mut default_sink: Option<CardDevice> = None;

    loop {
        if let Some(message) = rx.recv().await {
            if let Some(state) = try_downcast_ref!(message, MessageState) {
                default_source = state
                    .sources()
                    .iter()
                    .find(|source| source.is_default)
                    .unwrap()
                    .clone()
                    .into();

                default_sink = state
                    .sinks()
                    .iter()
                    .find(|sink| sink.is_default)
                    .unwrap()
                    .clone()
                    .into();
            }
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
