use std::{sync::Arc, time::Duration};

use tokio::{
    io,
    sync::{mpsc::UnboundedSender, Mutex},
    time::sleep,
};
use tracing::log::debug;

use crate::{
    device::audio,
    pubsub::{message::Message, message_state::MessageState, message_topic::MessageTopic},
};

pub async fn start(pubsub_tx: Arc<Mutex<UnboundedSender<Message>>>) -> io::Result<()> {
    loop {
        debug!("Fetching the state of audio devices in audio monitor");

        let (cards, sources, sinks) = audio::fetch_devices().await;

        pubsub_tx
            .lock()
            .await
            .send(Arc::new((
                MessageTopic::AudioState,
                Arc::new(MessageState::new(
                    Arc::new(cards),
                    Arc::new(sources),
                    Arc::new(sinks),
                )),
            )))
            .unwrap();

        sleep(Duration::from_secs(15)).await;
    }
}
