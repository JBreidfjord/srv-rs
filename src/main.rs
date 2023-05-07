use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    routing::get,
    Json, Router,
};
use clap::Parser;
use serde::Serialize;
use tokio::sync::oneshot;

use srv_rs::{
    config::Config, emitter::Emitter, error::Error, parser::parse_message, publisher::Publisher,
    sensors::fetch_sensors,
};
use srv_rs::{error::Result, AppState};

const BATCH_SIZE: usize = 25;
const SOCKET_TIMEOUT: u64 = 60;

#[derive(Serialize)]
struct UdpAddr {
    address: String,
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let config = Config::parse();
    let app_state = AppState {
        config: Arc::new(config),
        http: reqwest::Client::new(),
    };

    let router = Router::new()
        .route("/:thing_id/start", get(start_thing))
        .with_state(app_state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], app_state.config.server_port));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

async fn start_thing(
    Path(thing_id): Path<String>,
    State(app): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UdpAddr>> {
    let api_key = if let Some(key) = headers.get("apiKey") {
        key.to_str()
            .context("failed to parse apiKey header")?
            .to_string()
    } else {
        return Err(Error::Unauthorized);
    };

    let mut address = UdpAddr {
        address: app.config.host.clone(),
        port: 0,
    };

    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move {
        if let Err(e) = start_session(thing_id.clone(), app, api_key.clone(), tx).await {
            eprintln!("failed to start session: {e}");
        };
    });

    // Wait for the session task to start and get the port
    address.port = rx.await.context("failed to get port from session task")?;
    println!("got port from session task: {}", address.port);

    Ok(Json(address))
}

async fn start_session(
    thing_id: String,
    app: AppState,
    api_key: String,
    tx: oneshot::Sender<u16>,
) -> Result<()> {
    // Open a UDP socket
    let addr = SocketAddr::from(([0, 0, 0, 0], 0));
    let socket = tokio::net::UdpSocket::bind(addr)
        .await
        .context("failed to bind UDP socket")?;

    // Send the port back to the client
    let port = socket
        .local_addr()
        .context("failed to get socket address")?
        .port();
    if let Err(e) = tx.send(port) {
        eprintln!("failed to send port back to client: {e}");
    };

    let sensors = fetch_sensors(&thing_id, app.clone(), &api_key).await?;

    let mut snapshot_queue = Vec::with_capacity(BATCH_SIZE);
    let mut prev_timestamp = 0;

    let mut emitter = Emitter::default();
    if let Err(e) = emitter.start(app.clone(), &thing_id, &api_key).await {
        eprintln!("failed to start emitter: {e}");
    };

    let mut publisher = Publisher::connect(app, thing_id)?;

    // Receive messages from the socket
    loop {
        let mut buf = [0; 4096];
        let timeout = tokio::time::Duration::from_secs(SOCKET_TIMEOUT);
        let len = match tokio::time::timeout(timeout, socket.recv(&mut buf)).await {
            Ok(res) => res.context("failed to receive from socket")?,
            Err(_) => {
                eprintln!("socket time out, disconnecting");
                break;
            }
        };
        let buf = &buf[..len];
        println!("received {len} bytes");

        publisher.publish_connection()?;

        let (timestamp, snapshot) = match parse_message(buf, &sensors) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("failed to parse message: {e}");
                break;
            }
        };

        // We only want to emit messages that are newer than the previous one
        if prev_timestamp >= timestamp {
            continue;
        }

        prev_timestamp = timestamp;

        emitter.emit(&snapshot).await?;

        snapshot_queue.push(snapshot);
        if snapshot_queue.len() >= BATCH_SIZE {
            println!("publishing queue");
            publisher.push_snapshots(&snapshot_queue)?;
            snapshot_queue.clear();
        }
    }

    if !snapshot_queue.is_empty() {
        println!("publishing queue");
        publisher.push_snapshots(&snapshot_queue)?;
    }
    publisher.publish_disconnection()?;
    Ok(())
}
