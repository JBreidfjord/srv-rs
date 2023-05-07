use std::sync::Arc;

use config::Config;

pub mod config;
pub mod emitter;
pub mod error;
pub mod parser;
pub mod publisher;
pub mod sensors;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub http: reqwest::Client,
}
