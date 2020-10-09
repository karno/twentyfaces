use chrono::{Local, Utc};

use errors::{ConfigurationError, Error};
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use twitter_api::{misc::check_user_auth, models::Status, statuses};

mod config;
mod errors;
mod init;
mod twitter_api;

use std::{sync::mpsc::channel, sync::mpsc::Receiver, time::Duration};

use config::*;
use tokio::{self, time::delay_for};

static TOKEN_FILE: &str = "token.yaml";
static CONFIG_FILE: &str = "config.yaml";

#[tokio::main]
async fn main() {
    // check existence of config file
    let api_key = init::load_or_init_api_key(TOKEN_FILE)
        .await
        .expect("failed to load the token file.");
    // load or init configuration
    let conf = init::load_or_init_config(&api_key, CONFIG_FILE)
        .await
        .expect("failed to load the configuration file.");
    // check configuration validity
    let conf = check_config(&api_key, conf)
        .await
        .expect("invalid configuration detected.");
    main_proc(&api_key, CONFIG_FILE, conf).await;
}

async fn main_proc(api_key: &ApiKey, conf_file_path: &str, mut config: Config) {
    // activate config file watcher
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_millis(500)).unwrap();
    watcher
        .watch(conf_file_path, RecursiveMode::Recursive)
        .unwrap();
    println!("Press CTRL+C to exit...");
    let receive_interval = chrono::Duration::seconds(10);
    let mut last_received_id = None;
    loop {
        last_received_id = recv_and_fire_trigger(&api_key, &config, last_received_id)
            .await
            .or(last_received_id);
        let next_recv = Utc::now() + receive_interval;
        // spin wait
        loop {
            // check configuration changes and read it
            if let Some(new_config) = spin_until_update(&rx, conf_file_path, receive_interval).await
            {
                println!(
                    "Configuration file changed and reloaded at {}",
                    Local::now()
                );
                // update config if it is valid
                config = match check_config(api_key, new_config).await {
                    Ok(c) => c,
                    Err(e) => {
                        println!("[ERROR] The new token configured in the file seems be invalid.");
                        println!("        {:?}", e);
                        println!("        New configuration is not applied.");
                        config
                    }
                };
            }
            // wait until we have to acquire new timeline information
            if Utc::now() > next_recv {
                break;
            }
        }
    }
}

async fn check_config(api_key: &ApiKey, config: Config) -> Result<Config, Error> {
    check_user_auth(&api_key, config.auth_info()).await.map_err(|e| ConfigurationError::new(
        format!("Configuration error: authorization token has been invalidated or expired.\ndetail: {}", e)
    ))?;
    // check configuration validity
    config.validate(&api_key).await
}

async fn spin_until_update(
    rx: &Receiver<DebouncedEvent>,
    conf_file_path: &str,
    timeout: chrono::Duration,
) -> Option<Config> {
    let deadline = Utc::now() + timeout;
    loop {
        // check configuration has been changed
        if let Some(c) = check_config_update(rx, conf_file_path) {
            return Some(c);
        }
        // check
        if Utc::now() > deadline {
            return None;
        }
        // 0.1 msec await
        tokio::time::delay_for(Duration::from_millis(100)).await;
    }
}

fn check_config_update(rx: &Receiver<DebouncedEvent>, conf_file_path: &str) -> Option<Config> {
    match rx.try_recv() {
        Ok(event) => {
            match event {
                // fatal errors
                notify::DebouncedEvent::NoticeRemove(_)
                | notify::DebouncedEvent::Remove(_)
                | notify::DebouncedEvent::Rename(_, _) => {
                    panic!("Configuration file had been removed and that is unexpected behavior.")
                }
                notify::DebouncedEvent::Error(e, p) => {
                    panic!("Configuration watcher failed: {:?} - {:?}", e, p);
                }
                _ => {
                    // maybe the file has been changed
                    Some(
                        Config::load(conf_file_path)
                            .expect("Failed to read the configuration file"),
                    )
                }
            }
        }
        Err(err) => match err {
            std::sync::mpsc::TryRecvError::Empty => {
                // nothing to do
                None
            }
            std::sync::mpsc::TryRecvError::Disconnected => {
                // watcher failed
                panic!("Configuration watcher has been exited unexpectedly.")
            }
        },
    }
}

async fn recv_and_fire_trigger(
    api_key: &ApiKey,
    config: &Config,
    last_received: Option<u64>,
) -> Option<u64> {
    let recvd =
        statuses::user_timeline(api_key, config.auth_info(), Some(200u32), last_received).await;
    match recvd {
        Ok(mut statuses) => {
            // order by descending
            statuses.sort_by_key(|f| f.id);
            statuses.reverse();
            // pick max id
            let max_id = statuses.get(0).map(|s| s.id);
            let triggered_profile = statuses
                .into_iter()
                .filter(|s| last_received.map(|l| s.id > l).unwrap_or(true))
                .filter_map(|s| check_triggered_profile(&s, config).map(|t| (s, t)))
                .next();

            if let Some((status, profile)) = triggered_profile {
                println!(
                    "Profile \"{}\" triggered by status: {}",
                    profile.key, status.text
                );
                if last_received.is_none() {
                    println!("last_received property was not specified, so treat as dry-run mode and not triggered.");
                } else {
                    print!("applying...");
                    // profile is triggered!
                    let resolved = profile.resolve(config.profiles());
                    match resolved {
                        Ok(r) => match r.apply(api_key, config).await {
                            Ok(_) => println!(" -> applied!"),
                            Err(a) => println!(" -> failed X(\n{}", a),
                        },
                        Err(e) => println!("Invalid configuration detected: {}", e),
                    }
                }
            }

            // when profile
            max_id
        }
        Err(e) => {
            println!("[ERROR] Retriving timeline: {}", e);
            None
        }
    }
}

fn check_triggered_profile<'a>(status: &Status, config: &'a Config) -> Option<&'a Profile> {
    println!("r:{}", status.text);
    if status.retweeted_status.is_some() && !config.property().trigger_retweet {
        // this is retweet
        return None;
    }
    if status.quoted_status.is_some() && !config.property().trigger_quote {
        // this is quoted tweet
        return None;
    }
    if status.in_reply_to_status_id.is_some() && !config.property().trigger_reply {
        // this is reply
        return None;
    }
    for profile in config.profiles() {
        // triggers / patterns should not be resolved.
        if check_with_status(&profile.triggers, status.text.as_str(), check_trigger)
            || check_with_status(&profile.match_instances, status.text.as_str(), check_match)
        {
            return Some(profile);
        }
    }
    return None;
}

fn check_with_status<T, F>(candidates: &[T], text: &str, checker: F) -> bool
where
    F: Fn(&T, &str) -> bool,
{
    candidates
        .iter()
        .filter_map(|item| {
            if checker(item, text) {
                Some(true)
            } else {
                None
            }
        })
        .next()
        .unwrap_or_default()
}

fn check_trigger<T: AsRef<str>>(trigger: &T, text: &str) -> bool {
    text == trigger.as_ref()
}

fn check_match(pattern: &Regex, text: &str) -> bool {
    pattern.is_match(text)
}
