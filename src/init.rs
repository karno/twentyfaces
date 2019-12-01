use super::config;
use config::*;
use std::io;
use std::path;

pub fn initialize_api_key(token_file: &str) -> Result<Option<ApiKey>, ConfigError> {
    let token_path = path::Path::new(token_file);
    if token_path.exists() {
        match ApiKey::load(token_path) {
            Ok(token) => {
                println!("ok!");
                Ok(Some(token))
            }
            Err(e) => {
                eprintln!("failed to read application token file:");
                eprintln!("{}", e);
                Err(e)
            }
        }
    } else {
        // create
        println!("file not found. -> create token file.");
        if let Some(token) = create_api_key()? {
            token.save(token_file)?;
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }
}

fn create_api_key() -> Result<Option<ApiKey>, ConfigError> {
    use std::io::{stdin, stdout, Write};
    let mut api_key = String::new();
    let mut app_secret = String::new();
    let mut user_input = String::new();
    loop {
        println!("Application token?");
        stdout().flush().unwrap();
        api_key.clear();
        stdin().read_line(&mut api_key).expect("invalid input.");

        println!("Application secret?");
        stdout().flush().unwrap();
        app_secret.clear();
        stdin().read_line(&mut app_secret).expect("invalid input.");

        println!("Your input: ");
        println!("App Token : \"{}\".", api_key.trim());
        println!("App Secret: \"{}\".", app_secret.trim());
        loop {
            println!("Continue? ([Y]es/[n]o/[c]ancel)");
            stdout().flush().unwrap();
            user_input.clear();
            if let Ok(_) = stdin().read_line(&mut user_input) {
                match user_input.chars().next().unwrap_or('y') {
                    'Y' | 'y' => return Ok(Some(ApiKey::new(api_key.trim(), app_secret.trim()))),
                    'N' | 'n' => {
                        println!("please re-input.");
                    }
                    'C' | 'c' => {
                        println!("cancelled.");
                        return Ok(None);
                    }
                    s => {
                        println!("unknown input: {}.", s);
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

pub fn initialize_config(config_file: &str) -> Result<Option<Config>, ConfigError> {
    print!("checking configuration file...");
    let conf_path = path::Path::new(config_file);
    if conf_path.exists() {
        match Config::load(config_file) {
            Ok(conf) => {
                println!("ok!");
                Ok(Some(conf))
            }
            Err(e) => {
                eprintln!("failed to read configuration file:");
                eprintln!("{}", e);
                Err(e)
            }
        }
    } else {
        //create
        if let Some(conf) = create_config(config_file)? {
            conf.save(config_file)?;
            println!("configuration file has been generated.");
            println!("profiles could be modified even while the twentyfaces server is running.");
            Ok(Some(conf))
        } else {
            Ok(None)
        }
    }
}

fn create_config(config_file: &str) -> Result<Option<Config>, ConfigError> {
    Ok(Some(Config::new_example(
        0000,
        "sample_token",
        "sample_secret",
    )))
}
