#![feature(async_closure)]
#![feature(default_free_fn)]

use std::{net::SocketAddr, process::Stdio, sync::Arc};

use config::Config;
use tokio::{io::AsyncReadExt, process::Command, sync::mpsc};
use tracing::error;
use tracing_subscriber::prelude::*;

use crate::{
    pubsub::{message::Message, message_topic::MessageTopic},
    pwcli::Object,
};

// mod applet_updater;
mod config;
// mod device;
// mod device_monitor;
mod pubsub;
// mod util;
// mod watchdog;
// mod web;
mod pwcli;

#[tokio::main]
async fn main() {
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_agent_endpoint("localhost:6831")
        .with_service_name("cctl")
        .install_simple()
        .unwrap();

    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry().with(opentelemetry).init();

    // let config = Arc::new(get_config());

    let (pubsub_tx, j) = pubsub::start();
    // tokio::spawn(watchdog::audio::start(pubsub_tx.clone()));
    // tokio::spawn(applet_updater::start(pubsub_tx.clone()));
    // tokio::spawn(device_monitor::start(pubsub_tx.clone()));
    // tokio::spawn(web::server::start(config.clone(), pubsub_tx.clone()))
    //     .await
    //     .unwrap();

    let mut cnt = 0;
    let (tx, mut rx) = mpsc::unbounded_channel();
    let tx = Arc::new(tx);

    loop {
        if cnt == 2 {
            break;
        }

        pubsub_tx
            .clone()
            .lock()
            .await
            .send((
                MessageTopic::Register,
                Message::Register {
                    topic: MessageTopic::AudioState,
                    sender: tx.clone(),
                },
            ))
            .unwrap();

        let parsed_objects = pwcli::get_objects().await.unwrap();

        // println!(
        //     "les objects:\n{}\ntotal = {}",
        //     serde_json::to_string_pretty(&parsed_objects).unwrap(),
        //     parsed_objects.len()
        // );

        pubsub_tx
            .clone()
            .lock()
            .await
            .send((
                MessageTopic::AudioState,
                Message::AudioState(Arc::new(parsed_objects)),
            ))
            .unwrap();

        println!("waiting...");
        let r = rx.recv().await;
        let objects = if let Some(Message::AudioState(state)) = r {
            state
        } else {
            println!("error received: {:?}", r);
            unreachable!();
        };

        println!("received...");

        // println!("objects: {:?}", objects);

        cnt += 1;

        println!("cnt: {}", cnt);
    }

    println!("bye");

    // j.await.unwrap();

    // for x in &parsed_objects {
    //     println!(
    //         "count={}, parsed id: {}, default: {}, name: {}, type: {}, device id: {:?}, profile #: {}, active profile: {:?}, routes: {}",
    //         parsed_objects.len(),
    //         x.id,
    //         x.default,
    //         x.name,
    //         x.r#type,
    //         x.device_id,
    //         x.profiles.len(),
    //         x.profile,
    //         serde_json::to_string(&x.routes).unwrap()
    //     );
    // }
}

fn get_config() -> Config {
    let cli_config = clap::App::new("cctl")
        .arg(
            clap::Arg::with_name("host")
                .short("c")
                .long("host")
                .takes_value(true)
                .default_value("0.0.0.0")
                .value_name("HOST")
                .help("The host to listen as"),
        )
        .arg(
            clap::Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true)
                .default_value("3003")
                .value_name("PORT")
                .help("The port to listen to"),
        )
        .get_matches();

    let host: [u8; 4] = cli_config
        .value_of("host")
        .unwrap()
        .split('.')
        .map(|octet| octet.parse::<u8>().unwrap())
        .fold(([0u8; 4], 0), |(mut acc, i), val| {
            acc[i] = val;

            (acc, i + 1)
        })
        .0;

    let port: u16 = cli_config.value_of("port").unwrap().parse().unwrap();

    let server_addr = SocketAddr::from((host, port));

    Config { server_addr }
}
