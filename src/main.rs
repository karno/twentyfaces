mod config;
mod init;
mod twitter_api;

use std::error::Error;
use std::{io, process, thread, time};

use config::*;
use futures::future::TryFutureExt;
use tokio;
use tokio::prelude::*;

static TOKEN_FILE: &str = "token.yaml";
static CONFIG_FILE: &str = "config.yaml";

#[tokio::main]
async fn main() {
    // check existence of config file
    let api_key = init::load_or_init_api_key(TOKEN_FILE)
        .await
        .expect("failed to load the token file.");
    let conf = init::load_or_init_config(&api_key, CONFIG_FILE)
        .await
        .expect("failed to load the configuration file.");
    main_proc(&api_key, &conf);
}

fn main_proc(api_key: &config::ApiKey, config: &config::Config) {
    println!("id: {}", config.auth_info().user_id);
    println!("token: {}", config.auth_info().token);
    println!("secret: {}", config.auth_info().secret);
}

fn proc_triggered() {}

fn prepare_config() -> Option<Config> {
    None
}
