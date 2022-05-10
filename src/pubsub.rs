pub mod message;
pub mod message_topic;

use std::{collections::HashMap, sync::Arc};

use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::pubsub::message_topic::MessageTopic;

use self::message::Message;

pub type PubSubMessage = (MessageTopic, Message);

pub fn start() -> (
    Arc<UnboundedSender<PubSubMessage>>,
    tokio::task::JoinHandle<()>,
) {
    let (tx, rx) = mpsc::unbounded_channel::<PubSubMessage>();

    let task = tokio::spawn(start_loop(rx));

    (Arc::new(tx), task)
}

async fn start_loop(mut rx: UnboundedReceiver<PubSubMessage>) {
    let mut registrations: HashMap<MessageTopic, Vec<Arc<UnboundedSender<Message>>>> =
        HashMap::new();

    loop {
        let message = rx.recv().await;

        match message {
            Some((MessageTopic::Register, Message::Register { topic, tx })) => {
                handle_register(topic, tx, &mut registrations);
            }
            Some((topic, message)) => {
                broadcast(topic, message, &mut registrations);
            }
            _ => (),
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
    registrations: &mut HashMap<MessageTopic, Vec<Arc<UnboundedSender<Message>>>>,
) {
    if let Some(registrations) = registrations.get_mut(&topic) {
        let mut indexes_to_remove = Vec::new();

        for (i, tx) in registrations.iter().enumerate() {
            if tx.is_closed() {
                indexes_to_remove.push(i);
                continue;
            }

            if tx.send(message.clone()).is_err() {
                indexes_to_remove.push(i);
            }
        }

        indexes_to_remove.into_iter().for_each(|i| {
            registrations.remove(i);
        });
    }
}
