use std::{io::Read, path::PathBuf, process::Stdio, sync::Arc};

use glob::glob;
use log::error;
use tokio::{
    process::Command,
    sync::mpsc::{self, UnboundedSender},
};
use tracing::span;

use crate::{
    device::card_device::CardDevice,
    pubsub::{message::Message, message_topic::MessageTopic, PubSubMessage},
};

pub async fn start(pubsub_tx: Arc<UnboundedSender<PubSubMessage>>) {
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    pubsub_tx
        .send((
            MessageTopic::Register,
            Message::new_register(MessageTopic::AudioState, Arc::new(tx)),
        ))
        .unwrap();

    let mut default_source: Option<CardDevice> = None;

    loop {
        if let Some(Message::AudioState { ref sources, .. }) = rx.recv().await {
            let span = span!(tracing::Level::INFO, "applet_updater.start loop");
            let _enter = span.enter();
            let mut maybe_app_path: Option<String> = None;
            let mut volume_icon: Option<String> = None;
            let mut volume: Option<f64> = None;

            let new_default_source = sources.iter().find(|s| s.is_default).unwrap();

            let update_applet = if let Some(ref default_source_) = default_source {
                new_default_source.index != default_source_.index
                    || new_default_source.volume != default_source_.volume
                    || new_default_source.is_muted != default_source_.is_muted
            } else {
                true
            };

            if update_applet {
                default_source.replace(new_default_source.clone());

                volume_icon.replace(if new_default_source.is_muted {
                    "microphone-sensitivity-muted-symbolic".to_owned()
                } else if new_default_source.volume < 25.0 {
                    "microphone-sensitivity-low-symbolic".to_owned()
                } else if new_default_source.volume >= 25.0 || new_default_source.volume <= 75.0 {
                    "microphone-sensitivity-medium-symbolic".to_owned()
                } else {
                    "microphone-sensitivity-high-symbolic".to_owned()
                });

                volume.replace(new_default_source.volume);

                let app_file_path_glob: PathBuf = [
                    std::env::var("HOME").unwrap(),
                    ".config".to_owned(),
                    "xfce4".to_owned(),
                    "panel".to_owned(),
                    "**".to_owned(),
                    "*.desktop".to_owned(),
                ]
                .iter()
                .collect();

                let maybe_app_path_ =
                    glob(app_file_path_glob.to_str().unwrap())
                        .unwrap()
                        .find(|path| {
                            let mut file = std::fs::File::open(path.as_ref().unwrap()).unwrap();
                            let mut buf = String::new();
                            file.read_to_string(&mut buf).unwrap();
                            buf.contains("Name=toggle_microphone")
                        });

                if let Some(path) = maybe_app_path_ {
                    maybe_app_path.replace(path.unwrap().to_str().unwrap().to_owned());
                }
            }

            let mut tasks = Vec::new();

            if let Some(path) = maybe_app_path {
                let volume_icon = volume_icon.as_ref().unwrap().to_owned();

                tasks.push(tokio::spawn(async move {
                    let mut command = Command::new("sed")
                        .args(&["-i", &format!("s/Icon=.*/Icon={}/", &volume_icon), &path])
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                        .unwrap();

                    let exit_status = command.wait().await.unwrap();

                    if !exit_status.success() {
                        error!("Could not set the applet icon to {}", &volume_icon);
                    }
                }));
            }

            if let (Some(volume_icon), Some(volume)) = (volume_icon, volume) {
                let volume_icon = volume_icon.to_owned();

                tasks.push(tokio::spawn(async move {
                    let mut command = Command::new("notify-send")
                        .args(&["-t", "1", "-i", &volume_icon, &format!("{}%", volume), ""])
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                        .unwrap();

                    let exit_status = command.wait().await.unwrap();

                    if !exit_status.success() {
                        error!("Could not set the applet icon to {}", &volume_icon);
                    }
                }));
            }

            for task in tasks {
                task.await.unwrap();
            }
        }
    }
}
