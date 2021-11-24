pub mod message;
pub mod message_register;
pub mod message_state;
pub mod message_topic;
pub mod try_downcast_ref;

use std::{collections::HashMap, sync::Arc};

use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    Mutex,
};

use crate::pubsub::message_state::MessageState;
use crate::pubsub::message_topic::MessageTopic;

use self::{
    message::Message, message_register::MessageRegister, try_downcast_ref::try_downcast_ref,
};

pub fn start() -> (
    Arc<Mutex<UnboundedSender<Message>>>,
    tokio::task::JoinHandle<()>,
) {
    let (tx, rx) = mpsc::unbounded_channel::<Message>();

    let task = tokio::spawn(start_loop(rx));

    (Arc::new(Mutex::new(tx)), task)
}

async fn start_loop(mut rx: UnboundedReceiver<Message>) {
    let mut registrations: HashMap<MessageTopic, Vec<Arc<UnboundedSender<Message>>>> =
        HashMap::new();

    loop {
        let message = rx.recv().await;

        if let Some(message) = message {
            if let Some(message) = try_downcast_ref!(message, MessageRegister) {
                handle_register(message, &mut registrations);
            } else if try_downcast_ref!(message, (MessageTopic, Arc<MessageState>)).is_some() {
                if let Some((topic, message)) =
                    try_downcast_ref!(message, (MessageTopic, Arc<MessageState>)).cloned()
                {
                    broadcast(topic, message, &registrations);
                }
            }
        }
    }
}

fn handle_register(
    message: &MessageRegister,
    registrations: &mut HashMap<MessageTopic, Vec<Arc<UnboundedSender<Message>>>>,
) {
    registrations
        .entry(message.topic())
        .or_insert_with(Vec::new)
        .push(message.tx());
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
