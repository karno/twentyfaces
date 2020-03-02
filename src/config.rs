use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Yaml(serde_yaml::Error),
    UserCancelled,
}

impl error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &*self {
            ConfigError::Io(ref err) => Some(err),
            ConfigError::Yaml(ref err) => Some(err),
            UserCancelled => None,
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::Io(ref err) => write!(f, "Error: IO: {}", err),
            ConfigError::Yaml(ref err) => write!(f, "Error: YAML: {}", err),
            ConfigError::UserCancelled => write!(f, "Error: User Cancelled"),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> ConfigError {
        ConfigError::Io(err)
    }
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(err: serde_yaml::Error) -> ConfigError {
        ConfigError::Yaml(err)
    }
}

pub trait SaveAndLoad: Sized + Serialize + DeserializeOwned {
    fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_yaml::to_writer(writer, self)?;
        Ok(())
    }

    fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let payload = serde_yaml::from_reader(reader)?;
        Ok(payload)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
pub struct ApiKey {
    consumer_key: String,
    consumer_secret: String,
}

impl SaveAndLoad for ApiKey {}

impl ApiKey {
    pub fn new(consumer_key: impl Into<String>, consumer_secret: impl Into<String>) -> ApiKey {
        ApiKey {
            consumer_key: consumer_key.into(),
            consumer_secret: consumer_secret.into(),
        }
    }
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    auth_info: AuthInfo,
    profiles: Vec<Profile>,
}

impl SaveAndLoad for Config {}

impl Config {
    pub fn new(auth_info: AuthInfo, profiles: &[Profile]) -> Config {
        Config {
            auth_info,
            profiles: profiles.to_vec(),
        }
    }

    pub fn new_example(
        user_id: u64,
        token: impl Into<String>,
        secret: impl Into<String>,
    ) -> Config {
        Config {
            auth_info: AuthInfo::new(user_id, token, secret),
            profiles: vec![Profile::create_sample()],
        }
    }

    pub fn auth_info(&self) -> &AuthInfo {
        &self.auth_info
    }

    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Default, Debug, Serialize, Deserialize)]
pub struct AuthInfo {
    pub user_id: u64,
    pub token: String,
    pub secret: String,
}

impl AuthInfo {
    pub fn new(user_id: u64, token: impl Into<String>, secret: impl Into<String>) -> AuthInfo {
        AuthInfo {
            user_id: user_id,
            token: token.into(),
            secret: secret.into(),
        }
    }
}

#[derive(Clone, Hash, Default, Debug, Serialize, Deserialize)]
pub struct Profile {
    key: String,
    matches: Vec<String>,
    triggers: Vec<String>,
    derive: Option<String>,
    name: Option<String>,
    url: Option<String>,
    location: Option<String>,
    description: Option<String>,
    image: Option<String>,
    banner: Option<String>,
    intro: Option<String>,
}

impl Profile {
    pub fn create_sample() -> Profile {
        Profile {
            key: "sample".to_string(),
            matches: vec![".*change.*sample".to_string()],
            triggers: vec!["sample".to_string()],
            derive: None,
            name: Some("sample profile".to_string()),
            url: Some("example.com".to_string()),
            location: Some("sample location".to_string()),
            description: Some("this is sample profile.".to_string()),
            image: Some("~/profile_image.png".to_string()),
            banner: Some("~/banner_image.png".to_string()),
            intro: Some("Hello, I'm a example profile!".to_string()),
        }
    }
}
