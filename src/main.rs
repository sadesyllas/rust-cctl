#![feature(async_closure)]
#![feature(default_free_fn)]

use std::{net::SocketAddr, sync::Arc};

use config::Config;

mod applet_updater;
mod config;
mod device;
mod device_monitor;
mod pubsub;
mod util;
mod watchdog;
mod web;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_timer(tracing_subscriber::fmt::time::time())
        .init();

    let config = Arc::new(get_config());

    let (pubsub_tx, _) = pubsub::start();
    tokio::spawn(watchdog::audio::start(pubsub_tx.clone()));
    tokio::spawn(applet_updater::start(pubsub_tx.clone()));
    tokio::spawn(device_monitor::start(pubsub_tx.clone()));
    tokio::spawn(web::server::start(config.clone(), pubsub_tx.clone()))
        .await
        .unwrap();
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
