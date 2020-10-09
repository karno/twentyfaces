use crate::twitter_api::auth;

use super::config;
use super::twitter_api::misc;
use config::*;
use futures::future::TryFutureExt;
use std::iter::Iterator;
use std::path;

pub async fn load_or_init_api_key(token_file: &str) -> Result<ApiKey, ConfigError> {
    let token_path = path::Path::new(token_file);
    if token_path.exists() {
        ApiKey::load(token_path)
    } else {
        // create api key
        println!("token file not found -> create token file.");
        let token = create_api_key().await?;
        token.save(token_file)?;
        Ok(token)
    }
}

async fn create_api_key() -> Result<ApiKey, ConfigError> {
    async {
        acquire_user_input(&["App token", "App secret"])
            .map(|input| ApiKey::new(&input[0], &input[1]))
            .ok_or(ConfigError::UserCancelled)
    }
    .and_then(|k| validate_api_key(k))
    .await
}

async fn validate_api_key(api_key: ApiKey) -> Result<ApiKey, ConfigError> {
    misc::check_api_key(&api_key).await?;
    Ok(api_key)
}

pub async fn load_or_init_config(
    api_key: &ApiKey,
    config_file: &str,
) -> Result<Config, ConfigError> {
    let conf_path = path::Path::new(config_file);
    if conf_path.exists() {
        Config::load(config_file)
    } else {
        // create configuration
        println!("config file not found -> create config file.");
        let config = create_config(api_key).await?;
        config.save(config_file)?;
        Ok(config)
    }
}

async fn create_config(api_key: &ApiKey) -> Result<Config, ConfigError> {
    let auth_info = create_token(api_key).await?;
    Ok(Config::new_example(auth_info))
}

async fn create_token(api_key: &ApiKey) -> Result<AuthInfo, ConfigError> {
    let req_token = auth::request_token(api_key).await?;
    println!(
        "Please access to proceed the authorization: {}",
        auth::get_authorization_url(&req_token)
    );
    let pin = acquire_user_input(&["PIN"]).ok_or(ConfigError::UserCancelled)?;
    let pin = pin.into_iter().next().ok_or(ConfigError::UserCancelled)?;
    let acc_token = auth::access_token(api_key, req_token, pin).await?;
    let user_id = acc_token.remain.get("user_id").unwrap();
    let user_id = user_id.parse::<u64>().unwrap();

    Ok(AuthInfo::new(
        user_id,
        acc_token.oauth_token,
        acc_token.oauth_token_secret,
    ))
}

fn acquire_user_input<'a>(keys: &[&'a str]) -> Option<Vec<String>> {
    use std::io::{stdin, stdout, Write};
    let mut inputs = Vec::new();
    loop {
        inputs.clear();
        assert!(inputs.is_empty());
        // acquire user inputs...
        for &key in keys {
            let mut input = String::new();
            print!("{}? :", key);
            stdout().flush().unwrap();
            stdin().read_line(&mut input).expect("invalid input.");
            inputs.push(input.trim().to_string());
        }
        assert!(inputs.len() == keys.len());

        // review the input
        println!("Your input ... ");
        for (&key, input) in keys.iter().zip(inputs.iter()) {
            println!("{} : \"{}\".", key, input);
        }
        print!(" ... is correct? ([Y]es/[n]o/[c]ancel): ");

        // user check...
        loop {
            let mut input = String::new();
            stdout().flush().unwrap();
            input.clear();
            if stdin().read_line(&mut input).is_ok() {
                match input.chars().next().unwrap_or('y') {
                    'Y' | 'y' | '\n' => return Some(inputs),
                    'N' | 'n' => {
                        println!("please re-input.");
                        print!("Your input is correct? ([Y]es/[n]o/[c]ancel): ");
                    }
                    'C' | 'c' => {
                        println!("cancelled.");
                        return None;
                    }
                    s => {
                        println!("unknown input: \"{}\".", s);
                        continue;
                    }
                }
            } else {
                print!("invalid input. ");
            }
            break;
        }
    }
}
