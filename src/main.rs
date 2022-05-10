#![feature(async_closure)]
#![feature(default_free_fn)]

use std::{net::SocketAddr, str::FromStr, sync::Arc};

use config::Config;
use log::info;
use metrics_exporter_prometheus::PrometheusBuilder;
use tokio::net::UdpSocket;
use tracing_subscriber::prelude::*;

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
    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()
        .expect("Failed to setup logging");

    let jaeger_available = {
        let sock = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        sock.connect("localhost:6831").await.unwrap();
        sock.send(&[0]).await.unwrap();
        sock.take_error().unwrap().is_none()
    };

    if jaeger_available {
        info!("Jaeger is available");

        let tracer = opentelemetry_jaeger::new_pipeline()
            .with_agent_endpoint("localhost:6831")
            .with_service_name("rust-cctl")
            .install_simple()
            .unwrap();

        let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

        tracing_subscriber::registry().with(opentelemetry).init();
    } else {
        info!("Jaeger is not available");
    }

    PrometheusBuilder::new()
        .with_http_listener(SocketAddr::from_str("0.0.0.0:9009").unwrap())
        .install()
        .expect("Failed to install prometheus recorder/exporter");

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
    let cli_config = clap::Command::new("cctl")
        .arg(
            clap::Arg::new("host")
                .short('c')
                .long("host")
                .takes_value(true)
                .default_value("0.0.0.0")
                .value_name("HOST")
                .help("The host to listen as"),
        )
        .arg(
            clap::Arg::new("port")
                .short('p')
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
