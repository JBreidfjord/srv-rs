use std::collections::HashMap;

use anyhow::Context;
use futures_util::FutureExt;
use rust_socketio::asynchronous as socketio;
use serde_json::json;

use crate::{sensors::SensorValue, AppState};

#[derive(Default)]
pub struct Emitter {
    socket: Option<socketio::Client>,
}

impl Emitter {
    pub async fn start(
        &mut self,
        app: AppState,
        thing_id: &str,
        api_key: &str,
    ) -> anyhow::Result<()> {
        println!("starting emitter");
        let socket = socketio::ClientBuilder::new(app.config.gateway_url.clone())
            .opening_header("key", api_key)
            .on("room created", |payload, _| {
                async move { println!("room created, payload: {payload:?}") }.boxed()
            })
            .on("room creation error", |payload, _| {
                async move { println!("room creation error, payload: {payload:?}") }.boxed()
            })
            .on("disconnect", |payload, _| {
                async move { println!("disconnected, payload: {payload:?}") }.boxed()
            })
            .connect()
            .await
            .context("failed to connect to gateway socket")?;

        println!("connected to gateway socket");

        // Sleep for a bit to make sure the socket is connected before we try to create a room
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        socket
            .emit(
                "new room",
                json!({ "thingId": thing_id, "secret": app.config.new_room_secret }),
            )
            .await
            .context("failed to create new room")?;

        println!("created new room");

        self.socket = Some(socket);
        Ok(())
    }

    pub async fn emit(&mut self, data: &HashMap<String, SensorValue>) -> anyhow::Result<()> {
        if let Some(socket) = &mut self.socket {
            println!("emitting data: {data:?}");
            socket
                .emit("data", json!(data))
                .await
                .context("failed to emit data")?;
        };
        Ok(())
    }
}
