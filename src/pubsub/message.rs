use std::{any::Any, fmt::Debug, sync::Arc};

pub type Message = Arc<dyn MessagePayload + Send + Sync>;

pub trait MessagePayload: Debug {
    fn as_any(&self) -> &dyn Any;
}
