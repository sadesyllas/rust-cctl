pub mod message;
pub mod message_topic;
// pub mod try_downcast_ref;

use std::{collections::HashMap, sync::Arc};

use tokio::sync::{
    mpsc::{self, error::SendError, UnboundedReceiver, UnboundedSender},
    Mutex,
};

// use crate::pubsub::message_state::MessageState;
use crate::pubsub::message_topic::MessageTopic;

use self::message::Message;

pub type PubSubMessage = Arc<Mutex<UnboundedSender<(MessageTopic, Message)>>>;

pub fn start() -> (PubSubMessage, tokio::task::JoinHandle<()>) {
    let (tx, rx) = mpsc::unbounded_channel::<(MessageTopic, Message)>();

    let task = tokio::spawn(start_loop(rx));

    (Arc::new(Mutex::new(tx)), task)
}

async fn start_loop(mut rx: UnboundedReceiver<(MessageTopic, Message)>) {
    let mut registrations: HashMap<MessageTopic, Vec<Arc<UnboundedSender<Message>>>> =
        HashMap::new();

    loop {
        let message = rx.recv().await;

        println!("message...");

        if let Some((MessageTopic::Register, Message::Register { topic, sender })) = message {
            println!("message register...");
            handle_register(topic, sender, &mut registrations);
            println!("handle register ok");
        } else if let Some((topic, message)) = message {
            println!("message state...");
            broadcast(topic, message, &registrations);
            println!("message state ok...");
        } else {
            break;
        }
    }
}

fn handle_register(
    topic: MessageTopic,
    tx: Arc<UnboundedSender<Message>>,
    registrations: &mut HashMap<MessageTopic, Vec<Arc<UnboundedSender<Message>>>>,
) {
    registrations.entry(topic).or_insert_with(Vec::new).push(tx);
}

fn broadcast(
    topic: MessageTopic,
    message: Message,
    registrations: &HashMap<MessageTopic, Vec<Arc<UnboundedSender<Message>>>>,
) {
    if let Some(registrations) = registrations.get(&topic) {
        for tx in registrations {
            println!("sending");
            let message = message.clone();
            println!("clone ok");
            if let Err(error) = tx.send(message) {
                panic!("error: {}", error)
            } else {
                println!("sent");
            }
        }
    }
}
