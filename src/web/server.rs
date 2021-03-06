use std::sync::Arc;

use axum::{
    extract::{
        ws::{self, WebSocket},
        WebSocketUpgrade,
    },
    handler::options,
    response::{Headers, IntoResponse},
    Json, Router,
};
use serde::Deserialize;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    Mutex,
};
use tracing::{info, instrument, log::debug};

use crate::{
    config::Config,
    device::{audio, card_device_type::CardDeviceType, card_profile::CardProfile},
    pubsub::{
        message::{Message, MessagePayload},
        message_register::MessageRegister,
        message_state::MessageState,
        message_topic::MessageTopic,
        try_downcast_ref::try_downcast_ref,
    },
};

#[instrument]
pub async fn start(config: Arc<Config>, pubsub_tx: Arc<Mutex<UnboundedSender<Message>>>) {
    let (tx, rx) = mpsc::unbounded_channel::<Message>();
    let tx = Arc::new(tx);
    let rx = Arc::new(Mutex::new(rx));

    pubsub_tx
        .lock()
        .await
        .send(Arc::new(MessageRegister::new(
            MessageTopic::AudioState,
            tx.clone(),
        )))
        .unwrap();

    let app = Router::new()
        .route(
            "/audio",
            options(async move || wrap_cors(())).get({
                let pubsub_tx = pubsub_tx.clone();

                async move || wrap_cors(audio_handler(pubsub_tx).await)
            }),
        )
        .route(
            "/audio/volume",
            options(async move || wrap_cors(())).post({
                let pubsub_tx = pubsub_tx.clone();

                async move |request| {
                    handle_volume_request(request, pubsub_tx).await;
                    wrap_cors(())
                }
            }),
        )
        .route(
            "/audio/mute",
            options(async move || wrap_cors(())).post({
                let pubsub_tx = pubsub_tx.clone();

                async move |request| {
                    handle_mute_request(request, pubsub_tx).await;
                    wrap_cors(())
                }
            }),
        )
        .route(
            "/audio/default",
            options(async move || wrap_cors(())).post({
                let pubsub_tx = pubsub_tx.clone();

                async move |request| {
                    handle_default_request(request, pubsub_tx).await;
                    wrap_cors(())
                }
            }),
        )
        .route(
            "/audio/profile",
            options(async move || wrap_cors(())).post({
                let pubsub_tx = pubsub_tx.clone();

                async move |request| {
                    handle_profile_request(request, pubsub_tx).await;
                    wrap_cors(())
                }
            }),
        )
        .route(
            "/audio/ws",
            options(async move || wrap_cors(())).get({
                let rx = rx.clone();

                async move |ws| ws_handle_upgrade_messages(ws, rx).await
            }),
        );

    info!("Listening on http://{}", config.server_addr);

    axum::Server::bind(&config.server_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn wrap_cors(response: impl IntoResponse) -> impl IntoResponse {
    (
        Headers([
            ("Access-Control-Allow-Origin", "*"),
            ("Access-Control-Allow-Headers", "Content-Type"),
        ]),
        response,
    )
}

async fn audio_handler(
    pubsub_tx: Arc<Mutex<UnboundedSender<Arc<dyn MessagePayload + Send + Sync>>>>,
) -> impl IntoResponse {
    debug!("Fetching the state of audio devices in web server");

    let (cards, sources, sinks) = audio::fetch_devices().await;
    let message_state = MessageState::new(Arc::new(cards), Arc::new(sources), Arc::new(sinks));

    pubsub_tx
        .lock()
        .await
        .send(Arc::new((
            MessageTopic::AudioState,
            Arc::new(message_state.clone()),
        )))
        .unwrap();

    Json(message_state)
}

async fn ws_handle_upgrade_messages(
    ws: WebSocketUpgrade,
    rx: Arc<Mutex<UnboundedReceiver<Arc<dyn MessagePayload + Send + Sync>>>>,
) -> impl IntoResponse {
    ws.on_upgrade(async move |socket| ws_handle_messages_socket(socket, rx).await)
}

async fn ws_handle_messages_socket(
    mut socket: WebSocket,
    rx: Arc<Mutex<UnboundedReceiver<Arc<dyn MessagePayload + Send + Sync>>>>,
) {
    loop {
        if let Some(message) = rx.lock().await.recv().await {
            let message = try_downcast_ref!(message, MessageState).cloned();

            if let Some(message) = message {
                debug!("Sending message down the websocket");

                if socket
                    .send(ws::Message::Text(serde_json::to_string(&message).unwrap()))
                    .await
                    .is_err()
                {
                    return;
                }
            }
        }
    }
}

#[derive(Deserialize, Debug)]
struct VolumeRequest {
    #[serde(rename(deserialize = "type"))]
    _type: CardDeviceType,
    index: u64,
    volume: f64,
}

async fn handle_volume_request(
    Json(VolumeRequest {
        _type,
        index,
        volume,
    }): Json<VolumeRequest>,
    pubsub_tx: Arc<Mutex<UnboundedSender<Arc<dyn MessagePayload + Send + Sync>>>>,
) {
    debug!(
        "Setting the volume of {} device index {} to {}",
        _type, index, volume
    );

    audio::set_volume(_type, index, volume).await.unwrap();

    audio_handler(pubsub_tx.clone()).await;
}

#[derive(Deserialize, Debug)]
struct MuteRequest {
    #[serde(rename(deserialize = "type"))]
    _type: CardDeviceType,
    index: u64,
    mute: bool,
}

async fn handle_mute_request(
    Json(MuteRequest { _type, index, mute }): Json<MuteRequest>,
    pubsub_tx: Arc<Mutex<UnboundedSender<Arc<dyn MessagePayload + Send + Sync>>>>,
) {
    debug!(
        "Setting the mute state of {} device index {} to {}",
        _type, index, mute
    );

    audio::toggle_mute(_type, index, mute).await.unwrap();

    audio_handler(pubsub_tx.clone()).await;
}

#[derive(Deserialize, Debug)]
struct DefaultRequest {
    #[serde(rename(deserialize = "type"))]
    _type: CardDeviceType,
    index: u64,
    name: String,
}

async fn handle_default_request(
    Json(DefaultRequest { _type, index, name }): Json<DefaultRequest>,
    pubsub_tx: Arc<Mutex<UnboundedSender<Arc<dyn MessagePayload + Send + Sync>>>>,
) {
    debug!(
        "Setting the default {} device to index {} (name = {})",
        _type, index, name
    );

    audio::set_default_card_device(_type, index).await.unwrap();

    debug!(
        "Moving audio clients to {} device index {} (name = {})",
        _type, index, name
    );

    audio::move_audio_clients(_type, index, &name)
        .await
        .unwrap();

    audio_handler(pubsub_tx.clone()).await;
}

#[derive(Deserialize, Debug)]
struct ProfileRequest {
    index: u64,
    profile: CardProfile,
}

async fn handle_profile_request(
    Json(ProfileRequest { index, profile }): Json<ProfileRequest>,
    pubsub_tx: Arc<Mutex<UnboundedSender<Arc<dyn MessagePayload + Send + Sync>>>>,
) {
    debug!(
        "Setting the default bluetooth card index {} profile to {}",
        index, profile
    );

    audio::set_card_profile(index, profile).await.unwrap();

    audio_handler(pubsub_tx.clone()).await;
}
