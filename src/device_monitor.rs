use std::{sync::Arc, time::Duration};

use tokio::{
    io,
    sync::{mpsc::UnboundedSender, Mutex},
    time::sleep,
};
use tracing::log::debug;

use crate::{
    device::audio,
    pubsub::{message::Message, message_topic::MessageTopic, PubSubMessage},
};

pub async fn start(pubsub_tx: Arc<Mutex<UnboundedSender<PubSubMessage>>>) -> io::Result<()> {
    loop {
        debug!("Fetching the state of audio devices in audio monitor");

        let (cards, sources, sinks) = audio::fetch_devices().await;

        pubsub_tx
            .lock()
            .await
            .send((
                MessageTopic::AudioState,
                Message::new_audio_state(Arc::new(cards), Arc::new(sources), Arc::new(sinks)),
            ))
            .unwrap();

        sleep(Duration::from_secs(15)).await;
    }
}
