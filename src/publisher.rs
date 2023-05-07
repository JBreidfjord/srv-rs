use std::collections::HashMap;

use anyhow::Context;
use serde_json::json;

use crate::{sensors::SensorValue, AppState};

pub struct Publisher {
    conn: redis::Connection,
    thing_id: String,
    published: bool,
}

impl Publisher {
    pub fn connect(app: AppState, thing_id: String) -> anyhow::Result<Self> {
        let url = format!("redis://{}:{}", app.config.redis_url, app.config.redis_port);
        let redis = redis::Client::open(url).context("failed to connect to redis")?;
        let conn = redis
            .get_connection()
            .context("failed to get redis connection")?;

        Ok(Self {
            conn,
            thing_id,
            published: false,
        })
    }

    pub fn publish_connection(&mut self) -> anyhow::Result<()> {
        if self.published {
            return Ok(());
        }

        redis::cmd("PUBLISH")
            .arg("THING_CONNECTION")
            .arg(
                json!({
                    "active": true,
                    "THING": self.thing_id
                })
                .to_string(),
            )
            .query(&mut self.conn)?;

        redis::cmd("DEL")
            .arg(format!("THING_{}", self.thing_id))
            .query(&mut self.conn)?;

        self.published = true;
        Ok(())
    }

    pub fn publish_disconnection(&mut self) -> anyhow::Result<()> {
        if !self.published {
            return Ok(());
        }

        redis::cmd("PUBLISH")
            .arg(format!("THING_{}", self.thing_id))
            .arg(
                json!({
                    "active": false,
                    "THING": self.thing_id
                })
                .to_string(),
            )
            .query(&mut self.conn)?;

        self.published = false;
        Ok(())
    }

    pub fn push_snapshots(
        &mut self,
        snapshots: &[HashMap<String, SensorValue>],
    ) -> anyhow::Result<()> {
        redis::cmd("RPUSH")
            .arg(format!("THING_{}", self.thing_id))
            .arg(
                snapshots
                    .iter()
                    .map(|snapshot| json!(snapshot).to_string().replace(' ', ""))
                    .collect::<Vec<_>>()
                    .join(" "),
            )
            .query(&mut self.conn)?;

        Ok(())
    }
}
