mod config;
mod init;

use std::error::Error;
use std::{io, process, thread, time};

use config::*;
use tokio;
use tokio::prelude::*;

static TOKEN_FILE: &'static str = "token.yaml";
static CONFIG_FILE: &'static str = "config.yaml";

#[tokio::main]
async fn main() {
    // check existence of config file
    let api_key = init::initialize_api_key(TOKEN_FILE).expect("failed to load the app_token file.");
    init::initialize_config(CONFIG_FILE).expect("failed to load config.");
    // main_proc(conf);
}

fn main_proc(config: config::Config) {}
