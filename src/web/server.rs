use std::sync::Arc;

use axum::{
    extract::{
        ws::{self, WebSocket},
        WebSocketUpgrade,
    },
    response::{AppendHeaders, IntoResponse},
    routing::options,
    Json, Router,
};
use serde::Deserialize;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    Mutex,
};
use tracing::{instrument, log::info, span};

use crate::{
    config::Config,
    device::{audio, card_device_type::CardDeviceType, card_profile::CardProfile},
    pubsub::{message::Message, message_topic::MessageTopic, PubSubMessage},
};

#[instrument]
pub async fn start(config: Arc<Config>, pubsub_tx: Arc<Mutex<UnboundedSender<PubSubMessage>>>) {
    let (tx, rx) = mpsc::unbounded_channel::<Message>();
    let tx = Arc::new(tx);
    let rx = Arc::new(Mutex::new(rx));

    pubsub_tx
        .lock()
        .await
        .send((
            MessageTopic::Register,
            Message::new_register(MessageTopic::AudioState, tx.clone()),
        ))
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
        AppendHeaders([
            ("Access-Control-Allow-Origin", "*"),
            ("Access-Control-Allow-Headers", "Content-Type"),
        ]),
        response,
    )
}

#[instrument(level = "info")]
async fn audio_handler(pubsub_tx: Arc<Mutex<UnboundedSender<PubSubMessage>>>) -> impl IntoResponse {
    info!("Fetching the state of audio devices in web server");

    let (cards, sources, sinks) = audio::fetch_devices().await;
    let message_state =
        Message::new_audio_state(Arc::new(cards), Arc::new(sources), Arc::new(sinks));

    pubsub_tx
        .lock()
        .await
        .send((MessageTopic::AudioState, message_state.clone()))
        .unwrap();

    Json(message_state)
}

async fn ws_handle_upgrade_messages(
    ws: WebSocketUpgrade,
    rx: Arc<Mutex<UnboundedReceiver<Message>>>,
) -> impl IntoResponse {
    ws.on_upgrade(async move |socket| ws_handle_messages_socket(socket, rx).await)
}

async fn ws_handle_messages_socket(
    mut socket: WebSocket,
    rx: Arc<Mutex<UnboundedReceiver<Message>>>,
) {
    loop {
        if let Some(message @ Message::AudioState { .. }) = rx.lock().await.recv().await {
            let span = span!(tracing::Level::INFO, "web socket message");
            let _enter = span.enter();
            info!("Sending message down the websocket");
            drop(_enter);

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

#[derive(Deserialize, Debug)]
struct VolumeRequest {
    #[serde(rename(deserialize = "type"))]
    _type: CardDeviceType,
    index: u64,
    volume: f64,
}

#[instrument]
async fn handle_volume_request(
    Json(VolumeRequest {
        _type,
        index,
        volume,
    }): Json<VolumeRequest>,
    pubsub_tx: Arc<Mutex<UnboundedSender<PubSubMessage>>>,
) {
    info!(
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

#[instrument]
async fn handle_mute_request(
    Json(MuteRequest { _type, index, mute }): Json<MuteRequest>,
    pubsub_tx: Arc<Mutex<UnboundedSender<PubSubMessage>>>,
) {
    info!(
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

#[instrument]
async fn handle_default_request(
    Json(DefaultRequest { _type, index, name }): Json<DefaultRequest>,
    pubsub_tx: Arc<Mutex<UnboundedSender<PubSubMessage>>>,
) {
    info!(
        "Setting the default {} device to index {} (name = {})",
        _type, index, name
    );

    audio::set_default_card_device(_type, index).await.unwrap();

    info!(
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

#[instrument]
async fn handle_profile_request(
    Json(ProfileRequest { index, profile }): Json<ProfileRequest>,
    pubsub_tx: Arc<Mutex<UnboundedSender<PubSubMessage>>>,
) {
    info!(
        "Setting the default bluetooth card index {} profile to {}",
        index, profile
    );

    audio::set_card_profile(index, profile).await.unwrap();

    audio_handler(pubsub_tx.clone()).await;
}
