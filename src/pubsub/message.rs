use std::{any::Any, fmt::Debug, sync::Arc};

use tokio::sync::mpsc::UnboundedSender;

use crate::pwcli::Object;

use super::message_topic::MessageTopic;

#[derive(Clone, Debug)]
pub enum Message {
    Register {
        topic: MessageTopic,
        sender: Arc<UnboundedSender<Message>>,
    },

    AudioState(Arc<Vec<Object>>),
}

pub trait MessagePayload: Debug {
    fn as_any(&self) -> &dyn Any;
}
