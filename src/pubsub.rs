pub mod message;
pub mod message_topic;

use std::{collections::HashMap, sync::Arc};

use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    Mutex,
};

use crate::pubsub::message_topic::MessageTopic;

use self::message::Message;

pub type PubSubMessage = (MessageTopic, Message);

pub fn start() -> (
    Arc<Mutex<UnboundedSender<PubSubMessage>>>,
    tokio::task::JoinHandle<()>,
) {
    let (tx, rx) = mpsc::unbounded_channel::<PubSubMessage>();

    let task = tokio::spawn(start_loop(rx));

    (Arc::new(Mutex::new(tx)), task)
}

async fn start_loop(mut rx: UnboundedReceiver<PubSubMessage>) {
    let mut registrations: HashMap<MessageTopic, Vec<Arc<UnboundedSender<Message>>>> =
        HashMap::new();

    loop {
        let message = rx.recv().await;

        if let Some((MessageTopic::Register, Message::Register { topic, tx })) = message {
            handle_register(topic, tx, &mut registrations);
        } else if let Some((topic, message)) = message {
            broadcast(topic, message, &registrations);
        }
    }
}

fn handle_register(
    topic: MessageTopic,
    sender: Arc<UnboundedSender<Message>>,
    registrations: &mut HashMap<MessageTopic, Vec<Arc<UnboundedSender<Message>>>>,
) {
    registrations
        .entry(topic)
        .or_insert_with(Vec::new)
        .push(sender);
}

fn broadcast(
    topic: MessageTopic,
    message: Message,
    registrations: &HashMap<MessageTopic, Vec<Arc<UnboundedSender<Message>>>>,
) {
    if let Some(registrations) = registrations.get(&topic) {
        for tx in registrations {
            tx.send(message.clone()).unwrap();
        }
    }
}
